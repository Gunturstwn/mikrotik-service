use crate::AppState;
use crate::dto::mikrotik::{
    MikrotikClientRequest, MikrotikClientResponse, MikrotikResourceResponse,
};
use crate::errors::app_error::AppError;
use crate::middlewares::auth::UserContext;
use crate::services::audit::AuditService;
use crate::services::mikrotik_service::MikrotikService;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::json;
use uuid::Uuid;

// ... existing helper and handlers ...

/// Get real-time system resource usage from the MikroTik device.
///
/// ### Resource Metrics:
/// - **Uptime**: Time since last reboot.
/// - **CPU Load**: Current CPU percentage.
/// - **Memory**: Free and total RAM.
/// - **HDD**: Free and total disk space.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/system/resource/print",
    responses(
        (status = 200, description = "System resource info", body = MikrotikResourceResponse),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "MikroTik connection failed")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn get_system_resource(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<MikrotikResourceResponse>, AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let res =
        MikrotikService::get_system_resource(&state.db, &state.mikrotik_pool, id, &aes_key).await?;

    // Audit log
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_READ",
        "GET",
        &format!("/api/mikrotik_client/{}", id),
        200,
        &ip,
        Some(json!({ "action": "system_resource_print", "device_id": id })),
    )
    .await;

    Ok(Json(res))
}

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

/// Create a new MikroTik device client.
///
/// ### Security Notes:
/// - Sensitive fields (username, password, ports) are **AES-256-GCM encrypted** at rest.
/// - Credentials are only decrypted in memory during active connection attempts.
#[utoipa::path(
    post,
    path = "/api/mikrotik_client",
    request_body = MikrotikClientRequest,
    responses(
        (status = 201, description = "MikroTik device created successfully", body = MikrotikClientResponse),
        (status = 400, description = "Invalid request or encryption failure", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Access denied"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn create_client(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Json(payload): Json<MikrotikClientRequest>,
) -> Result<(axum::http::StatusCode, Json<MikrotikClientResponse>), AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let res =
        MikrotikService::create_client(&state.db, user_ctx.user_id, payload.clone(), &aes_key)
            .await?;

    // Audit log with masked payload
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_CREATE",
        "POST",
        "/api/mikrotik_client",
        201,
        &ip,
        Some(json!({
            "name_device": payload.name_device,
            "host": payload.host,
            "username": "encrypted",
            "password": "encrypted",
            "port_winbox": payload.port_winbox.as_ref().map(|_| "encrypted"),
            "port_api": payload.port_api.as_ref().map(|_| "encrypted"),
            "port_ftp": payload.port_ftp.as_ref().map(|_| "encrypted"),
            "port_ssh": payload.port_ssh.as_ref().map(|_| "encrypted"),
            "location": payload.location,
        })),
    )
    .await;

    Ok((axum::http::StatusCode::CREATED, Json(res)))
}

/// List all registered MikroTik devices.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client",
    responses(
        (status = 200, description = "List of devices", body = Vec<MikrotikClientResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn list_clients(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
) -> Result<Json<Vec<MikrotikClientResponse>>, AppError> {
    let ip = extract_ip(&headers);
    let res = MikrotikService::list_clients(&state.db).await?;

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_READ",
        "GET",
        "/api/mikrotik_client",
        200,
        &ip,
        None,
    )
    .await;

    Ok(Json(res))
}

/// Get detailed information of a single MikroTik device.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}",
    responses(
        (status = 200, description = "Device details", body = MikrotikClientResponse),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn get_client(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<MikrotikClientResponse>, AppError> {
    let ip = extract_ip(&headers);
    let res = MikrotikService::get_client(&state.db, id).await?;

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_READ",
        "GET",
        &format!("/api/mikrotik_client/{}", id),
        200,
        &ip,
        None,
    )
    .await;

    Ok(Json(res))
}

/// Update an existing MikroTik device configuration.
#[utoipa::path(
    put,
    path = "/api/mikrotik_client/{id}",
    request_body = MikrotikClientRequest,
    responses(
        (status = 200, description = "Device updated successfully", body = MikrotikClientResponse),
        (status = 404, description = "Device not found"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn update_client(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Json(payload): Json<MikrotikClientRequest>,
) -> Result<Json<MikrotikClientResponse>, AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let res =
        MikrotikService::update_client(&state.db, id, user_ctx.user_id, payload.clone(), &aes_key)
            .await?;

    // Invalidate connection in pool
    state.mikrotik_pool.invalidate(id);

    // Audit log with masked payload
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_UPDATE",
        "PUT",
        &format!("/api/mikrotik_client/{}", id),
        200,
        &ip,
        Some(json!({
            "name_device": payload.name_device,
            "host": payload.host,
            "username": "encrypted",
            "password": "encrypted",
            "port_winbox": payload.port_winbox.as_ref().map(|_| "encrypted"),
            "port_api": payload.port_api.as_ref().map(|_| "encrypted"),
            "port_ftp": payload.port_ftp.as_ref().map(|_| "encrypted"),
            "port_ssh": payload.port_ssh.as_ref().map(|_| "encrypted"),
            "location": payload.location,
        })),
    )
    .await;

    Ok(Json(res))
}

/// Remove a MikroTik device (Soft Delete).
#[utoipa::path(
    delete,
    path = "/api/mikrotik_client/{id}",
    responses(
        (status = 204, description = "Device removed successfully"),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn delete_client(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let ip = extract_ip(&headers);

    MikrotikService::delete_client(&state.db, id, user_ctx.user_id).await?;

    // Invalidate connection in pool
    state.mikrotik_pool.invalidate(id);

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_DELETE",
        "DELETE",
        &format!("/api/mikrotik_client/{}", id),
        204,
        &ip,
        None,
    )
    .await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
