use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::errors::app_error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub roles: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

// Issue fix #10: Cache JWT_SECRET agar tidak dibaca dari env setiap request.
// OnceLock diinisialisasi sekali saat pertama kali digunakan (thread-safe).
static JWT_SECRET_CACHE: OnceLock<String> = OnceLock::new();

fn get_jwt_secret() -> &'static str {
    JWT_SECRET_CACHE.get_or_init(|| {
        env::var("JWT_SECRET").expect("JWT_SECRET must be set")
    })
}

pub fn create_token(user_id: Uuid, roles: Vec<String>) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("invalid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        roles,
        exp: expiration,
        iat: Utc::now().timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_ref()),
    )
    .map_err(|e| AppError::InternalServerError(format!("JWT creation failed: {}", e)))
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_ref()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    .map(|data| data.claims)
}
