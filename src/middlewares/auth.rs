use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::config::auth::verify_token;
use crate::errors::app_error::AppError;
use uuid::Uuid;
use crate::AppState;
use sea_orm::EntityTrait;

pub struct UserContext {
    pub user_id: Uuid,
    pub roles: Vec<String>,
}

#[async_trait]
impl FromRequestParts<AppState> for UserContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid Authorization header format".to_string()));
        }

        let token = &auth_header[7..];
        let claims = verify_token(token)?;

        // Verify in DB that the user is still verified and exists
        let user_opt = crate::models::users::Entity::find_by_id(claims.sub)
            .one(&state.db)
            .await?;

        let user = user_opt.ok_or_else(|| AppError::Unauthorized("User no longer exists".to_string()))?;

        if !user.is_verified {
            return Err(AppError::Forbidden("Account is no longer verified".to_string()));
        }

        Ok(UserContext { 
            user_id: claims.sub, 
            roles: claims.roles,
        })
    }
}
