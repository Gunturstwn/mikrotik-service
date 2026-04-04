use bcrypt::{hash, verify, DEFAULT_COST};
use crate::errors::app_error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| AppError::InternalServerError(format!("Password hashing failed: {}", e)))
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, AppError> {
    verify(password, hashed)
        .map_err(|e| AppError::InternalServerError(format!("Password verification failed: {}", e)))
}
