use lapin::{options::*, types::FieldTable};
use futures_util::StreamExt;
use lettre::{Message, SmtpTransport, Transport};
use crate::errors::app_error::AppError;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub struct EmailWorker {
    connection: std::sync::Arc<lapin::Connection>,
    smtp_transport: SmtpTransport,
}

impl EmailWorker {
    pub fn new(connection: std::sync::Arc<lapin::Connection>, smtp_transport: SmtpTransport) -> Self {
        Self { 
            connection, 
            smtp_transport 
        }
    }

    pub async fn run(&self) -> Result<(), AppError> {
        let channel = self.connection.create_channel().await
            .map_err(|e| AppError::InternalServerError(format!("RabbitMQ channel creation failed: {}", e)))?;

        channel.queue_declare(
            "email_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await
        .map_err(|e| AppError::InternalServerError(format!("RabbitMQ queue declaration failed: {}", e)))?;

        let mut consumer = channel.basic_consume(
            "email_queue",
            "email_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        ).await
        .map_err(|e| AppError::InternalServerError(format!("RabbitMQ consume failed: {}", e)))?;

        info!("EmailWorker: Started consuming 'email_queue'...");

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let payload = String::from_utf8_lossy(&delivery.data);
                    match serde_json::from_str::<EmailJob>(&payload) {
                        Ok(job) => {
                            self.send_email(job).await;
                        }
                        Err(e) => {
                            error!("EmailWorker: Failed to parse job JSON: {}", e);
                        }
                    }
                    let _ack_res: Result<(), lapin::Error> = delivery.ack(BasicAckOptions::default()).await;
                }
                Err(e) => {
                    error!("EmailWorker: Consumer error: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn send_email(&self, job: EmailJob) {
        let from_addr = std::env::var("SMTP_USER")
            .unwrap_or_else(|_| "noreply@mikrotik-billing.com".to_string());

        let from_header = format!("MikroTik Billing <{}>", from_addr);

        let from_header_parsed = match from_header.parse() {
            Ok(a) => a,
            Err(e) => {
                error!("EmailWorker: Failed to parse from-address '{}': {}", from_header, e);
                return;
            }
        };

        let to_addr_parsed = match job.to.parse() {
            Ok(a) => a,
            Err(e) => {
                error!("EmailWorker: Failed to parse to-address '{}': {}", job.to, e);
                return;
            }
        };

        let email = match Message::builder()
            .from(from_header_parsed)
            .to(to_addr_parsed)
            .subject(job.subject)
            .singlepart(
                lettre::message::SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_PLAIN)
                    .body(job.body),
            ) {
                Ok(e) => e,
                Err(e) => {
                    error!("EmailWorker: Failed to build email to {}: {}", job.to, e);
                    return;
                }
            };


        match self.smtp_transport.send(&email) {
            Ok(_) => info!("EmailWorker: Successfully sent email to {}", job.to),
            Err(e) => error!("EmailWorker: Failed to send email to {}: {}", job.to, e),
        }
    }
}
