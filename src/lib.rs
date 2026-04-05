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
pub mod pool;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis: cache::RedisClient,
    pub rabbit: queue::RabbitMQClient,
    pub storage: std::sync::Arc<aws_sdk_s3::Client>,
    pub security: std::sync::Arc<services::security_service::SecurityService>,
    pub captcha: std::sync::Arc<services::captcha_service::CaptchaService>,
    pub mikrotik_pool: std::sync::Arc<pool::MikrotikPool>,
}

impl AppState {
    pub fn new(
        db: sea_orm::DatabaseConnection,
        redis: cache::RedisClient,
        rabbit: queue::RabbitMQClient,
        storage: aws_sdk_s3::Client,
        security: services::security_service::SecurityService,
        captcha: services::captcha_service::CaptchaService,
        mikrotik_pool: std::sync::Arc<pool::MikrotikPool>,
    ) -> Self {
        Self {
            db,
            redis,
            rabbit,
            storage: std::sync::Arc::new(storage),
            security: std::sync::Arc::new(security),
            captcha: std::sync::Arc::new(captcha),
            mikrotik_pool,
        }
    }
}
