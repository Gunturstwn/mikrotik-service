use axum::extract::State;
use crate::services::user_service::UserService;
use crate::services::audit::AuditService;
use crate::middlewares::auth::UserContext;
use crate::AppState;
use crate::errors::app_error::AppError;
use crate::export::{CSVExporter, ExcelExporter};
use crate::utils::ip::extract_ip_from_headers;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;



#[utoipa::path(
    get,
    path = "/api/export/users/csv",
    responses(
        (status = 200, description = "CSV file download", content_type = "text/csv"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Super Admin required)")
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_users_csv(
    State(state): State<AppState>,
    req_headers: axum::http::HeaderMap,
    user_ctx: UserContext,
) -> Result<impl IntoResponse, AppError> {
    let ip = extract_ip_from_headers(&req_headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "EXPORT_CSV_FORBIDDEN", "GET", "/api/export/users/csv", 403, &ip,
            Some(serde_json::json!({"error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    let res = UserService::find_all(&state.db, 1, 10000).await?;
    let total = res.total;
    let csv_bytes = CSVExporter::export_users(res.items)?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "EXPORT_CSV_SUCCESS", "GET", "/api/export/users/csv", 200, &ip,
        Some(serde_json::json!({"total_exported": total})),
    ).await;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/csv".parse().map_err(|e| AppError::InternalServerError(format!("Invalid content type: {}", e)))?);
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"users_export.csv\"".parse().map_err(|e| AppError::InternalServerError(format!("Invalid content disposition: {}", e)))?,
    );

    Ok((StatusCode::OK, headers, csv_bytes))
}

#[utoipa::path(
    get,
    path = "/api/export/users/xlsx",
    responses(
        (status = 200, description = "Excel file download", content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Super Admin required)")
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_users_xlsx(
    State(state): State<AppState>,
    req_headers: axum::http::HeaderMap,
    user_ctx: UserContext,
) -> Result<impl IntoResponse, AppError> {
    let ip = extract_ip_from_headers(&req_headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "EXPORT_XLSX_FORBIDDEN", "GET", "/api/export/users/xlsx", 403, &ip,
            Some(serde_json::json!({"error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    let res = UserService::find_all(&state.db, 1, 10000).await?;
    let total = res.total;
    let xlsx_bytes = ExcelExporter::export_users(res.items)?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "EXPORT_XLSX_SUCCESS", "GET", "/api/export/users/xlsx", 200, &ip,
        Some(serde_json::json!({"total_exported": total})),
    ).await;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            .parse()
            .map_err(|e| AppError::InternalServerError(format!("Invalid content type: {}", e)))?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"users_export.xlsx\"".parse().map_err(|e| AppError::InternalServerError(format!("Invalid content disposition: {}", e)))?,
    );

    Ok((StatusCode::OK, headers, xlsx_bytes))
}
