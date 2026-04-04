use axum::{extract::State, Json};
use crate::AppState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub redis: String,
    pub rabbitmq: String,
    pub storage: String,
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check for infrastructure services", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_status = match state.db.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let redis_status = match state.redis.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let rabbit_status = if state.rabbit.is_connected() {
        "connected"
    } else {
        "disconnected"
    };

    let storage_status = match state.storage.list_buckets().send().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
        redis: redis_status.to_string(),
        rabbitmq: rabbit_status.to_string(),
        storage: storage_status.to_string(),
    })
}
