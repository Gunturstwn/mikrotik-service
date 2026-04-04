use crate::models::audit_logs::ActiveModel as AuditLogActiveModel;
use crate::errors::app_error::AppError;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

pub struct AuditService;

impl AuditService {
    pub async fn log(
        db: &DatabaseConnection,
        user_id: Option<Uuid>,
        action: &str,
        method: &str,
        path: &str,
        status: i32,
        ip: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        let audit_log = AuditLogActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            action: Set(action.to_string()),
            method: Set(method.to_string()),
            path: Set(path.to_string()),
            status: Set(status),
            ip: Set(ip.to_string()),
            metadata: Set(metadata),
            created_at: Set(Utc::now().naive_utc()),
        };

        audit_log.insert(db).await?;
        Ok(())
    }
}
