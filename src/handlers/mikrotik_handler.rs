use crate::AppState;
use crate::dto::mikrotik::{
    MikrotikClientRequest, MikrotikClientResponse, MikrotikResourceResponse, MikrotikInterfaceResponse,
    MikrotikConfigSnapshotResponse, MikrotikConfigViewResponse, MikrotikConfigDiffResponse,
    BackupCreateRequest, BackupFileResponse, BackupAndSendRequest, BackupAndSendResponse,
    BackupLogListResponse, BackupLogResponse,
};
use axum::http::{StatusCode, HeaderMap, HeaderValue};
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
use validator::Validate;
use crate::utils::ip::extract_ip_from_headers;

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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let res =
        MikrotikService::get_system_resource(&state.db, &state.mikrotik_pool, id, aes_key, Some(user_ctx.user_id)).await?;

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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let res = MikrotikService::get_interfaces(&state.db, &state.mikrotik_pool, id, aes_key, Some(user_ctx.user_id)).await?;

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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let stream = MikrotikService::monitor_interfaces(
        state.db.clone(), 
        state.mikrotik_pool.clone(), 
        id, 
        aes_key.to_string(), 
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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let stream = MikrotikService::get_torch(
        state.db.clone(), 
        state.mikrotik_pool.clone(), 
        id, 
        aes_key.to_string(), 
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

    let ip = extract_ip_from_headers(&headers);
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

    let ip = extract_ip_from_headers(&headers);
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
    let aes_key = crate::config::mikrotik::get_aes_key();

    let result = MikrotikService::perform_versioned_backup(
        &state.db,
        &state.mikrotik_pool,
        id,
        aes_key,
        Some(user_ctx.user_id)
    ).await?;

    let ip = extract_ip_from_headers(&headers);
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

    let ip = extract_ip_from_headers(&headers);
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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let is_connected = MikrotikService::check_connectivity(&state.db, &state.mikrotik_pool, id, aes_key, Some(user_ctx.user_id)).await?;

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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    // Validasi input fields
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let res =
        MikrotikService::create_client(&state.db, user_ctx.user_id, payload.clone(), aes_key)
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
    let ip = extract_ip_from_headers(&headers);
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
    let ip = extract_ip_from_headers(&headers);
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
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    // Validasi input fields
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let res =
        MikrotikService::update_client(&state.db, id, user_ctx.user_id, payload.clone(), aes_key)
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
    let ip = extract_ip_from_headers(&headers);

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

/// Trigger a binary system backup on the MikroTik device.
///
/// Executes `/system backup save` on the device. The backup file (.backup)
/// is stored on the device's local filesystem and can be downloaded afterwards.
#[utoipa::path(
    post,
    path = "/api/mikrotik_client/{id}/backup",
    request_body = BackupCreateRequest,
    responses(
        (status = 201, description = "Backup triggered successfully", body = Object),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "MikroTik backup failed")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Backup"
)]
pub async fn trigger_backup_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Json(payload): Json<BackupCreateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let filename = MikrotikService::trigger_backup(
        &state.db,
        &state.mikrotik_pool,
        id,
        aes_key,
        Some(user_ctx.user_id),
        payload.name.as_deref(),
        payload.password.as_deref(),
    ).await?;

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_BACKUP_CREATE",
        "POST",
        &format!("/api/mikrotik_client/{}/backup", id),
        201,
        &ip,
        Some(json!({
            "device_id": id,
            "filename": filename,
            "encrypted": payload.password.is_some(),
        })),
    ).await;

    Ok((StatusCode::CREATED, Json(json!({ "filename": filename }))))
}

#[derive(serde::Deserialize)]
pub struct BackupLogQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// List backup activity logs (from audit_logs filtered by MIKROTIK_BACKUP_SEND).
///
/// Returns enriched data with device names and bot names.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/backup-logs",
    responses(
        (status = 200, description = "Paginated list of backup activity logs", body = BackupLogListResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("page" = Option<u64>, Query, description = "Page number (1-based)"),
        ("page_size" = Option<u64>, Query, description = "Items per page (max 100)"),
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Backup"
)]
pub async fn list_backup_logs_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Query(params): Query<BackupLogQuery>,
) -> Result<Json<BackupLogListResponse>, AppError> {
    use crate::models::audit_logs::Entity as AuditLogEntity;
    use crate::models::mikrotik_clients::Entity as MikrotikClientEntity;
    use crate::models::telegram_bots::Entity as TelegramBotEntity;
    use sea_orm::*;
    use std::collections::HashMap;

    let ip = extract_ip_from_headers(&headers);
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);

    // Filter by backup send action
    let condition = crate::models::audit_logs::Column::Action.eq("MIKROTIK_BACKUP_SEND");

    let total = AuditLogEntity::find()
        .filter(condition.clone())
        .count(&state.db)
        .await?;

    let total_pages = (total as f64 / page_size as f64).ceil() as u64;
    let offset = (page - 1) * page_size;

    let audit_items = AuditLogEntity::find()
        .filter(condition)
        .order_by_desc(crate::models::audit_logs::Column::CreatedAt)
        .offset(offset)
        .limit(page_size)
        .all(&state.db)
        .await?;

    // Collect device IDs and bot IDs for batch resolution
    let mut device_ids = Vec::new();
    let mut bot_ids = Vec::new();
    for item in &audit_items {
        if let Some(ref meta) = item.metadata {
            if let Some(did) = meta.get("device_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok()) {
                device_ids.push(did);
            }
            if let Some(bid) = meta.get("telegram_bot_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok()) {
                bot_ids.push(bid);
            }
        }
    }

    // Batch resolve device names
    let device_map: HashMap<Uuid, (String, String)> = if !device_ids.is_empty() {
        MikrotikClientEntity::find()
            .filter(crate::models::mikrotik_clients::Column::Id.is_in(device_ids))
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|d| (d.id, (d.name_device, d.host)))
            .collect()
    } else {
        HashMap::new()
    };

    // Batch resolve bot names
    let bot_map: HashMap<Uuid, String> = if !bot_ids.is_empty() {
        TelegramBotEntity::find()
            .filter(crate::models::telegram_bots::Column::Id.is_in(bot_ids))
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|b| (b.id, b.name))
            .collect()
    } else {
        HashMap::new()
    };

    // Resolve user names
    let user_ids: Vec<Uuid> = audit_items.iter().filter_map(|m| m.user_id).collect();
    let user_map: HashMap<Uuid, String> = if !user_ids.is_empty() {
        crate::models::users::Entity::find()
            .filter(crate::models::users::Column::Id.is_in(user_ids))
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|u| (u.id, u.name))
            .collect()
    } else {
        HashMap::new()
    };

    let items: Vec<BackupLogResponse> = audit_items.into_iter().map(|m| {
        let meta = m.metadata.as_ref();
        let device_id: Option<Uuid> = meta.and_then(|v| v.get("device_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()));
        let bot_id: Option<Uuid> = meta.and_then(|v| v.get("telegram_bot_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()));

        let (device_name, device_host) = device_id.and_then(|did| device_map.get(&did)).cloned().unwrap_or(("Unknown".to_string(), "".to_string()));
        let bot_name = bot_id.and_then(|bid| bot_map.get(&bid)).cloned();
        let user_name = m.user_id.and_then(|uid| user_map.get(&uid)).cloned();

        BackupLogResponse {
            id: m.id,
            device_name,
            device_host,
            filename: meta.and_then(|v| v.get("filename").and_then(|v| v.as_str().map(|s| s.to_string()))).unwrap_or_default(),
            format: meta.and_then(|v| v.get("format").and_then(|v| v.as_str().map(|s| s.to_string()))),
            telegram_bot_name: bot_name,
            telegram_success: meta.and_then(|v| v.get("telegram_success").and_then(|v| v.as_bool())).unwrap_or(false),
            deleted_from_device: meta.and_then(|v| v.get("deleted_from_device").and_then(|v| v.as_bool())).unwrap_or(false),
            user_name,
            created_at: m.created_at,
        }
    }).collect();

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "MIKROTIK_BACKUP_LOG_LIST", "GET", "/api/mikrotik_client/backup-logs", 200, &ip,
        Some(serde_json::json!({"total": total})),
    ).await;

    Ok(Json(BackupLogListResponse { items, total, page, page_size, total_pages }))
}

#[derive(serde::Deserialize)]
pub struct DownloadBackupParams {
    pub filename: String,
}

/// List backup files (.backup) on the MikroTik device's filesystem.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/backup/files",
    responses(
        (status = 200, description = "List of .backup files on the device", body = Vec<BackupFileResponse>),
        (status = 404, description = "Device not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Backup"
)]
pub async fn list_backup_files_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<BackupFileResponse>>, AppError> {
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let files = MikrotikService::list_backup_files(
        &state.db,
        &state.mikrotik_pool,
        id,
        aes_key,
        Some(user_ctx.user_id),
    ).await?;

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_BACKUP_LIST",
        "GET",
        &format!("/api/mikrotik_client/{}/backup/files", id),
        200,
        &ip,
        Some(json!({
            "device_id": id,
            "file_count": files.len(),
        })),
    ).await;

    Ok(Json(files))
}

/// Create a backup and send it to a Telegram bot (with optional auto-delete).
///
/// Flow: create backup (.backup or .rsc) → download → send to Telegram → delete from device (if requested).
#[utoipa::path(
    post,
    path = "/api/mikrotik_client/{id}/backup/send",
    request_body = BackupAndSendRequest,
    responses(
        (status = 201, description = "Backup created, sent to Telegram", body = BackupAndSendResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Device or bot not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Backup or send failed")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Backup"
)]
pub async fn backup_and_send_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Json(payload): Json<BackupAndSendRequest>,
) -> Result<(StatusCode, Json<BackupAndSendResponse>), AppError> {
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let format = payload.format.unwrap_or(crate::dto::mikrotik::BackupFormat::Backup);
    let delete_after = payload.delete_after_send.unwrap_or(true);

    let result = MikrotikService::backup_and_send(
        &state.db,
        &state.mikrotik_pool,
        id,
        aes_key,
        Some(user_ctx.user_id),
        payload.name.as_deref(),
        payload.password.as_deref(),
        &format,
        payload.telegram_bot_id,
        delete_after,
    ).await?;

    let status = if result.telegram_success {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_BACKUP_SEND",
        "POST",
        &format!("/api/mikrotik_client/{}/backup/send", id),
        status.as_u16() as i32,
        &ip,
        Some(serde_json::json!({
            "device_id": id,
            "filename": result.filename,
            "format": result.format,
            "telegram_bot_id": result.telegram_bot_id,
            "telegram_success": result.telegram_success,
            "deleted_from_device": result.deleted_from_device,
        })),
    ).await;

    Ok((status, Json(result)))
}

/// Download a backup file from the MikroTik device's filesystem.
///
/// Returns the raw binary content of the backup file with appropriate headers.
#[utoipa::path(
    get,
    path = "/api/mikrotik_client/{id}/backup/download",
    responses(
        (status = 200, description = "Raw backup file content (binary)", content_type = "application/octet-stream"),
        (status = 404, description = "Device or file not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = Uuid, Path, description = "MikroTik Device ID"),
        ("filename" = String, Query, description = "Backup filename to download (e.g. device-20260626.backup)")
    ),
    security(("bearer_auth" = [])),
    tag = "MikroTik Backup"
)]
pub async fn download_backup_file_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Query(params): Query<DownloadBackupParams>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), AppError> {
    let ip = extract_ip_from_headers(&headers);
    let aes_key = crate::config::mikrotik::get_aes_key();

    let contents = MikrotikService::download_backup_file(
        &state.db,
        &state.mikrotik_pool,
        id,
        aes_key,
        Some(user_ctx.user_id),
        &params.filename,
    ).await?;

    let _ = AuditService::log(
        &state.db,
        Some(user_ctx.user_id),
        "MIKROTIK_BACKUP_DOWNLOAD",
        "GET",
        &format!("/api/mikrotik_client/{}/backup/download", id),
        200,
        &ip,
        Some(json!({
            "device_id": id,
            "filename": params.filename,
            "size": contents.len(),
        })),
    ).await;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    resp_headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", params.filename))
            .map_err(|_| AppError::InternalServerError("Invalid filename for header".to_string()))?,
    );

    Ok((StatusCode::OK, resp_headers, contents))
}
