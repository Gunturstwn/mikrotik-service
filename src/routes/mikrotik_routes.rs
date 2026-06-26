use axum::{routing::get, routing::post, Router};
use crate::AppState;
use crate::handlers::mikrotik_handler;

pub fn routes() -> Router<AppState> {
    // Backup logs endpoint ditempatkan di root dulu agar tidak conflict dengan /:id
    Router::new()
        .route("/backup-logs", get(mikrotik_handler::list_backup_logs_handler))
        .route("/", get(mikrotik_handler::list_clients).post(mikrotik_handler::create_client))
        .route("/:id", get(mikrotik_handler::get_client).put(mikrotik_handler::update_client).delete(mikrotik_handler::delete_client))
        .route("/:id/test-connection", get(mikrotik_handler::test_connection_handler))
        .route("/:id/system/resource/print", get(mikrotik_handler::get_system_resource))
        .route("/:id/interfaces/print", get(mikrotik_handler::get_interfaces))
        .route("/:id/interfaces/monitor", get(mikrotik_handler::monitor_interfaces))
        .route("/:id/interfaces/torch", get(mikrotik_handler::get_torch))
        .route("/:id/config/history", get(mikrotik_handler::get_config_history))
        .route("/:id/config/view/:snapshot_id", get(mikrotik_handler::view_config_snapshot))
        .route("/:id/config/backup-now", post(mikrotik_handler::backup_now))
        .route("/:id/config/diff", get(mikrotik_handler::get_config_diff))
        // ── Binary Backup ──────────────────────────────────────────────
        .route("/:id/backup", post(mikrotik_handler::trigger_backup_handler))
        .route("/:id/backup/files", get(mikrotik_handler::list_backup_files_handler))
        .route("/:id/backup/download", get(mikrotik_handler::download_backup_file_handler))
        .route("/:id/backup/send", post(mikrotik_handler::backup_and_send_handler))
}
