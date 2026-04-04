use axum::{routing::get, Router};
use crate::AppState;
use crate::handlers::export_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/csv", get(export_handler::export_users_csv))
        .route("/users/xlsx", get(export_handler::export_users_xlsx))
}
