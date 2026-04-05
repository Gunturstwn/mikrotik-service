use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::NaiveDateTime;
use sea_orm::prelude::Decimal;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikClientRequest {
    /// Friendly name for the MikroTik device
    #[schema(example = "Core Router HQ")]
    pub name_device: String,
    /// Hostname or IP address of the device
    #[schema(example = "192.168.1.1")]
    pub host: String,
    /// RouterOS username (will be encrypted at rest)
    #[schema(example = "admin")]
    pub username: String,
    /// RouterOS password (will be encrypted at rest)
    #[schema(example = "p@ssw0rd123")]
    pub password: String,
    /// Optional Winbox port (encrypted at rest)
    #[schema(example = "8291")]
    pub port_winbox: Option<String>,
    /// Optional API port (encrypted at rest)
    #[schema(example = "8728")]
    pub port_api: Option<String>,
    /// Optional FTP port (encrypted at rest)
    #[schema(example = "21")]
    pub port_ftp: Option<String>,
    /// Optional SSH port
    #[schema(example = 22)]
    pub port_ssh: Option<i32>,
    /// Physical location description
    #[schema(example = "Jakarta Data Center, Rack A1")]
    pub location: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikClientResponse {
    pub id: Uuid,
    pub name_device: String,
    pub host: String,
    /// Note: Encrypted in DB, masked in response for security
    #[schema(example = "********")]
    pub username: String,
    pub port_ssh: Option<i32>,
    pub location: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub timezone: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    /// UUID of the user who registered the device
    pub created_by: Uuid,
    /// UUID of the user who last updated the device
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikResourceResponse {
    #[schema(example = "2d 3h 12m")]
    pub uptime: String,
    #[schema(example = 12)]
    pub cpu_load: i32,
    #[schema(example = 256000000)]
    pub free_memory: i64,
    #[schema(example = 512000000)]
    pub total_memory: i64,
    #[schema(example = 1000000000)]
    pub free_hdd_space: i64,
    #[schema(example = 2000000000)]
    pub total_hdd_space: i64,
}
