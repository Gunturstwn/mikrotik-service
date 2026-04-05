use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use utoipa::ToSchema;
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

    #[error("Too many requests: {0}")]
    TooManyRequests(String),
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Technical error identifier (e.g. AUTH_FAILED, RATE_LIMIT_EXCEEDED)
    #[schema(example = "UNAUTHORIZED")]
    pub error: String,
    /// Human-readable error message
    #[schema(example = "Invalid email or password")]
    pub message: String,
    /// HTTP Status Code
    #[schema(example = 401)]
    pub code: u16,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            AppError::RedisError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "REDIS_ERROR"),
            AppError::RabbitMQError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "RABBITMQ_ERROR"),
            AppError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::InternalServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::TooManyRequests(_) => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED"),
        };

        let message = self.to_string();

        let body = Json(ErrorResponse {
            error: error.to_string(),
            message,
            code: status.as_u16(),
        });

        (status, body).into_response()
    }
}
