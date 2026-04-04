use lapin::{options::*, types::FieldTable, BasicProperties};
use crate::errors::app_error::AppError;

#[derive(Clone)]
pub struct RabbitMQClient {
    connection: std::sync::Arc<lapin::Connection>,
}

impl RabbitMQClient {
    pub fn new(connection: std::sync::Arc<lapin::Connection>) -> Self {
        Self { connection }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.status().connected()
    }

    pub async fn publish(&self, queue_name: &str, payload: &str) -> Result<(), AppError> {
        let channel = self.connection.create_channel().await
            .map_err(|e| AppError::InternalServerError(format!("RabbitMQ channel creation failed: {}", e)))?;

        channel.queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await
        .map_err(|e| AppError::InternalServerError(format!("RabbitMQ queue declaration failed: {}", e)))?;

        channel.basic_publish(
            "",
            queue_name,
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default(),
        ).await
        .map_err(|e| AppError::InternalServerError(format!("RabbitMQ publish failed: {}", e)))?;

        Ok(())
    }
}
