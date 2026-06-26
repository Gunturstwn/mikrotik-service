use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub photo: Option<String>,
    pub address: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub payment_token: Option<String>,
    pub is_verified: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// Nama lengkap (opsional)
    #[schema(example = "John Updated", nullable = true)]
    pub name: Option<String>,

    /// Nomor telepon (opsional)
    #[schema(example = "08123456789", nullable = true)]
    pub phone: Option<String>,

    /// Alamat lengkap (opsional)
    #[schema(example = "Jl. Baru No. 456", nullable = true)]
    pub address: Option<String>,

    /// File foto profil (opsional, gunakan format multipart/form-data)
    #[schema(value_type = Option<String>, format = Binary, nullable = true)]
    pub photo: Option<String>,


    /// Latitude koordinat (opsional)
    #[schema(example = "-6.2088", nullable = true)]
    pub lat: Option<f64>,

    /// Longitude koordinat (opsional)
    #[schema(example = "106.8456", nullable = true)]
    pub lng: Option<f64>,

    /// Token pembayaran (opsional)
    #[schema(example = "tok_visa_new", nullable = true)]
    pub payment_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserListResponse {
    pub items: Vec<UserProfileResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    /// Current password for verification
    #[validate(length(min = 1, message = "Current password cannot be empty"))]
    #[schema(example = "current_password_123", min_length = 1)]
    pub current_password: String,

    /// New password (min 6 characters)
    #[validate(length(min = 6, message = "New password must be at least 6 characters"))]
    #[schema(example = "new_secure_password_456", min_length = 6)]
    pub new_password: String,
}

#[derive(Debug, ToSchema)]
pub struct UploadPhotoRequest {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}
