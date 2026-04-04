pub mod config;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;
pub mod cache;
pub mod queue;
pub mod workers;
pub mod export;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis: cache::RedisClient,
    pub rabbit: queue::RabbitMQClient,
    pub storage: std::sync::Arc<aws_sdk_s3::Client>,
}

impl AppState {
    pub fn new(
        db: sea_orm::DatabaseConnection,
        redis: cache::RedisClient,
        rabbit: queue::RabbitMQClient,
        storage: aws_sdk_s3::Client,
    ) -> Self {
        Self {
            db,
            redis,
            rabbit,
            storage: std::sync::Arc::new(storage),
        }
    }
}
