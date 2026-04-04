use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    #[schema(example = "John Doe Customer")]
    pub name: String,
    #[validate(email)]
    #[schema(example = "customer@mnet.com")]
    pub email: String,
    #[validate(length(min = 6))]
    #[schema(example = "secure_password_123")]
    pub password: String,
    #[schema(example = "08123456789")]
    pub phone: Option<String>,
    #[schema(example = "Customer")]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    #[schema(example = "customer@mnet.com")]
    pub email: String,
    #[schema(example = "secure_password_123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    #[schema(example = "customer@mnet.com")]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub token: String,
    #[validate(length(min = 6))]
    #[schema(example = "new_secure_password_456")]
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyTokenResponse {
    pub valid: bool,
    pub user_id: Option<String>,
}
