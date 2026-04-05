use axum::{routing::get, routing::post, routing::put, routing::delete, Router};
use crate::AppState;
use crate::handlers::mikrotik_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(mikrotik_handler::list_clients))
        .route("/", post(mikrotik_handler::create_client))
        .route("/:id", get(mikrotik_handler::get_client))
        .route("/:id", put(mikrotik_handler::update_client))
        .route("/:id", delete(mikrotik_handler::delete_client))
        .route("/:id/system/resource/print", get(mikrotik_handler::get_system_resource))
}
