use axum::{extract::State, Json};
use crate::dto::auth::{RegisterRequest, LoginRequest, AuthResponse, ForgotPasswordRequest, ResetPasswordRequest, VerifyTokenResponse};
use crate::services::auth_service::AuthService;
use crate::services::audit::AuditService;
use crate::middlewares::auth::UserContext;
use crate::AppState;
use crate::errors::app_error::AppError;
use validator::Validate;

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
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Super Admin required)")
    ),
    security(("bearer_auth" = []))
)]
pub async fn register(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let ip = extract_ip(&headers);
    let email = req.email.clone();

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_REGISTER_FAILED", "POST", "/api/auth/register", 403, &ip,
            Some(serde_json::json!({"email": email, "error": "Forbidden (Super Admin required)"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required to register new users".to_string()));
    }

    if let Err(e) = req.validate() {
        return Err(AppError::BadRequest(e.to_string()));
    }

    match AuthService::register(&state.db, &state.rabbit, req).await {
        Ok(res) => {
            let _ = AuditService::log(
                &state.db, Some(user_ctx.user_id),
                "USER_REGISTER_SUCCESS", "POST", "/api/auth/register", 201, &ip,
                Some(serde_json::json!({"registered_email": email})),
            ).await;
            Ok(Json(res))
        }
        Err(e) => {
            let _ = AuditService::log(
                &state.db, Some(user_ctx.user_id),
                "USER_REGISTER_FAILED", "POST", "/api/auth/register", 400, &ip,
                Some(serde_json::json!({"email": email, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let ip = extract_ip(&headers);
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let email = req.email.clone();

    match AuthService::login(&state.db, req).await {
        Ok(res) => {
            let user_id = uuid::Uuid::parse_str(&res.user_id).ok();
            let _ = AuditService::log(
                &state.db, user_id,
                "USER_LOGIN_SUCCESS", "POST", "/api/auth/login", 200, &ip,
                Some(serde_json::json!({"email": email})),
            ).await;
            Ok(Json(res))
        }
        Err(e) => {
            let _ = AuditService::log(
                &state.db, None,
                "USER_LOGIN_FAILED", "POST", "/api/auth/login", 401, &ip,
                Some(serde_json::json!({"email": email, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/verify-token",
    responses(
        (status = 200, description = "Token verification status", body = VerifyTokenResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Super Admin required)")
    ),
    security(("bearer_auth" = []))
)]
pub async fn verify_token_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
) -> Result<Json<VerifyTokenResponse>, AppError> {
    let ip = extract_ip(&headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "VERIFY_TOKEN_FAILED", "POST", "/api/auth/verify-token", 403, &ip,
            Some(serde_json::json!({"error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "VERIFY_TOKEN_SUCCESS", "POST", "/api/auth/verify-token", 200, &ip,
        None,
    ).await;

    Ok(Json(VerifyTokenResponse {
        valid: true,
        user_id: Some(user_ctx.user_id.to_string()),
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/{id}/verify-email",
    params(
        ("id" = Uuid, Path, description = "User ID to verify email for")
    ),
    responses(
        (status = 200, description = "Email verified successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Super Admin required)"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn verify_email_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    user_ctx: UserContext,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let ip = extract_ip(&headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_VERIFY_EMAIL_FAILED", "POST", "/api/auth/verify-email", 403, &ip,
            Some(serde_json::json!({"target_user_id": user_id, "error": "Super Admin role required"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required".to_string()));
    }

    AuthService::verify_email(&state.db, user_id).await?;

    let _ = AuditService::log(
        &state.db, Some(user_ctx.user_id),
        "USER_EMAIL_VERIFIED", "POST", "/api/auth/verify-email", 200, &ip,
        Some(serde_json::json!({"verified_user_id": user_id})),
    ).await;

    Ok(axum::http::StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset link sent if email exists")
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let ip = extract_ip(&headers);
    let email = req.email.clone();

    match AuthService::forgot_password(&state.db, &state.redis, &state.rabbit, req).await {
        Ok(_) => {
            let _ = AuditService::log(
                &state.db, None,
                "FORGOT_PASSWORD_REQUESTED", "POST", "/api/auth/forgot-password", 200, &ip,
                Some(serde_json::json!({"email": email})),
            ).await;
            Ok(axum::http::StatusCode::OK)
        }
        Err(e) => {
            let _ = AuditService::log(
                &state.db, None,
                "FORGOT_PASSWORD_FAILED", "POST", "/api/auth/forgot-password", 500, &ip,
                Some(serde_json::json!({"email": email, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful"),
        (status = 400, description = "Invalid token")
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let ip = extract_ip(&headers);
    let token_prefix = if req.token.len() > 8 { req.token[..8].to_string() } else { req.token.clone() };

    match AuthService::reset_password(&state.db, &state.redis, req).await {
        Ok(_) => {
            let _ = AuditService::log(
                &state.db, None,
                "PASSWORD_RESET_SUCCESS", "POST", "/api/auth/reset-password", 200, &ip,
                Some(serde_json::json!({"token_prefix": token_prefix})),
            ).await;
            Ok(axum::http::StatusCode::OK)
        }
        Err(e) => {
            let _ = AuditService::log(
                &state.db, None,
                "PASSWORD_RESET_FAILED", "POST", "/api/auth/reset-password", 400, &ip,
                Some(serde_json::json!({"token_prefix": token_prefix, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}
