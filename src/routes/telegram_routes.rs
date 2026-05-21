use axum::{routing::{get, post, put, delete}, Router};
use crate::AppState;
use crate::handlers::telegram_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(telegram_handler::list_bots))
        .route("/", post(telegram_handler::create_bot))
        .route("/:id", get(telegram_handler::get_bot))
        .route("/:id", put(telegram_handler::update_bot))
        .route("/:id", delete(telegram_handler::delete_bot))
        .route("/:id/test", post(telegram_handler::test_bot))
}
