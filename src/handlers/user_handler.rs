use axum::{extract::State, Json};
use crate::dto::user::{UserProfileResponse, UpdateUserRequest, UserListResponse};
use crate::services::user_service::UserService;
use crate::services::audit::AuditService;
use crate::middlewares::auth::UserContext;
use crate::AppState;
use crate::errors::app_error::AppError;

/// Helper: extract client IP from headers
fn extract_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
        .or_else(|| headers.get("host").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown")
        .to_string()
}

#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Current user profile", body = UserProfileResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
) -> Result<Json<UserProfileResponse>, AppError> {
    let ip = extract_ip(&headers);
    let res = UserService::get_profile(&state.db, user_ctx.user_id).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_GET_PROFILE", "GET", "/api/users/me", 200, &ip,
        None,
    ).await;

    Ok(Json(res))
}

#[utoipa::path(
    put,
    path = "/api/users/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserProfileResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let ip = extract_ip(&headers);
    let res = UserService::update_profile(&state.db, user_ctx.user_id, req).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_PROFILE_UPDATED", "PUT", "/api/users/me", 200, &ip,
        Some(serde_json::json!({"updated_fields": "name/phone/address"})),
    ).await;

    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/users/me/photo",
    request_body(content = mikrotik_service::dto::user::UploadPhotoRequest, description = "Image file via multipart/form-data. Max ~5MB.", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Photo successfully processed and uploaded", body = UserProfileResponse),
        (status = 400, description = "Bad request (Not an image, or too large)"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn upload_photo(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<UserProfileResponse>, AppError> {
    let ip = extract_ip(&headers);
    let mut photo_bytes = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "photo" {
            let data = field.bytes().await.map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?;
            photo_bytes = data.to_vec();
            break;
        }
    }

    if photo_bytes.is_empty() {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_PHOTO_UPLOAD_FAILED", "POST", "/api/users/me/photo", 400, &ip,
            Some(serde_json::json!({"error": "No image file provided"})),
        ).await;
        return Err(AppError::BadRequest("No image file provided".to_string()));
    }

    let public_url = crate::services::storage_service::StorageService::process_and_upload_image(&state.storage, &photo_bytes).await?;

    let update_req = UpdateUserRequest {
        name: None,
        phone: None,
        address: None,
        photo: Some(public_url.clone()),
    };

    let res = UserService::update_profile(&state.db, user_ctx.user_id, update_req).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_PHOTO_UPLOADED", "POST", "/api/users/me/photo", 200, &ip,
        Some(serde_json::json!({"photo_url": public_url})),
    ).await;

    Ok(Json(res))
}

#[utoipa::path(
    get,
    path = "/api/users",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("page_size" = Option<u64>, Query, description = "Page size")
    ),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_users(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    axum::extract::Query(params): axum::extract::Query<serde_json::Value>,
) -> Result<Json<UserListResponse>, AppError> {
    let ip = extract_ip(&headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_LIST_FORBIDDEN", "GET", "/api/users", 403, &ip,
            Some(serde_json::json!({"error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    let page = params.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
    let page_size = params.get("page_size").and_then(|v| v.as_u64()).unwrap_or(10);

    let res = UserService::find_all(&state.db, page, page_size).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_LIST_ACCESSED", "GET", "/api/users", 200, &ip,
        Some(serde_json::json!({"page": page, "page_size": page_size, "total": res.total})),
    ).await;

    Ok(Json(UserListResponse {
        items: res.items,
        total: res.total,
        page: res.page,
        page_size: res.page_size,
    }))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    responses(
        (status = 200, description = "User profile", body = UserProfileResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let ip = extract_ip(&headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_DETAIL_FORBIDDEN", "GET", "/api/users/{id}", 403, &ip,
            Some(serde_json::json!({"target_user_id": id, "error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    let res = UserService::get_profile(&state.db, id).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_DETAIL_ACCESSED", "GET", "/api/users/{id}", 200, &ip,
        Some(serde_json::json!({"target_user_id": id})),
    ).await;

    Ok(Json(res))
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let ip = extract_ip(&headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_DELETE_FORBIDDEN", "DELETE", "/api/users/{id}", 403, &ip,
            Some(serde_json::json!({"target_user_id": id, "error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    UserService::soft_delete(&state.db, id).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_DELETED", "DELETE", "/api/users/{id}", 200, &ip,
        Some(serde_json::json!({"deleted_user_id": id})),
    ).await;

    Ok(axum::http::StatusCode::OK)
}
