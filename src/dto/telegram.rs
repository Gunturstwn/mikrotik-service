use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTelegramBotRequest {
    /// Nama bot (untuk identifikasi internal)
    #[schema(example = "Alert Bot Production")]
    pub name: String,
    /// Token bot dari @BotFather
    #[schema(example = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz")]
    pub token: String,
    /// Chat ID tujuan (bisa group atau personal)
    #[schema(example = "-1001234567890")]
    pub chat_id: String,
    /// Deskripsi opsional
    #[schema(example = "Bot untuk notifikasi alert MikroTik")]
    pub description: Option<String>,
    /// Apakah bot aktif
    #[schema(example = true)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTelegramBotRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub chat_id: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct TelegramBotResponse {
    pub id: Uuid,
    pub name: String,
    /// Token di-mask untuk keamanan (hanya tampilkan 8 karakter terakhir)
    pub token_masked: String,
    pub chat_id: String,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TelegramTestResponse {
    pub success: bool,
    pub message: String,
}
