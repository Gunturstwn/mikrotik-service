use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::models::telegram_bots::{self, Entity as TelegramBot, ActiveModel};
use crate::dto::telegram::{CreateTelegramBotRequest, UpdateTelegramBotRequest, TelegramBotResponse, TelegramTestResponse};
use crate::errors::app_error::AppError;

pub struct TelegramService;

impl TelegramService {
    /// Mask token untuk response (keamanan)
    fn mask_token(token: &str) -> String {
        if token.len() <= 8 {
            return "********".to_string();
        }
        let visible = &token[token.len() - 8..];
        format!("***{}", visible)
    }

    fn to_response(model: telegram_bots::Model) -> TelegramBotResponse {
        TelegramBotResponse {
            id: model.id,
            name: model.name,
            token_masked: Self::mask_token(&model.token),
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
    pub async fn list(db: &DatabaseConnection) -> Result<Vec<TelegramBotResponse>, AppError> {
        let bots = TelegramBot::find()
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .order_by_desc(telegram_bots::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(bots.into_iter().map(Self::to_response).collect())
    }

    /// Get single bot by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: Uuid) -> Result<TelegramBotResponse, AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        Ok(Self::to_response(bot))
    }

    /// Create new telegram bot
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: CreateTelegramBotRequest,
    ) -> Result<TelegramBotResponse, AppError> {
        let now = Utc::now().naive_utc();
        let model = ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(req.name),
            token: Set(req.token),
            chat_id: Set(req.chat_id),
            is_active: Set(req.is_active.unwrap_or(true)),
            description: Set(req.description),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            created_by: Set(user_id),
            updated_by: Set(None),
        };

        let result = model.insert(db).await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(Self::to_response(result))
    }

    /// Update telegram bot
    pub async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
        req: UpdateTelegramBotRequest,
    ) -> Result<TelegramBotResponse, AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        let mut active: ActiveModel = bot.into();
        if let Some(name) = req.name { active.name = Set(name); }
        if let Some(token) = req.token { active.token = Set(token); }
        if let Some(chat_id) = req.chat_id { active.chat_id = Set(chat_id); }
        if let Some(desc) = req.description { active.description = Set(Some(desc)); }
        if let Some(is_active) = req.is_active { active.is_active = Set(is_active); }
        active.updated_at = Set(Utc::now().naive_utc());
        active.updated_by = Set(Some(user_id));

        let result = active.update(db).await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(Self::to_response(result))
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
    pub async fn test_send(db: &DatabaseConnection, id: Uuid) -> Result<TelegramTestResponse, AppError> {
        let bot = TelegramBot::find_by_id(id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Telegram bot not found".to_string()))?;

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            bot.token
        );

        let message = format!(
            "✅ *Test Connection Berhasil*\n\nBot: {}\nWaktu: {}",
            bot.name,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let client = reqwest::Client::new();
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

    /// Send message via specific bot (untuk digunakan service lain)
    pub async fn send_message(
        db: &DatabaseConnection,
        bot_id: Uuid,
        message: &str,
    ) -> Result<bool, AppError> {
        let bot = TelegramBot::find_by_id(bot_id)
            .filter(telegram_bots::Column::DeletedAt.is_null())
            .filter(telegram_bots::Column::IsActive.eq(true))
            .one(db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Active telegram bot not found".to_string()))?;

        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot.token);

        let client = reqwest::Client::new();
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
