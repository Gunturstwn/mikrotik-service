use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::NotKeyed};
use once_cell::sync::Lazy;
use std::num::NonZeroU32;
use std::sync::Arc;

type IpRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

static LIMITER: Lazy<Arc<IpRateLimiter>> = Lazy::new(|| {
    Arc::new(RateLimiter::direct(
        Quota::per_second(NonZeroU32::new(10).unwrap()).allow_burst(NonZeroU32::new(20).unwrap()),
    ))
});

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if LIMITER.check().is_ok() {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
