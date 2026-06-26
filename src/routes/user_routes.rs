use axum::{routing::{get, post, put}, Router};
use crate::AppState;
use crate::handlers::user_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Bug fix #2: Gabungkan method routing pada path yang sama
        // Menghindari potensi konflik /me (statis) vs /:id (dinamis) di Axum 0.7
        .route("/", get(user_handler::get_users))
        .route("/me", get(user_handler::get_me).put(user_handler::update_me))
        .route("/me/password", put(user_handler::change_password))
        .route("/me/photo", post(user_handler::upload_photo))
        .route("/:id", get(user_handler::get_user)
            .put(user_handler::update_user)
            .delete(user_handler::delete_user))
}
