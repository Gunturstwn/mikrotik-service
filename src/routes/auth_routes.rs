use axum::{routing::{get, post}, Router};
use crate::AppState;
use crate::handlers::auth_handler;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login).layer(
            axum::middleware::from_fn_with_state(state.clone(), crate::middlewares::login_rate_limit_middleware)
        ))
        .route("/verify-token", post(auth_handler::verify_token_handler))
        .route("/:id/verify-email", post(auth_handler::verify_email_handler))
        .route("/forgot-password", post(auth_handler::forgot_password))
        .route("/reset-password", post(auth_handler::reset_password))
        .route("/login-status", get(auth_handler::get_login_status))
}
