mod health;
mod auth_routes;
mod user_routes;
mod export_routes;

use axum::{Router, middleware};
use crate::AppState;
use crate::middlewares::{rate_limit_middleware, login_attempts_middleware};
use tower_http::cors::CorsLayer;
use axum::http::{HeaderValue, Method};

pub fn create_router(state: AppState) -> Router {
    // ────────────────────────────────────────────────────
    // CORS Configuration
    // Ubah allowed_origins di bawah ini jika domain production berubah.
    // Contoh: "https://billing.mnet.id"
    // ────────────────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),   // Vite dev server
            "http://localhost:8080".parse::<HeaderValue>().unwrap(),   // Alt dev server
            "https://billing.example.com".parse::<HeaderValue>().unwrap(), // Production (ganti ini)
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ])
        .allow_credentials(true);

    Router::new()
        .nest("/api/health", health::routes())
        .nest("/api/auth", auth_routes::routes())
        .nest("/api/users", user_routes::routes())
        .nest("/api/export", export_routes::routes())
        .layer(cors)
        .layer(middleware::from_fn_with_state(state.clone(), login_attempts_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .with_state(state)
}
