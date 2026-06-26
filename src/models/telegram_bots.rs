use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::aes_gcm::{encrypt, decrypt};
use crate::errors::app_error::AppError;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "telegram_bots")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    /// Token terenkripsi AES-256-GCM (Base64). Jangan pernah disimpan plaintext.
    pub token: String,
    pub chat_id: String,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CreatedBy,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UpdatedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    UpdatedBy,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Dekripsi token Telegram untuk digunakan saat mengirim pesan.
    pub fn decrypt_token(&self, key: &str) -> Result<String, AppError> {
        decrypt(&self.token, key)
    }

    /// Enkripsi token Telegram dan set ke field token.
    pub fn set_encrypted_token(&mut self, plaintext: &str, key: &str) -> Result<(), AppError> {
        self.token = encrypt(plaintext, key)?;
        Ok(())
    }
}
