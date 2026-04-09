use crate::AppState;
use crate::models::mikrotik_clients::{Entity as MikrotikClient, Model as ClientModel};
use crate::models::interface_metrics::{ActiveModel as MetricActiveModel};
use crate::services::mikrotik_service::MikrotikService;
use crate::services::audit::AuditService;
use sea_orm::*;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::Utc;

pub struct MetricsWorker;

impl MetricsWorker {
    pub async fn run(state: AppState) {
        let interval_secs = std::env::var("METRICS_INTERVAL")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .unwrap_or(60);
        
        let interval = Duration::from_secs(interval_secs);
        
        tracing::info!("MetricsWorker: Monitoring started with interval {}s", interval_secs);

        loop {
            let aes_key = std::env::var("AES_KEY").unwrap_or_default();
            if aes_key.is_empty() {
                tracing::error!("MetricsWorker: AES_KEY not set, worker sleeping for 30s...");
                sleep(Duration::from_secs(30)).await;
                continue;
            }

            match MikrotikClient::find()
                .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
                .all(&state.db)
                .await 
            {
                Ok(clients) => {
                    for client in clients {
                        let state_clone = state.clone();
                        let aes_key_clone = aes_key.clone();
                        
                        tokio::spawn(async move {
                            let client_id = client.id;
                            if let Err(e) = Self::scrape_client(&state_clone, client, &aes_key_clone).await {
                                tracing::error!("MetricsWorker: Error scraping client {}: {}", client_id, e);
                            }
                        });
                    }
                }
                Err(e) => tracing::error!("MetricsWorker: Error fetching clients from DB: {}", e),
            }

            sleep(interval).await;
        }
    }

    async fn scrape_client(state: &AppState, client: ClientModel, aes_key: &str) -> Result<(), crate::errors::app_error::AppError> {
        let system_user_id = Uuid::nil();
        
        let interfaces = MikrotikService::get_interfaces(
            &state.db,
            &state.mikrotik_pool,
            client.id,
            aes_key,
            Some(system_user_id)
        ).await?;

        // Audit log for the automated scrape
        let _ = AuditService::log(
            &state.db,
            Some(system_user_id),
            "MIKROTIK_METRICS_SCRAPE",
            "SYSTEM",
            "/background/metrics_worker",
            200,
            "127.0.0.1",
            Some(serde_json::json!({ "device_id": client.id, "interface_count": interfaces.len() })),
        ).await;

        for iface in interfaces {
            let iface_name = iface.name.clone();
            let metric = MetricActiveModel {
                id: Set(Uuid::new_v4()),
                mikrotik_id: Set(client.id),
                interface_name: Set(iface.name),
                rx_byte: Set(iface.rx_byte.unwrap_or(0) as i64),
                tx_byte: Set(iface.tx_byte.unwrap_or(0) as i64),
                rx_packet: Set(iface.rx_packet.unwrap_or(0) as i64),
                tx_packet: Set(iface.tx_packet.unwrap_or(0) as i64),
                captured_at: Set(Utc::now().naive_utc()),
            };

            if let Err(e) = metric.insert(&state.db).await {
                tracing::error!("MetricsWorker: DB insert error for interface {}: {}", iface_name, e);
            }
        }

        Ok(())
    }
}
