use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::AppState;
use crate::errors::app_error::AppError;
use axum::http::StatusCode;

pub async fn login_attempts_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Only apply to /api/auth/login
    if req.uri().path() != "/api/auth/login" {
        return Ok(next.run(req).await);
    }

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| req.headers().get("host").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown");

    let cache_key = format!("login_attempts:{}", ip);
    let attempts = state.redis.get(&cache_key).await?;

    if let Some(count_str) = attempts {
        let count: i32 = count_str.parse().unwrap_or(0);
        if count >= 10 {
            return Err(AppError::Forbidden("Too many login attempts. Please wait 15 minutes.".to_string()));
        }
    }

    let response = next.run(req).await;

    if response.status() == StatusCode::UNAUTHORIZED {
        state.redis.incr(&cache_key, 900).await?; // 900s = 15m
    } else if response.status().is_success() {
        state.redis.del(&cache_key).await?;
    }

    Ok(response)
}
