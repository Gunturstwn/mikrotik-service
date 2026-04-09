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
        .route("/:id/test-connection", get(mikrotik_handler::test_connection_handler))
        .route("/:id/system/resource/print", get(mikrotik_handler::get_system_resource))
        .route("/:id/interfaces/print", get(mikrotik_handler::get_interfaces))
        .route("/:id/interfaces/monitor", get(mikrotik_handler::monitor_interfaces))
        .route("/:id/interfaces/torch", get(mikrotik_handler::get_torch))
        .route("/:id/config/history", get(mikrotik_handler::get_config_history))
        .route("/:id/config/view/:snapshot_id", get(mikrotik_handler::view_config_snapshot))
        .route("/:id/config/backup-now", post(mikrotik_handler::backup_now))
        .route("/:id/config/diff", get(mikrotik_handler::get_config_diff))
}
