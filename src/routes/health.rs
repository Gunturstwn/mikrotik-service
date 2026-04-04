use axum::{routing::get, Router};
use crate::AppState;
use crate::handlers::health_handler;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health_handler::health_check))
}
