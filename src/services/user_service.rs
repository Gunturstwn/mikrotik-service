use crate::dto::user::{UserProfileResponse, UpdateUserRequest, UserListResponse};
use crate::models::users::{Entity as User, ActiveModel as UserActiveModel};
use crate::models::{roles, user_roles};
use crate::errors::app_error::AppError;
use sea_orm::*;
use sea_orm::prelude::Decimal;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

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
            lat: user.lat.map(|v| v.to_string().parse::<f64>().unwrap_or_default()),
            lng: user.lng.map(|v| v.to_string().parse::<f64>().unwrap_or_default()),
            payment_token: user.payment_token,
            is_verified: user.is_verified,
            roles: user_role_names,
        })
    }

    pub async fn update_profile(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserProfileResponse, AppError> {
        // Issue fix #5: Gunakan NotFound (404) bukan BadRequest (400) untuk "user not found"
        let mut user: UserActiveModel = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
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
        if let Some(lat) = req.lat {
            user.lat = Set(Some(lat.to_string().parse::<Decimal>().unwrap_or_default()));
        }
        if let Some(lng) = req.lng {
            user.lng = Set(Some(lng.to_string().parse::<Decimal>().unwrap_or_default()));
        }
        if let Some(payment_token) = req.payment_token {
            user.payment_token = Set(Some(payment_token));
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
            lat: updated_user.lat.map(|v| v.to_string().parse::<f64>().unwrap_or_default()),
            lng: updated_user.lng.map(|v| v.to_string().parse::<f64>().unwrap_or_default()),
            payment_token: updated_user.payment_token,
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

        // Issue fix #4: Eliminasi N+1 query — batch load semua roles sekaligus
        // Sebelumnya: 2 query DB per user dalam loop → untuk 10 user = 20+ query
        // Sekarang: cukup 2 query total untuk semua user
        let user_ids: Vec<Uuid> = users.iter().map(|u| u.id.into()).collect();

        // Batch query semua user_roles
        let all_user_roles = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.is_in(user_ids))
            .all(db)
            .await?;

        // Kumpulkan semua role_id unik
        let all_role_ids: Vec<Uuid> = all_user_roles.iter().map(|ur| ur.role_id.into()).collect();

        // Batch query semua roles
        let all_roles = if all_role_ids.is_empty() {
            vec![]
        } else {
            roles::Entity::find()
                .filter(roles::Column::Id.is_in(all_role_ids))
                .all(db)
                .await?
        };

        // Build lookup map: role_id → role_name
        let role_map: HashMap<Uuid, String> = all_roles.into_iter()
            .map(|r| (r.id.into(), r.name))
            .collect();

        // Build lookup map: user_id → Vec<role_name>
        let mut user_roles_map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for ur in all_user_roles {
            let user_id_key: Uuid = ur.user_id.into();
            let role_id_key: Uuid = ur.role_id.into();
            if let Some(role_name) = role_map.get(&role_id_key) {
                user_roles_map
                    .entry(user_id_key)
                    .or_insert_with(Vec::new)
                    .push(role_name.clone());
            }
        }

        let mut items = Vec::new();
        for user in users {
            let user_id_key: Uuid = user.id.into();
            // Issue fix #8: Kembalikan vec kosong jika tidak ada role (tidak hardcode "Customer")
            let user_role_names = user_roles_map.get(&user_id_key).cloned().unwrap_or_default();

            items.push(UserProfileResponse {
                id: user.id.into(),
                name: user.name,
                email: user.email,
                phone: user.phone,
                photo: user.photo,
                address: user.address,
                lat: user.lat.map(|v| v.to_string().parse::<f64>().unwrap_or_default()),
                lng: user.lng.map(|v| v.to_string().parse::<f64>().unwrap_or_default()),
                payment_token: user.payment_token,
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
        // Issue fix #5: Gunakan NotFound (404) bukan BadRequest (400)
        let user_model = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Issue fix #6: Cek apakah user sudah di-delete (idempoten)
        if user_model.deleted_at.is_some() {
            return Err(AppError::NotFound("User already deleted".to_string()));
        }

        let mut user: UserActiveModel = user_model.into();
        user.deleted_at = Set(Some(Utc::now().naive_utc()));
        user.update(db).await?;

        Ok(())
    }

    /// Helper: resolve user roles dari DB join (digunakan untuk get_profile & update_profile)
    async fn resolve_roles(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let ur_list = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<Uuid> = ur_list.into_iter().map(|ur| ur.role_id).collect();

        if role_ids.is_empty() {
            // Issue fix #8: Jangan hardcode "Customer" — kembalikan vec kosong jika tidak ada role
            tracing::warn!("User {} has no roles assigned in user_roles table", user_id);
            return Ok(vec![]);
        }

        let db_roles = roles::Entity::find()
            .filter(roles::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        Ok(db_roles.into_iter().map(|r| r.name).collect())
    }
}
