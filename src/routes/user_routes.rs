use axum::{routing::get, routing::put, routing::delete, Router};
use crate::AppState;
use crate::handlers::user_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(user_handler::get_users))
        .route("/me", get(user_handler::get_me))
        .route("/me", put(user_handler::update_me))
        .route("/me/photo", axum::routing::post(user_handler::upload_photo))
        .route("/:id", get(user_handler::get_user))
        .route("/:id", delete(user_handler::delete_user))
}
