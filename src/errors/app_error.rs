use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("RabbitMQ error: {0}")]
    RabbitMQError(#[from] lapin::Error),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::DatabaseError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::RedisError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            AppError::RabbitMQError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::StorageError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            AppError::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
            AppError::Unauthorized(err) => (StatusCode::UNAUTHORIZED, err),
            AppError::Forbidden(err) => (StatusCode::FORBIDDEN, err),
            AppError::InternalServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            AppError::NotFound(err) => (StatusCode::NOT_FOUND, err),
        };

        let body = Json(ErrorResponse {
            status: "error".to_string(),
            message,
        });

        (status, body).into_response()
    }
}
