use crate::dto::auth::{RegisterRequest, LoginRequest, AuthResponse, VerifyTokenResponse, ForgotPasswordRequest, ResetPasswordRequest};
use crate::models::users::{Entity as User, ActiveModel as UserActiveModel};
use crate::utils::encryption::{hash_password, verify_password};
use crate::config::auth::create_token;
use crate::errors::app_error::AppError;
use sea_orm::*;
use crate::models::{roles, user_roles};
use uuid::Uuid;
use chrono::Utc;
use tracing::info;

pub struct AuthService;

impl AuthService {
    pub async fn register(db: &DatabaseConnection, rabbit: &crate::queue::rabbitmq_client::RabbitMQClient, req: RegisterRequest) -> Result<AuthResponse, AppError> {
        let password_hash = hash_password(&req.password)?;
        let user_id = Uuid::new_v4();

        // 1. Resolve role (default to "Customer" if none specified)
        let role_name = req.role.unwrap_or_else(|| "Customer".to_string());
        
        let role = roles::Entity::find()
            .filter(roles::Column::Name.eq(&role_name))
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("Role '{}' not found", role_name)))?;

        // 2. Create the user
        let user = UserActiveModel {
            id: Set(user_id.into()),
            name: Set(req.name.clone()),
            email: Set(req.email.clone()),
            password: Set(password_hash),
            phone: Set(req.phone),
            is_verified: Set(false),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        user.insert(db).await.map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::BadRequest("Email already exists".to_string())
            } else {
                AppError::DatabaseError(e)
            }
        })?;

        // 3. Link user to role
        let user_role = user_roles::ActiveModel {
            user_id: Set(user_id.into()),
            role_id: Set(role.id),
            ..Default::default()
        };
        user_role.insert(db).await?;

        // 4. Trigger Email Notification via RabbitMQ
        let email_payload = serde_json::json!({
            "to": req.email,
            "subject": "Welcome to MikroTik Billing!",
            "body": format!("Hello {},\n\nYour account has been created with the role '{}'.\nPlease verify your email.", req.name, role_name)
        });
        
        if let Err(e) = rabbit.publish("email_queue", &email_payload.to_string()).await {
            tracing::error!("Failed to publish welcome email for {}: {}", req.email, e);
        }

        let token = create_token(user_id, vec![role_name])?;

        Ok(AuthResponse {
            token,
            user_id: user_id.to_string(),
        })
    }

    pub async fn login(db: &DatabaseConnection, req: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = User::find()
            .filter(crate::models::users::Column::Email.eq(req.email))
            .one(db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        if !verify_password(&req.password, &user.password)? {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        if !user.is_verified {
            return Err(AppError::Forbidden("Email not verified. Please check your inbox.".to_string()));
        }

        let user_roles_mapped = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user.id))
            .all(db)
            .await?;

        let role_ids: Vec<Uuid> = user_roles_mapped.into_iter().map(|ur| ur.role_id).collect();
        
        let db_roles = roles::Entity::find()
            .filter(roles::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        let mut token_roles: Vec<String> = db_roles.into_iter().map(|r| r.name).collect();
        
        if token_roles.is_empty() {
            token_roles.push("Customer".to_string());
        }

        let token = create_token(user.id.into(), token_roles)?;

        Ok(AuthResponse {
            token,
            user_id: user.id.to_string(),
        })
    }

    pub async fn verify_token(token: &str) -> Result<VerifyTokenResponse, AppError> {
        match crate::config::auth::verify_token(token) {
            Ok(claims) => Ok(VerifyTokenResponse {
                valid: true,
                user_id: Some(claims.sub.to_string()),
            }),
            Err(_) => Ok(VerifyTokenResponse {
                valid: false,
                user_id: None,
            }),
        }
    }

    pub async fn verify_email(db: &DatabaseConnection, user_id: Uuid) -> Result<(), AppError> {
        let user = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let mut user_active: UserActiveModel = user.into();
        user_active.is_verified = Set(true);
        user_active.updated_at = Set(Utc::now().naive_utc());
        
        user_active.update(db).await?;
        
        Ok(())
    }

    pub async fn forgot_password(
        db: &DatabaseConnection,
        redis: &crate::cache::RedisClient,
        rabbit: &crate::queue::rabbitmq_client::RabbitMQClient,
        req: ForgotPasswordRequest,
    ) -> Result<(), AppError> {
        // 1. Find the user by email
        let user = User::find()
            .filter(crate::models::users::Column::Email.eq(&req.email))
            .one(db)
            .await?;

        // Always return OK to avoid email enumeration attacks
        let user = match user {
            Some(u) => u,
            None => {
                info!("Forgot password for non-existent email: {}", req.email);
                return Ok(());
            }
        };

        // 2. Generate a secure random token
        let token = Uuid::new_v4().to_string();
        let redis_key = format!("password_reset:{}", token);

        // 3. Store in Redis with 1-hour TTL
        redis.set(&redis_key, &user.id.to_string(), 3600).await?;

        // 4. Send reset email via RabbitMQ
        let email_payload = serde_json::json!({
            "to": req.email,
            "subject": "MikroTik Billing — Reset Password",
            "body": format!(
                "Hello {},\n\nYou requested a password reset.\nYour reset token: {}\n\nThis token expires in 1 hour.\nIf you did not request this, ignore this email.",
                user.name, token
            )
        });

        if let Err(e) = rabbit.publish("email_queue", &email_payload.to_string()).await {
            tracing::error!("Failed to send password reset email for {}: {}", req.email, e);
        }

        info!("Password reset token generated for: {}", req.email);
        Ok(())
    }

    pub async fn reset_password(
        db: &DatabaseConnection,
        redis: &crate::cache::RedisClient,
        req: ResetPasswordRequest,
    ) -> Result<(), AppError> {
        if req.token.is_empty() {
            return Err(AppError::BadRequest("Reset token is required".to_string()));
        }

        // 1. Look up token in Redis
        let redis_key = format!("password_reset:{}", req.token);
        let user_id_str = redis.get(&redis_key).await?
            .ok_or_else(|| AppError::BadRequest("Invalid or expired reset token".to_string()))?;

        let user_id: Uuid = user_id_str.parse()
            .map_err(|_| AppError::InternalServerError("Corrupted reset token data".to_string()))?;

        // 2. Hash the new password
        let new_hash = hash_password(&req.new_password)?;

        // 3. Update user in DB
        let user = User::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let mut user_active: UserActiveModel = user.into();
        user_active.password = Set(new_hash);
        user_active.updated_at = Set(Utc::now().naive_utc());
        user_active.update(db).await?;

        // 4. Delete the used token from Redis
        redis.del(&redis_key).await?;

        info!("Password successfully reset for user_id: {}", user_id);
        Ok(())
    }
}
