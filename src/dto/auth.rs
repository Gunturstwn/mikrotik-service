use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// Nama lengkap pengguna (3-50 karakter)
    #[validate(length(min = 3, max = 50))]
    #[schema(example = "John Doe Customer")]
    pub name: String,

    /// Email pengguna (harus unik)
    #[validate(email)]
    #[schema(example = "customer@mnet.com")]
    pub email: String,

    /// Password minimal 6 karakter
    #[validate(length(min = 6))]
    #[schema(example = "secure_password_123")]
    pub password: String,

    /// Nomor telepon (opsional)
    #[schema(example = "08123456789", nullable = true)]
    pub phone: Option<String>,

    /// Alamat lengkap (opsional)
    #[schema(example = "Jl. Merdeka No. 123, Jakarta", nullable = true)]
    pub address: Option<String>,

    /// Upload file foto profil (opsional, max 5MB, otomatis resize & convert ke WebP)
    #[schema(value_type = Option<String>, format = Binary, nullable = true)]
    pub photo: Option<String>,

    /// Latitude koordinat lokasi (opsional)
    #[schema(example = "-6.2088", nullable = true)]
    pub lat: Option<f64>,

    /// Longitude koordinat lokasi (opsional)
    #[schema(example = "106.8456", nullable = true)]
    pub lng: Option<f64>,

    /// Token pembayaran (opsional)
    #[schema(example = "tok_visa", nullable = true)]
    pub payment_token: Option<String>,

    /// Role pengguna: Super Admin, Admin, Finance, Teknisi, Customer (default: Customer)
    #[schema(example = "Customer", nullable = true)]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    #[schema(example = "customer@mnet.com")]
    pub email: String,
    #[schema(example = "secure_password_123")]
    pub password: String,
    /// Cloudflare Turnstile token. Required if the system detects suspicious login failure patterns.
    #[schema(example = "1x0.0.0.0...", nullable = true)]
    pub captcha_token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT Bearer token valid for 24 hours
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// Unique identifier of the authenticated user
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
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
    #[validate(email)]
    #[schema(example = "customer@mnet.com")]
    pub email: String,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginStatusResponse {
    pub captcha_required: bool,
    pub blocked_until: Option<u64>, // Nullable, seconds remaining
}
