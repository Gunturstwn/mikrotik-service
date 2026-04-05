use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::aes_gcm::{encrypt, decrypt};
use crate::errors::app_error::AppError;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mikrotik_clients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name_device: String,
    pub host: String,
    pub username: String, // Encrypted
    pub password: String, // Encrypted
    pub port_winbox: Option<String>, // Encrypted
    pub port_api: Option<String>, // Encrypted
    pub port_ftp: Option<String>, // Encrypted
    pub port_ssh: Option<i32>,
    pub location: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub timezone: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub deleted_by: Option<Uuid>,
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
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::DeletedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    DeletedBy,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn decrypt_username(&self, key: &str) -> Result<String, AppError> {
        decrypt(&self.username, key)
    }

    pub fn decrypt_password(&self, key: &str) -> Result<String, AppError> {
        decrypt(&self.password, key)
    }

    pub fn decrypt_port_winbox(&self, key: &str) -> Result<Option<String>, AppError> {
        match &self.port_winbox {
            Some(p) => Ok(Some(decrypt(p, key)?)),
            None => Ok(None),
        }
    }

    pub fn decrypt_port_api(&self, key: &str) -> Result<Option<String>, AppError> {
        match &self.port_api {
            Some(p) => Ok(Some(decrypt(p, key)?)),
            None => Ok(None),
        }
    }

    pub fn decrypt_port_ftp(&self, key: &str) -> Result<Option<String>, AppError> {
        match &self.port_ftp {
            Some(p) => Ok(Some(decrypt(p, key)?)),
            None => Ok(None),
        }
    }

    pub fn set_encrypted_fields(&mut self, username: &str, password: &str, winbox: Option<&str>, api: Option<&str>, ftp: Option<&str>, key: &str) -> Result<(), AppError> {
        self.username = encrypt(username, key)?;
        self.password = encrypt(password, key)?;
        self.port_winbox = match winbox {
            Some(p) => Some(encrypt(p, key)?),
            None => None,
        };
        self.port_api = match api {
            Some(p) => Some(encrypt(p, key)?),
            None => None,
        };
        self.port_ftp = match ftp {
            Some(p) => Some(encrypt(p, key)?),
            None => None,
        };
        Ok(())
    }
}
