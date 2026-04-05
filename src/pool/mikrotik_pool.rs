use dashmap::DashMap;
use mikrotik_rs::MikrotikDevice;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Instant, Duration};
use uuid::Uuid;
use crate::models::mikrotik_clients::Entity as MikrotikClient;
use crate::errors::app_error::AppError;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use tracing::info;

pub struct MikrotikPoolEntry {
    pub device: Arc<Mutex<MikrotikDevice>>,
    pub last_used: Instant,
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
    ) -> Result<Arc<Mutex<MikrotikDevice>>, AppError> {
        // 1. Try to get from cache first
        if let Some(mut entry) = self.inner.get_mut(&device_id) {
            if entry.last_used.elapsed() < self.ttl {
                entry.last_used = Instant::now();
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

        let device = crate::config::mikrotik::MikrotikConnection::connect(&client, aes_key).await?;
        let device_arc = Arc::new(Mutex::new(device));

        self.inner.insert(device_id, MikrotikPoolEntry {
            device: Arc::clone(&device_arc),
            last_used: Instant::now(),
        });

        Ok(device_arc)
    }

    /// Background task to clean up expired connections.
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut to_remove = Vec::new();

                for entry in self.inner.iter() {
                    if now.duration_since(entry.last_used) > self.ttl {
                        to_remove.push(*entry.key());
                    }
                }

                for id in to_remove {
                    info!("MikroTik Pool: Evicting expired connection for device {}", id);
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
