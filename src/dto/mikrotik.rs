use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::NaiveDateTime;
use sea_orm::prelude::Decimal;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, Clone)]
pub struct MikrotikClientRequest {
    /// Friendly name for the MikroTik device
    #[validate(length(min = 1, max = 100, message = "name_device must be between 1 and 100 characters"))]
    #[schema(example = "Core Router HQ", min_length = 1, max_length = 100)]
    pub name_device: String,
    /// Hostname or IP address of the device
    #[validate(length(min = 1, max = 255, message = "host must be between 1 and 255 characters"))]
    #[schema(example = "192.168.1.1", min_length = 1, max_length = 255)]
    pub host: String,
    /// RouterOS username (will be encrypted at rest)
    #[validate(length(min = 1, max = 100, message = "username must be between 1 and 100 characters"))]
    #[schema(example = "admin", min_length = 1, max_length = 100)]
    pub username: String,
    /// RouterOS password (will be encrypted at rest)
    #[validate(length(min = 1, max = 255, message = "password must be between 1 and 255 characters"))]
    #[schema(example = "p@ssw0rd123", min_length = 1, max_length = 255)]
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
    /// Default Telegram bot ID for notifications/backups
    pub telegram_bot_id: Option<Uuid>,
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
    /// Default Telegram bot ID for this device
    pub telegram_bot_id: Option<Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2026-06-26T12:00:00")]
    pub created_at: NaiveDateTime,
    #[schema(value_type = String, format = "date-time", example = "2026-06-26T12:00:00")]
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
    #[schema(value_type = String, format = "date-time", example = "2026-06-26T12:00:00")]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MikrotikConfigViewResponse {
    pub id: Uuid,
    pub config_content: String,
    #[schema(value_type = String, format = "date-time", example = "2026-06-26T12:00:00")]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum BackupFormat {
    #[serde(rename = "backup")]
    Backup,
    #[serde(rename = "rsc")]
    Rsc,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct BackupCreateRequest {
    /// Backup filename (without extension, default: [identity]-[date]-[time])
    #[schema(example = "pre-upgrade-backup", max_length = 200)]
    pub name: Option<String>,
    /// Password for encrypted backup (only for .backup format)
    #[schema(example = "backup-pass-123")]
    pub password: Option<String>,
    /// Backup format: "backup" (binary .backup) or "rsc" (text export .rsc)
    #[schema(example = "backup")]
    pub format: Option<BackupFormat>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct BackupFileResponse {
    pub name: String,
    pub size: i64,
    #[schema(value_type = String, format = "date-time", example = "2026-06-26T12:00:00")]
    pub creation_time: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct BackupAndSendRequest {
    /// Backup filename (without extension)
    #[schema(example = "pre-upgrade-backup", max_length = 200)]
    pub name: Option<String>,
    /// Password for encrypted backup (only for .backup format)
    #[schema(example = "backup-pass-123")]
    pub password: Option<String>,
    /// Backup format: "backup" (binary .backup) or "rsc" (text export .rsc)
    #[schema(example = "backup")]
    pub format: Option<BackupFormat>,
    /// Telegram bot ID to send the backup to
    pub telegram_bot_id: Uuid,
    /// Whether to delete the backup file from device after successful send
    #[schema(example = true)]
    pub delete_after_send: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct BackupAndSendResponse {
    pub filename: String,
    pub format: String,
    pub telegram_bot_id: Uuid,
    pub telegram_success: bool,
    pub telegram_message: Option<String>,
    pub deleted_from_device: bool,
}

/// Backup activity log entry (enriched from audit_logs)
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct BackupLogResponse {
    pub id: Uuid,
    pub device_name: String,
    pub device_host: String,
    pub filename: String,
    pub format: Option<String>,
    pub telegram_bot_name: Option<String>,
    pub telegram_success: bool,
    pub deleted_from_device: bool,
    pub user_name: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Paginated list response for backup logs
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BackupLogListResponse {
    pub items: Vec<BackupLogResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
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
