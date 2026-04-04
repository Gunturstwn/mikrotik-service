use crate::errors::app_error::AppError;
use crate::models::{permissions, role_permissions, user_roles};
use sea_orm::*;
use uuid::Uuid;

pub struct PermissionService;

impl PermissionService {
    /// Check if a user has a specific permission code (e.g. "users.list", "export.csv")
    pub async fn user_has_permission(
        db: &DatabaseConnection,
        user_id: Uuid,
        permission_code: &str,
    ) -> Result<bool, AppError> {
        // 1. Get user's role IDs
        let ur_list = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<Uuid> = ur_list.into_iter().map(|ur| ur.role_id).collect();

        if role_ids.is_empty() {
            return Ok(false);
        }

        // 2. Get permission ID by code
        let perm = permissions::Entity::find()
            .filter(permissions::Column::Code.eq(permission_code))
            .one(db)
            .await?;

        let perm = match perm {
            Some(p) => p,
            None => return Ok(false),
        };

        // 3. Check if any of user's roles has that permission
        let role_perm = role_permissions::Entity::find()
            .filter(role_permissions::Column::PermissionId.eq(perm.id))
            .filter(role_permissions::Column::RoleId.is_in(role_ids))
            .one(db)
            .await?;

        Ok(role_perm.is_some())
    }

    /// Get all permission codes for a user
    pub async fn get_user_permissions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let ur_list = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<Uuid> = ur_list.into_iter().map(|ur| ur.role_id).collect();

        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        let rp_list = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.is_in(role_ids))
            .all(db)
            .await?;

        let perm_ids: Vec<Uuid> = rp_list.into_iter().map(|rp| rp.permission_id).collect();

        if perm_ids.is_empty() {
            return Ok(vec![]);
        }

        let perms = permissions::Entity::find()
            .filter(permissions::Column::Id.is_in(perm_ids))
            .all(db)
            .await?;

        Ok(perms.into_iter().map(|p| p.code).collect())
    }
}
