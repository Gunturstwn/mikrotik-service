use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::AppState;
use crate::errors::app_error::AppError;

pub async fn global_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_ip(&request);
    let key = format!("rate:global:{}", ip);

    // 10 req/s, 20 burst
    if state.redis.check_rate_limit(&key, 10.0, 20.0).await? {
        Ok(next.run(request).await)
    } else {
        Err(AppError::TooManyRequests("Global rate limit exceeded. Please slow down.".to_string()))
    }
}

pub async fn login_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_ip(&request);
    let key = format!("rate:login:{}", ip);

    // 2 req/s, 5 burst
    if state.redis.check_rate_limit(&key, 2.0, 5.0).await? {
        Ok(next.run(request).await)
    } else {
        Err(AppError::TooManyRequests("Login rate limit exceeded. Please wait a moment.".to_string()))
    }
}

fn extract_ip(req: &Request) -> &str {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim())
        .or_else(|| req.headers().get("x-real-ip").and_then(|h| h.to_str().ok()))
        .or_else(|| req.headers().get("host").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown")
}

