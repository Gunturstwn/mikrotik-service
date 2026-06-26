use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use std::time::Duration;
use crate::models::telegram_bots::{self, Entity as TelegramBot, ActiveModel};
use crate::dto::telegram::{CreateTelegramBotRequest, UpdateTelegramBotRequest, TelegramBotResponse, TelegramTestResponse};
use crate::errors::app_error::AppError;
use reqwest::multipart;

pub struct TelegramService;

impl TelegramService {
    /// Mask token untuk response — hanya tampilkan 8 karakter terakhir
    fn mask_token(token: &str) -> String {
        if token.len() <= 8 {
            return "********".to_string();
        }
        let visible = &token[token.len() - 8..];
        format!("***{}", visible)
    }

    fn to_response(model: telegram_bots::Model, aes_key: &str) -> TelegramBotResponse {
        // Dekripsi token untuk ditampilkan secara ter-mask di response
        let plain_token = model.decrypt_token(aes_key).unwrap_or_default();

        TelegramBotResponse {
            id: model.id,
            name: model.name,
            token_masked: Self::mask_token(&plain_token),
            chat_id: model.chat_id,
            is_active: model.is_active,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }

    /// List semua telegram bots (exclude soft-deleted)
    pub async fn list(db: &DatabaseConnection, aes_key: &str) -> Result<Vec<TelegramBotResponse>, AppError> {
        let bots = TelegramBot::find()
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .order_by_desc(telegram_bots::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(bots.into_iter().map(|b| Self::to_response(b, aes_key)).collect())
    }

    /// Get single bot by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: Uuid, aes_key: &str) -> Result<TelegramBotResponse, AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        Ok(Self::to_response(bot, aes_key))
    }

    /// Create new telegram bot (token akan dienkripsi AES-256-GCM sebelum disimpan)
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: CreateTelegramBotRequest,
        aes_key: &str,
    ) -> Result<TelegramBotResponse, AppError> {
        // Enkripsi token sebelum disimpan ke DB
        let mut model = telegram_bots::Model {
            id: Uuid::new_v4(),
            name: req.name,
            token: String::new(), // Akan diisi via set_encrypted_token
            chat_id: req.chat_id,
            is_active: req.is_active.unwrap_or(true),
            description: req.description,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            created_by: user_id,
            updated_by: None,
        };

        model.set_encrypted_token(&req.token, aes_key)?;

        let active_model: ActiveModel = model.into();
        let result = active_model.insert(db).await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(Self::to_response(result, aes_key))
    }

    /// Update telegram bot (token akan dienkripsi jika ada perubahan)
    pub async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
        req: UpdateTelegramBotRequest,
        aes_key: &str,
    ) -> Result<TelegramBotResponse, AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        let mut active: ActiveModel = bot.into();
        if let Some(name) = req.name { active.name = Set(name); }
        if let Some(token) = req.token {
            // Enkripsi token baru sebelum disimpan
            let mut temp_model = telegram_bots::Model {
                id: Uuid::nil(),
                name: String::new(),
                token: String::new(),
                chat_id: String::new(),
                is_active: false,
                description: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                deleted_at: None,
                created_by: Uuid::nil(),
                updated_by: None,
            };
            temp_model.set_encrypted_token(&token, aes_key)?;
            active.token = Set(temp_model.token);
        }
        if let Some(chat_id) = req.chat_id { active.chat_id = Set(chat_id); }
        if let Some(desc) = req.description { active.description = Set(Some(desc)); }
        if let Some(is_active) = req.is_active { active.is_active = Set(is_active); }
        active.updated_at = Set(Utc::now().naive_utc());
        active.updated_by = Set(Some(user_id));

        let result = active.update(db).await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(Self::to_response(result, aes_key))
    }

    /// Soft delete telegram bot
    pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        let mut active: ActiveModel = bot.into();
        active.deleted_at = Set(Some(Utc::now().naive_utc()));

        active.update(db).await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(())
    }

    /// Test bot: kirim pesan test ke chat_id
    pub async fn test_send(db: &DatabaseConnection, id: Uuid, aes_key: &str) -> Result<TelegramTestResponse, AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        // Dekripsi token untuk digunakan
        let plain_token = bot.decrypt_token(aes_key)?;

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            plain_token
        );

        let message = format!(
            "✅ *Test Connection Berhasil*\\n\\nBot: {}\\nWaktu: {}",
            bot.name,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Failed to build Telegram client: {}", e)))?;
        let resp = client
            .post(&url)
            .form(&[
                ("chat_id", bot.chat_id.as_str()),
                ("text", message.as_str()),
                ("parse_mode", "Markdown"),
            ])
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to send Telegram message: {}", e)))?;

        if resp.status().is_success() {
            Ok(TelegramTestResponse {
                success: true,
                message: "Pesan test berhasil dikirim ke Telegram.".to_string(),
            })
        } else {
            let body = resp.text().await.unwrap_or_default();
            Ok(TelegramTestResponse {
                success: false,
                message: format!("Gagal mengirim pesan: {}", body),
            })
        }
    }

    /// Send a document (file) to the Telegram chat via sendDocument API
    /// Returns (success, optional_error_message) so callers can show the actual Telegram error
    pub async fn send_document(
        db: &DatabaseConnection,
        bot_id: Uuid,
        file_bytes: &[u8],
        filename: &str,
        caption: &str,
        aes_key: &str,
    ) -> Result<(bool, Option<String>), AppError> {
        let bot = TelegramBot::find_by_id(bot_id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .filter(telegram_bots::Column::IsActive.eq(true))
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Active telegram bot not found".to_string()))?;

        let plain_token = bot.decrypt_token(aes_key)?;

        let url = format!("https://api.telegram.org/bot{}/sendDocument", plain_token);

        let part = multipart::Part::bytes(file_bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| AppError::InternalServerError(format!("Failed to create multipart part: {}", e)))?;

        let form = multipart::Form::new()
            .text("chat_id", bot.chat_id.clone())
            .text("caption", caption.to_string())
            .text("parse_mode", "Markdown".to_string())
            .part("document", part);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Failed to build Telegram client: {}", e)))?;

        let resp = client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Telegram sendDocument failed: {}", e)))?;

        let success = resp.status().is_success();
        if !success {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Telegram sendDocument failed - {}", body);
            return Ok((false, Some(body)));
        }

        Ok((true, None))
    }

    /// Send message via specific bot (untuk digunakan service lain)
    pub async fn send_message(
        db: &DatabaseConnection,
        bot_id: Uuid,
        message: &str,
        aes_key: &str,
    ) -> Result<bool, AppError> {
        let bot = TelegramBot::find_by_id(bot_id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .filter(telegram_bots::Column::IsActive.eq(true))
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Active telegram bot not found".to_string()))?;

        // Dekripsi token untuk digunakan
        let plain_token = bot.decrypt_token(aes_key)?;

        let url = format!("https://api.telegram.org/bot{}/sendMessage", plain_token);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::InternalServerError(format!("Failed to build Telegram client: {}", e)))?;
        let resp = client
            .post(&url)
            .form(&[
                ("chat_id", bot.chat_id.as_str()),
                ("text", message),
                ("parse_mode", "Markdown"),
            ])
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Telegram send failed: {}", e)))?;

        Ok(resp.status().is_success())
    }
}
