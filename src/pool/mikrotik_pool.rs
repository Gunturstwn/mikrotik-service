use dashmap::DashMap;
use mikrotik_rs::MikrotikDevice;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Instant, Duration};
use uuid::Uuid;
use crate::models::mikrotik_clients::Entity as MikrotikClient;
use crate::errors::app_error::AppError;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use tracing::{info, error, warn};
use crate::services::audit::audit_service::AuditService;
use serde_json::json;

pub struct MikrotikPoolEntry {
    pub device: Arc<Mutex<MikrotikDevice>>,
    pub last_used: Instant,
    pub is_used: bool,
    pub device_id: Uuid,
}

pub struct MikrotikPool {
    inner: DashMap<Uuid, MikrotikPoolEntry>,
    ttl: Duration,
}

impl MikrotikPool {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            inner: DashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Retrieves an existing connection or creates a new one lazily.
    pub async fn get_connection(
        &self,
        device_id: Uuid,
        db: &DatabaseConnection,
        aes_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<Arc<Mutex<MikrotikDevice>>, AppError> {
        let start_time = Instant::now();

        // 1. Try to get from cache first
        if let Some(mut entry) = self.inner.get_mut(&device_id) {
            if entry.last_used.elapsed() < self.ttl {
                entry.last_used = Instant::now();
                entry.is_used = true;
                return Ok(Arc::clone(&entry.device));
            }
        }

        // 2. Not in cache or expired, create new connection
        info!("MikroTik Pool: Connecting to device {} (Lazy Loading)...", device_id);
        
        let client = MikrotikClient::find_by_id(device_id)
            .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("MikroTik client not found or deleted".to_string()))?;

        // Attempt connection and measure duration
        let result = crate::config::mikrotik::MikrotikConnection::connect(&client, aes_key).await;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(device) => {
                info!("CONNECTED_SUCCESS: Device {} connected in {}ms", device_id, duration_ms);
                
                // Log success to audit trail
                let _ = AuditService::log(
                    db,
                    user_id,
                    "MIKROTIK_CONNECTION",
                    "POOL",
                    &format!("/mikrotik/{}", device_id),
                    200,
                    &client.host,
                    Some(json!({
                        "status": "SUCCESS",
                        "device_id": device_id,
                        "duration_ms": duration_ms,
                        "message": "Connection established successfully"
                    })),
                ).await;

                let device_arc = Arc::new(Mutex::new(device));
                self.inner.insert(device_id, MikrotikPoolEntry {
                    device: Arc::clone(&device_arc),
                    last_used: Instant::now(),
                    is_used: true,
                    device_id,
                });

                Ok(device_arc)
            }
            Err(e) => {
                error!("CONNECTED_FAILED: Device {} failed to connect after {}ms. Reason: {}", device_id, duration_ms, e);
                
                // Log failure to audit trail
                let _ = AuditService::log(
                    db,
                    user_id,
                    "MIKROTIK_CONNECTION",
                    "POOL",
                    &format!("/mikrotik/{}", device_id),
                    500,
                    &client.host,
                    Some(json!({
                        "status": "FAILED",
                        "device_id": device_id,
                        "duration_ms": duration_ms,
                        "message": format!("{}", e)
                    })),
                ).await;

                Err(e)
            }
        }
    }

    /// Background task to clean up expired connections.
    pub fn start_cleanup_task(self: Arc<Self>, db: DatabaseConnection) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut to_remove = Vec::new();

                for entry in self.inner.iter() {
                    if now.duration_since(entry.last_used) > self.ttl {
                        to_remove.push((*entry.key(), entry.is_used));
                    }
                }

                for (id, was_used) in to_remove {
                    if !was_used {
                        warn!("MikroTik Pool: Evicting expired connection for device {} (NEVER USED)", id);
                        let _ = AuditService::log(
                            &db,
                            None,
                            "MIKROTIK_CONNECTION",
                            "POOL_CLEANUP",
                            &format!("/mikrotik/{}", id),
                            410,
                            "0.0.0.0", // IP not available during cleanup
                            Some(json!({
                                "status": "EXPIRED_UNUSED",
                                "device_id": id,
                                "message": "Connection evicted after inactivity without being used"
                            })),
                        ).await;
                    } else {
                        info!("MikroTik Pool: Evicting expired connection for device {}", id);
                    }
                    self.inner.remove(&id);
                }
            }
        });
    }

    /// Inform pool that a device has been updated/deleted, forcing reconnection next time.
    pub fn invalidate(&self, device_id: Uuid) {
        if self.inner.remove(&device_id).is_some() {
            info!("MikroTik Pool: Invalidated connection for device {}", device_id);
        }
    }
}
