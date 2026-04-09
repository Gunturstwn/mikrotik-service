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
    /// Optional SSH port (encrypted at rest)
    #[schema(example = "22")]
    pub port_ssh: Option<String>,
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
    pub port_ssh: Option<String>,
    pub port_winbox: Option<String>,
    pub port_api: Option<String>,
    pub port_ftp: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikInterfaceResponse {
    #[schema(example = "ether1")]
    pub name: String,
    #[schema(example = "ether1")]
    pub default_name: Option<String>,
    #[schema(example = "ether")]
    pub type_name: Option<String>,
    #[schema(example = 1500)]
    pub mtu: Option<i32>,
    #[schema(example = 1500)]
    pub actual_mtu: Option<i32>,
    #[schema(example = "00:00:00:00:00:00")]
    pub mac_address: Option<String>,
    #[schema(example = "jan/01/1970 00:00:00")]
    pub last_link_up_time: Option<String>,
    #[schema(example = 0)]
    pub link_downs: Option<i32>,
    #[schema(example = 123456789)]
    pub rx_byte: Option<u64>,
    #[schema(example = 123456789)]
    pub tx_byte: Option<u64>,
    #[schema(example = 123456)]
    pub rx_packet: Option<u64>,
    #[schema(example = 123456)]
    pub tx_packet: Option<u64>,
    #[schema(example = 0)]
    pub rx_error: Option<u64>,
    #[schema(example = 0)]
    pub tx_error: Option<u64>,
    #[schema(example = 0)]
    pub rx_drop: Option<u64>,
    #[schema(example = 0)]
    pub tx_drop: Option<u64>,
    pub running: bool,
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikMonitorResponse {
    pub name: String,
    #[schema(example = 1000000)]
    pub rx_bits_per_second: u64,
    #[schema(example = 500000)]
    pub tx_bits_per_second: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikTorchResponse {
    pub source_address: Option<String>,
    pub destination_address: Option<String>,
    pub protocol: Option<String>,
    pub port: Option<String>,
    #[schema(example = 1000000)]
    pub tx_rate: u64,
    #[schema(example = 500000)]
    pub rx_rate: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikConfigSnapshotResponse {
    pub id: Uuid,
    pub config_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikConfigViewResponse {
    pub id: Uuid,
    pub config_content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikConfigDiffItem {
    pub status: String, // "added", "removed", "unchanged"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikConfigDiffResponse {
    pub diffs: Vec<MikrotikConfigDiffItem>,
}
