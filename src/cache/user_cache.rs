use super::redis_client::RedisClient;
use crate::dto::user::UserProfileResponse;
use crate::errors::app_error::AppError;
use uuid::Uuid;

pub struct UserCache {
    client: RedisClient,
}

impl UserCache {
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<Option<UserProfileResponse>, AppError> {
        let key = format!("user:profile:{}", user_id);
        let data = self.client.get(&key).await?;
        
        if let Some(json) = data {
            let profile: UserProfileResponse = serde_json::from_str(&json)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            return Ok(Some(profile));
        }

        Ok(None)
    }

    pub async fn set_profile(&self, profile: &UserProfileResponse) -> Result<(), AppError> {
        let key = format!("user:profile:{}", profile.id);
        let json = serde_json::to_string(profile)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            
        self.client.set(&key, &json, 3600).await?; // 1 hour TTL
        
        Ok(())
    }

    pub async fn delete_profile(&self, user_id: Uuid) -> Result<(), AppError> {
        let key = format!("user:profile:{}", user_id);
        self.client.del(&key).await?;
        Ok(())
    }
}
