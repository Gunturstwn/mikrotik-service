use std::collections::HashMap;
use crate::dto::audit::{AuditLogListResponse, AuditLogQuery, AuditLogResponse};
use crate::models::audit_logs;
use crate::models::audit_logs::ActiveModel as AuditLogActiveModel;
use crate::models::audit_logs::Entity as AuditLogEntity;
use crate::models::users::Entity as UserEntity;
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

        if let Err(e) = audit_log.insert(db).await {
            tracing::warn!("Audit log insert failed: {} (action: {}, path: {})", e, action, path);
        }
        Ok(())
    }

    /// List audit logs with filtering and pagination
    pub async fn list_logs(
        db: &DatabaseConnection,
        query: AuditLogQuery,
    ) -> Result<AuditLogListResponse, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

        let mut condition = Condition::all();

        if let Some(action) = &query.action {
            condition = condition.add(audit_logs::Column::Action.eq(action));
        }
        if let Some(user_id) = &query.user_id {
            condition = condition.add(audit_logs::Column::UserId.eq(*user_id));
        }
        if let Some(method) = &query.method {
            condition = condition.add(audit_logs::Column::Method.eq(method));
        }
        if let Some(status) = query.status {
            condition = condition.add(audit_logs::Column::Status.eq(status));
        }
        if let Some(ip) = &query.ip {
            condition = condition.add(audit_logs::Column::Ip.contains(ip));
        }
        if let Some(path) = &query.path {
            condition = condition.add(audit_logs::Column::Path.contains(path));
        }
        if let Some(start_date) = &query.start_date {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
                let dt = d.and_hms_opt(0, 0, 0).unwrap();
                condition = condition.add(audit_logs::Column::CreatedAt.gte(dt));
            }
        }
        if let Some(end_date) = &query.end_date {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
                let dt = d.and_hms_opt(23, 59, 59).unwrap();
                condition = condition.add(audit_logs::Column::CreatedAt.lte(dt));
            }
        }

        let total = AuditLogEntity::find()
            .filter(condition.clone())
            .count(db)
            .await?;

        let total_pages = (total as f64 / page_size as f64).ceil() as u64;

        let offset = (page - 1) * page_size;

        let audit_items = AuditLogEntity::find()
            .filter(condition)
            .order_by_desc(audit_logs::Column::CreatedAt)
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        // Batch-resolve user names
        let user_map = Self::resolve_user_names(db, &audit_items).await;

        let items: Vec<AuditLogResponse> = audit_items.into_iter().map(|m| {
            let user_name = m.user_id.and_then(|uid| user_map.get(&uid).cloned());
            Self::to_response_with_user(m, user_name)
        }).collect();

        Ok(AuditLogListResponse {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    /// Get a single audit log by ID
    pub async fn get_log(db: &DatabaseConnection, id: Uuid) -> Result<AuditLogResponse, AppError> {
        let log = AuditLogEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Audit log not found".to_string()))?;

        let user_name = if let Some(uid) = log.user_id {
            crate::models::users::Entity::find_by_id(uid)
                .one(db)
                .await
                .ok()
                .flatten()
                .map(|u| u.name)
        } else {
            None
        };

        Ok(Self::to_response_with_user(log, user_name))
    }

    /// Batch-resolve user names from a list of audit log models
    async fn resolve_user_names(db: &DatabaseConnection, items: &[audit_logs::Model]) -> HashMap<Uuid, String> {
        let user_ids: Vec<Uuid> = items.iter()
            .filter_map(|m| m.user_id)
            .collect();

        if user_ids.is_empty() {
            return HashMap::new();
        }

        let users = UserEntity::find()
            .filter(crate::models::users::Column::Id.is_in(user_ids))
            .all(db)
            .await;

        match users {
            Ok(user_list) => user_list.into_iter().map(|u| (u.id, u.name)).collect(),
            Err(_) => HashMap::new(),
        }
    }

    fn to_response_with_user(model: audit_logs::Model, user_name: Option<String>) -> AuditLogResponse {
        AuditLogResponse {
            id: model.id,
            user_id: model.user_id,
            user_name,
            action: model.action,
            method: model.method,
            path: model.path,
            status: model.status,
            ip: model.ip,
            metadata: model.metadata,
            created_at: model.created_at,
        }
    }
}
