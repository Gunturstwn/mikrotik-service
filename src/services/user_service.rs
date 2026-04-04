use crate::dto::user::{UserProfileResponse, UpdateUserRequest, UserListResponse};
use crate::models::users::{Entity as User, ActiveModel as UserActiveModel};
use crate::models::{roles, user_roles};
use crate::errors::app_error::AppError;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

pub struct UserService;

impl UserService {
    pub async fn get_profile(db: &DatabaseConnection, user_id: Uuid) -> Result<UserProfileResponse, AppError> {
        let user = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        let user_role_names = Self::resolve_roles(db, user.id.into()).await?;

        Ok(UserProfileResponse {
            id: user.id.into(),
            name: user.name,
            email: user.email,
            phone: user.phone,
            photo: user.photo,
            address: user.address,
            is_verified: user.is_verified,
            roles: user_role_names,
        })
    }

    pub async fn update_profile(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserProfileResponse, AppError> {
        let mut user: UserActiveModel = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?
            .into();

        if let Some(name) = req.name {
            user.name = Set(name);
        }
        if let Some(phone) = req.phone {
            user.phone = Set(Some(phone));
        }
        if let Some(address) = req.address {
            user.address = Set(Some(address));
        }
        if let Some(photo) = req.photo {
            user.photo = Set(Some(photo));
        }
        user.updated_at = Set(Utc::now().naive_utc());

        let updated_user = user.update(db).await?;
        let user_role_names = Self::resolve_roles(db, updated_user.id.into()).await?;

        Ok(UserProfileResponse {
            id: updated_user.id.into(),
            name: updated_user.name,
            email: updated_user.email,
            phone: updated_user.phone,
            photo: updated_user.photo,
            address: updated_user.address,
            is_verified: updated_user.is_verified,
            roles: user_role_names,
        })
    }

    pub async fn find_all(
        db: &DatabaseConnection,
        page: u64,
        page_size: u64,
    ) -> Result<UserListResponse, AppError> {
        let paginator = User::find()
            .filter(crate::models::users::Column::DeletedAt.is_null())
            .paginate(db, page_size);

        let total = paginator.num_items().await?;
        let users = paginator.fetch_page(page - 1).await?;

        let mut items = Vec::new();
        for user in users {
            let user_role_names = Self::resolve_roles(db, user.id.into()).await?;
            items.push(UserProfileResponse {
                id: user.id.into(),
                name: user.name,
                email: user.email,
                phone: user.phone,
                photo: user.photo,
                address: user.address,
                is_verified: user.is_verified,
                roles: user_role_names,
            });
        }

        Ok(UserListResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn soft_delete(db: &DatabaseConnection, user_id: Uuid) -> Result<(), AppError> {
        let mut user: UserActiveModel = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?
            .into();

        user.deleted_at = Set(Some(Utc::now().naive_utc()));
        user.update(db).await?;

        Ok(())
    }

    /// Helper: resolve user roles from DB join
    async fn resolve_roles(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let ur_list = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<Uuid> = ur_list.into_iter().map(|ur| ur.role_id).collect();

        if role_ids.is_empty() {
            return Ok(vec!["Customer".to_string()]);
        }

        let db_roles = roles::Entity::find()
            .filter(roles::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        Ok(db_roles.into_iter().map(|r| r.name).collect())
    }
}
