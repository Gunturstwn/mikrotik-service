mod health;
mod auth_routes;
mod user_routes;
mod export_routes;
mod mikrotik_routes;

use axum::Router;
use crate::AppState;
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
        .nest("/api/auth", auth_routes::routes(state.clone()))
        .nest("/api/users", user_routes::routes())
        .nest("/api/export", export_routes::routes())
        .nest("/api/mikrotik_client", mikrotik_routes::routes())
        .layer(cors)
        .with_state(state)
}
