use lapin::{Connection, ConnectionProperties};
use std::env;

pub async fn connect() -> Connection {
    let rabbitmq_url = env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");
    Connection::connect(&rabbitmq_url, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ")
}
