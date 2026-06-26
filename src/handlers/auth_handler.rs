use axum::{extract::{State, Query}, Json, http::StatusCode};
use crate::dto::auth::{RegisterRequest, LoginRequest, AuthResponse, ForgotPasswordRequest, ResetPasswordRequest, VerifyTokenResponse, LoginStatusResponse};
use crate::services::auth_service::AuthService;
use crate::services::audit::AuditService;
use crate::middlewares::auth::UserContext;
use crate::AppState;
use crate::errors::app_error::AppError;
use crate::utils::ip::extract_ip_from_headers;
use crate::utils::multipart::parse_register_multipart;
use validator::Validate;


#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body(content = RegisterRequest, description = "Register a new user with optional photo upload", content_type = "multipart/form-data"),
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
    multipart: axum::extract::Multipart,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let ip = extract_ip_from_headers(&headers);

    if !user_ctx.roles.contains(&"Super Admin".to_string()) {
        let _ = AuditService::log(
            &state.db, Some(user_ctx.user_id),
            "USER_REGISTER_FAILED", "POST", "/api/auth/register", 403, &ip,
            Some(serde_json::json!({"error": "Forbidden (Super Admin required)"})),
        ).await;
        return Err(AppError::Forbidden("Super Admin role required to register new users".to_string()));
    }

    // Parse multipart fields using shared utility
    let parsed = parse_register_multipart(multipart, &state.storage).await?;

    let req = RegisterRequest {
        name: parsed.name,
        email: parsed.email.clone(),
        password: parsed.password,
        phone: parsed.phone,
        address: parsed.address,
        photo: parsed.photo_url,
        lat: parsed.lat,
        lng: parsed.lng,
        payment_token: parsed.payment_token,
        role: parsed.role,
    };

    // Bug fix #4: Andalkan req.validate() sepenuhnya — validasi manual duplikat sudah dihapus.
    // RegisterRequest sudah memiliki #[validate] pada name (length 3-50), email (format),
    // dan password (length min 6) via derive(Validate).
    if let Err(e) = req.validate() {
        return Err(AppError::BadRequest(e.to_string()));
    }

    match AuthService::register(&state.db, &state.rabbit, req).await {
        Ok(res) => {
            let _ = AuditService::log(
                &state.db, Some(user_ctx.user_id),
                "USER_REGISTER_SUCCESS", "POST", "/api/auth/register", 201, &ip,
                Some(serde_json::json!({"registered_email": parsed.email})),
            ).await;
            // Bug fix #3: Return 201 Created (bukan 200 OK) sesuai HTTP semantics dan Swagger docs
            Ok((StatusCode::CREATED, Json(res)))
        }
        Err(e) => {
            let _ = AuditService::log(
                &state.db, Some(user_ctx.user_id),
                "USER_REGISTER_FAILED", "POST", "/api/auth/register", 400, &ip,
                Some(serde_json::json!({"email": parsed.email, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}

/// Authenticates a user and returns a JWT Bearer token.
/// 
/// ### Security Behavior Notes:
/// 1. **Atomic Rate Limiting**: Enforces a dual-layer bucket (10 req/s global, 2 req/s login) via Redis-backed Lua scripts.
/// 2. **Brute-Force Protection**: Independent IP and Account failure tracking with exponential backoff.
/// 3. **CAPTCHA Enforcement**: Suspicious activity (3+ failures) triggers mandatory Cloudflare Turnstile verification.
/// 4. **Lockout Policy**: 10, 20, or 30 consecutive failures result in 15m, 1h, or 6h access suspension respectively.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authentication successful", body = AuthResponse),
        (status = 400, description = "Invalid request or CAPTCHA required", body = ErrorResponse),
        (status = 401, description = "Authentication failed - Invalid credentials", body = ErrorResponse),
        (status = 403, description = "Access denied - Account/IP temporarily locked", body = ErrorResponse),
        (status = 429, description = "Resource exhausted - Rate limit exceeded", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let ip = extract_ip_from_headers(&headers);
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let email = req.email.clone();

    match AuthService::login(&state.db, &state.security, &state.captcha, &ip, req).await {
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
            let status = e.status_code();
            let _ = AuditService::log(
                &state.db, None,
                "USER_LOGIN_FAILED", "POST", "/api/auth/login", status.as_u16() as i32, &ip,
                Some(serde_json::json!({"email": email, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}

#[derive(Debug, serde::Deserialize, validator::Validate)]
pub struct LoginStatusQuery {
    #[validate(email)]
    pub email: String,
}

/// Checks the security status of an email/IP (CAPTCHA or Blocked).
#[utoipa::path(
    get,
    path = "/api/auth/login-status",
    params(
        ("email" = String, Query, description = "Email to check status for")
    ),
    responses(
        (status = 200, description = "Status retrieved", body = LoginStatusResponse),
        (status = 400, description = "Invalid email format")
    ),
    tag = "Authentication"
)]
pub async fn get_login_status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<LoginStatusQuery>,
) -> Result<Json<LoginStatusResponse>, AppError> {
    let ip = extract_ip_from_headers(&headers);
    
    // If email validation fails, just return Allowed instead of 400 error.
    // This provides better UX while the user is still typing.
    if query.validate().is_err() {
        return Ok(Json(LoginStatusResponse {
            captcha_required: false,
            blocked_until: None,
        }));
    }
    
    let res = AuthService::check_login_status(&state.security, &ip, &query.email).await?;
    Ok(Json(res))
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
    let ip = extract_ip_from_headers(&headers);

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
    let ip = extract_ip_from_headers(&headers);

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
    let ip = extract_ip_from_headers(&headers);
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
    let ip = extract_ip_from_headers(&headers);
    let token_prefix = if req.token.len() > 8 { req.token[..8].to_string() } else { req.token.clone() };
    let email = req.email.clone();

    match AuthService::reset_password(&state.db, &state.redis, req).await {
        Ok(_) => {
            let _ = AuditService::log(
                &state.db, None,
                "PASSWORD_RESET_SUCCESS", "POST", "/api/auth/reset-password", 200, &ip,
                Some(serde_json::json!({"email": email, "token_prefix": token_prefix})),
            ).await;
            Ok(axum::http::StatusCode::OK)
        }
        Err(e) => {
            let _ = AuditService::log(
                &state.db, None,
                "PASSWORD_RESET_FAILED", "POST", "/api/auth/reset-password", 400, &ip,
                Some(serde_json::json!({"email": email, "token_prefix": token_prefix, "error": e.to_string()})),
            ).await;
            Err(e)
        }
    }
}
