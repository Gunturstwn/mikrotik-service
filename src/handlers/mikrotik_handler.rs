use crate::AppState;
use crate::dto::mikrotik::{
    MikrotikClientRequest, MikrotikClientResponse, MikrotikResourceResponse, MikrotikInterfaceResponse,
    MikrotikConfigSnapshotResponse, MikrotikConfigViewResponse, MikrotikConfigDiffResponse,
};
use axum::http::{StatusCode, HeaderMap};
use crate::errors::app_error::AppError;
use crate::middlewares::auth::UserContext;
use crate::services::audit::AuditService;
use crate::services::mikrotik_service::MikrotikService;
use serde_json::json;
use uuid::Uuid;
use axum::{
    extract::{Path, State, Query},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures_util::StreamExt;
use std::convert::Infallible;

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
        MikrotikService::get_system_resource(&state.db, &state.mikrotik_pool, id, &aes_key, Some(user_ctx.user_id)).await?;

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

/// Get real-time interface statistics from the MikroTik device.
/// 
/// This endpoint provides detailed information about each interface, 
/// including RX/TX bytes and packets, error counts, and status.
/// Useful for bandwidth monitoring.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/interfaces/print",
    responses(
        (status = 200, description = "Interface list and stats", body = Vec<MikrotikInterfaceResponse>),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "MikroTik communication failed")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn get_interfaces(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<MikrotikInterfaceResponse>>, AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let res = MikrotikService::get_interfaces(&state.db, &state.mikrotik_pool, id, &aes_key, Some(user_ctx.user_id)).await?;

    // Audit log
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_INTERFACE_READ",
        "GET",
        &format!("/api/mikrotik_client/{}/interfaces/print", id),
        200,
        &ip,
        Some(json!({ "action": "interface_print", "device_id": id })),
    )
    .await;

    Ok(Json(res))
}

#[derive(serde::Deserialize)]
pub struct MonitorParams {
    pub interface: Option<String>,
}

/// Stream real-time interface traffic (rx/tx bits per second).
/// 
/// This endpoint uses **Server-Sent Events (SSE)** to provide continuous 
/// throughput updates from the MikroTik device.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/interfaces/monitor",
    responses(
        (status = 200, description = "SSE stream of interface traffic", body = Vec<MikrotikMonitorResponse>),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID"),
        ("interface" = Option<String>, Query, description = "Comma-separated interface names (default: all)")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn monitor_interfaces(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Query(params): Query<MonitorParams>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let stream = MikrotikService::monitor_interfaces(
        state.db.clone(), 
        state.mikrotik_pool.clone(), 
        id, 
        aes_key, 
        Some(user_ctx.user_id),
        params.interface.clone()
    ).await?;

    // Audit log for starting monitor
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_MONITOR",
        "GET",
        &format!("/api/mikrotik_client/{}/interfaces/monitor", id),
        200,
        &ip,
        Some(json!({ "action": "monitor_start", "device_id": id, "interfaces": params.interface })),
    )
    .await;

    let event_stream = stream.map(|result| {
        match result {
            Ok(data) => Ok::<Event, Infallible>(Event::default().json_data(data).unwrap()),
            Err(e) => Ok::<Event, Infallible>(Event::default().event("error").data(e.to_string())),
        }
    });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

#[derive(serde::Deserialize)]
pub struct TorchParams {
    pub interface: String,
}

/// Stream real-time traffic identification (Top Talkers) using Torch.
/// 
/// This endpoint uses **Server-Sent Events (SSE)** to provide continuous 
/// updates on source/destination IPs, protocols, and rates.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/interfaces/torch",
    responses(
        (status = 200, description = "SSE stream of torch data", body = Vec<MikrotikTorchResponse>),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID"),
        ("interface" = String, Query, description = "Interface to monitor")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn get_torch(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Query(params): Query<TorchParams>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let stream = MikrotikService::get_torch(
        state.db.clone(), 
        state.mikrotik_pool.clone(), 
        id, 
        aes_key, 
        Some(user_ctx.user_id),
        params.interface.clone()
    ).await?;

    // Audit log for starting torch
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_TORCH",
        "GET",
        &format!("/api/mikrotik_client/{}/interfaces/torch", id),
        200,
        &ip,
        Some(json!({ "action": "torch_start", "device_id": id, "interface": params.interface })),
    )
    .await;

    let event_stream = stream.map(|result| {
        match result {
            Ok(data) => Ok::<Event, Infallible>(Event::default().json_data(data).unwrap()),
            Err(e) => Ok::<Event, Infallible>(Event::default().event("error").data(e.to_string())),
        }
    });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

/// Get configuration backup history for a MikroTik device.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/config/history",
    responses(
        (status = 200, description = "List of configuration snapshots", body = Vec<MikrotikConfigSnapshotResponse>),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Config"
)]
pub async fn get_config_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<MikrotikConfigSnapshotResponse>>, AppError> {
    let history = MikrotikService::get_config_history(&state.db, id).await?;

    let ip = extract_ip(&headers);
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CONFIG_READ",
        "GET",
        &format!("/api/mikrotik_client/{}/config/history", id),
        200,
        &ip,
        Some(serde_json::json!({ "action": "config_history", "device_id": id, "snapshot_count": history.len() })),
    ).await;

    Ok(Json(history))
}

/// View raw configuration content of a specific snapshot.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/config/view/{snapshot_id}",
    responses(
        (status = 200, description = "Raw configuration content", body = MikrotikConfigViewResponse),
        (status = 404, description = "Snapshot not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID"),
        ("snapshot_id" = Uuid, Path, description = "Snapshot ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Config"
)]
pub async fn view_config_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path((id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MikrotikConfigViewResponse>, AppError> {
    let snapshot = MikrotikService::get_config_snapshot(&state.db, snapshot_id).await?;

    let ip = extract_ip(&headers);
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CONFIG_READ",
        "GET",
        &format!("/api/mikrotik_client/{}/config/view/{}", id, snapshot_id),
        200,
        &ip,
        Some(serde_json::json!({ "action": "config_view", "device_id": id, "snapshot_id": snapshot_id })),
    ).await;

    Ok(Json(snapshot))
}

/// Trigger an immediate configuration backup.
#[utoipa::path(
    post,
    path = "/api/mikrotik_client/{id}/config/backup-now",
    responses(
        (status = 201, description = "Backup created or deduplicated", body = MikrotikConfigSnapshotResponse),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Config"
)]
pub async fn backup_now(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<MikrotikConfigSnapshotResponse>), AppError> {
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let result = MikrotikService::perform_versioned_backup(
        &state.db,
        &state.mikrotik_pool,
        id,
        &aes_key,
        Some(user_ctx.user_id)
    ).await?;

    let ip = extract_ip(&headers);
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CONFIG_BACKUP",
        "POST",
        &format!("/api/mikrotik_client/{}/config/backup-now", id),
        201,
        &ip,
        Some(json!({ "snapshot_id": result.id, "hash": result.config_hash })),
    ).await;

    Ok((StatusCode::CREATED, Json(result)))
}

#[derive(serde::Deserialize)]
pub struct ConfigDiffParams {
    pub v1: Uuid,
    pub v2: Uuid,
}

/// Compare two configuration snapshots.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/config/diff",
    responses(
        (status = 200, description = "Configuration diff", body = MikrotikConfigDiffResponse),
        (status = 404, description = "Snapshot(s) not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID"),
        ("v1" = Uuid, Query, description = "First Snapshot ID"),
        ("v2" = Uuid, Query, description = "Second Snapshot ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Config"
)]
pub async fn get_config_diff(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Query(params): Query<ConfigDiffParams>,
) -> Result<Json<MikrotikConfigDiffResponse>, AppError> {
    let diff = MikrotikService::get_config_diff(&state.db, params.v1, params.v2).await?;

    let ip = extract_ip(&headers);
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CONFIG_DIFF",
        "GET",
        &format!("/api/mikrotik_client/{}/config/diff", id),
        200,
        &ip,
        Some(serde_json::json!({ "v1": params.v1, "v2": params.v2, "device_id": id })),
    ).await;

    Ok(Json(diff))
}

/// Test connection to a MikroTik device.
/// 
/// Returns 200 OK if connection/authentication is successful, 
/// or 503 Service Unavailable if it fails.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/test-connection",
    responses(
        (status = 200, description = "Connection successful"),
        (status = 503, description = "Connection failed"),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Client"
)]
pub async fn test_connection_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let ip = extract_ip(&headers);
    let aes_key = std::env::var("AES_KEY")
        .map_err(|_| AppError::InternalServerError("AES_KEY not configured".to_string()))?;

    let is_connected = MikrotikService::check_connectivity(&state.db, &state.mikrotik_pool, id, &aes_key, Some(user_ctx.user_id)).await?;

    // Audit log
    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_CLIENT_TEST",
        "GET",
        &format!("/api/mikrotik_client/{}/test-connection", id),
        if is_connected { 200 } else { 503 },
        &ip,
        Some(json!({ "connected": is_connected })),
    )
    .await;

    if is_connected {
        Ok(axum::http::StatusCode::OK)
    } else {
        Ok(axum::http::StatusCode::SERVICE_UNAVAILABLE)
    }
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
