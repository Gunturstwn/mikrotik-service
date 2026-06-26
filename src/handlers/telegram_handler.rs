use axum::{extract::{State, Path}, Json, http::StatusCode};
use crate::dto::telegram::{CreateTelegramBotRequest, UpdateTelegramBotRequest, TelegramBotResponse, TelegramTestResponse};
use crate::services::telegram_service::TelegramService;
use crate::services::audit::AuditService;
use crate::middlewares::auth::UserContext;
use crate::AppState;
use crate::errors::app_error::AppError;
use crate::utils::ip::extract_ip_from_headers;
use uuid::Uuid;

/// Macro-like check for Super Admin role
fn require_super_admin(roles: &[String]) -> Result<(), AppError> {
    if !roles.contains(&"Super Admin".to_string()) {
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }
    Ok(())
}

/// GET /api/telegram — List all telegram bots
#[utoipa::path(
    get,
    path = "/api/telegram",
    responses(
        (status = 200, description = "List of telegram bots", body = Vec<TelegramBotResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Super Admin required)")
    ),
    security(("bearer_auth" = [])),
    tag = "Telegram"
)]
pub async fn list_bots(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
) -> Result<Json<Vec<TelegramBotResponse>>, AppError> {
    require_super_admin(&user_ctx.roles)?;
    let ip = extract_ip_from_headers(&headers);

    let aes_key = crate::config::mikrotik::get_aes_key();
    let bots = TelegramService::list(&state.db, aes_key).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "TELEGRAM_BOT_LIST", "GET", "/api/telegram", 200, &ip, None,
    ).await;

    Ok(Json(bots))
}

/// GET /api/telegram/:id — Get single bot
#[utoipa::path(
    get,
    path = "/api/telegram/{id}",
    responses(
        (status = 200, description = "Telegram bot detail", body = TelegramBotResponse),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    params(("id" = Uuid, Path, description = "Bot ID")),
    security(("bearer_auth" = [])),
    tag = "Telegram"
)]
pub async fn get_bot(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<TelegramBotResponse>, AppError> {
    require_super_admin(&user_ctx.roles)?;
    let ip = extract_ip_from_headers(&headers);

    let aes_key = crate::config::mikrotik::get_aes_key();
    let bot = TelegramService::get_by_id(&state.db, id, aes_key).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "TELEGRAM_BOT_READ", "GET", &format!("/api/telegram/{}", id), 200, &ip, None,
    ).await;

    Ok(Json(bot))
}

/// POST /api/telegram — Create new bot
#[utoipa::path(
    post,
    path = "/api/telegram",
    request_body = CreateTelegramBotRequest,
    responses(
        (status = 201, description = "Bot created", body = TelegramBotResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = [])),
    tag = "Telegram"
)]
pub async fn create_bot(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Json(req): Json<CreateTelegramBotRequest>,
) -> Result<(StatusCode, Json<TelegramBotResponse>), AppError> {
    require_super_admin(&user_ctx.roles)?;
    let ip = extract_ip_from_headers(&headers);

    if req.name.is_empty() || req.token.is_empty() || req.chat_id.is_empty() {
        return Err(AppError::BadRequest("name, token, dan chat_id wajib diisi".to_string()));
    }

    let aes_key = crate::config::mikrotik::get_aes_key();
    let bot = TelegramService::create(&state.db, user_ctx.user_id, req, aes_key).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "TELEGRAM_BOT_CREATED", "POST", "/api/telegram", 201, &ip,
        Some(serde_json::json!({"bot_id": bot.id, "name": bot.name})),
    ).await;

    Ok((StatusCode::CREATED, Json(bot)))
}

/// PUT /api/telegram/:id — Update bot
#[utoipa::path(
    put,
    path = "/api/telegram/{id}",
    request_body = UpdateTelegramBotRequest,
    responses(
        (status = 200, description = "Bot updated", body = TelegramBotResponse),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    params(("id" = Uuid, Path, description = "Bot ID")),
    security(("bearer_auth" = [])),
    tag = "Telegram"
)]
pub async fn update_bot(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTelegramBotRequest>,
) -> Result<Json<TelegramBotResponse>, AppError> {
    require_super_admin(&user_ctx.roles)?;
    let ip = extract_ip_from_headers(&headers);

    let aes_key = crate::config::mikrotik::get_aes_key();
    let bot = TelegramService::update(&state.db, id, user_ctx.user_id, req, aes_key).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "TELEGRAM_BOT_UPDATED", "PUT", &format!("/api/telegram/{}", id), 200, &ip,
        Some(serde_json::json!({"bot_id": id})),
    ).await;

    Ok(Json(bot))
}

/// DELETE /api/telegram/:id — Soft delete bot
#[utoipa::path(
    delete,
    path = "/api/telegram/{id}",
    responses(
        (status = 204, description = "Bot deleted"),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    params(("id" = Uuid, Path, description = "Bot ID")),
    security(("bearer_auth" = [])),
    tag = "Telegram"
)]
pub async fn delete_bot(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    require_super_admin(&user_ctx.roles)?;
    let ip = extract_ip_from_headers(&headers);

    TelegramService::delete(&state.db, id).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "TELEGRAM_BOT_DELETED", "DELETE", &format!("/api/telegram/{}", id), 204, &ip,
        Some(serde_json::json!({"bot_id": id})),
    ).await;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/telegram/:id/test — Test send message
#[utoipa::path(
    post,
    path = "/api/telegram/{id}/test",
    responses(
        (status = 200, description = "Test result", body = TelegramTestResponse),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    params(("id" = Uuid, Path, description = "Bot ID")),
    security(("bearer_auth" = [])),
    tag = "Telegram"
)]
pub async fn test_bot(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Path(id): Path<Uuid>,
) -> Result<Json<TelegramTestResponse>, AppError> {
    require_super_admin(&user_ctx.roles)?;
    let ip = extract_ip_from_headers(&headers);

    let aes_key = crate::config::mikrotik::get_aes_key();
    let result = TelegramService::test_send(&state.db, id, aes_key).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "TELEGRAM_BOT_TEST", "POST", &format!("/api/telegram/{}/test", id), 200, &ip,
        Some(serde_json::json!({"success": result.success})),
    ).await;

    Ok(Json(result))
}
