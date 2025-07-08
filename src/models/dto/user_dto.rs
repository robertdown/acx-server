use serde::{Deserialize, Serialize};
use uuid::Uuid; // Needed if you reference other Uuids in DTOs (e.g., parent IDs)
use validator::Validate;

// DTO for creating a new User
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 1, max = 255))]
    pub auth_provider_id: String,
    #[validate(length(min = 1, max = 50))]
    pub auth_provider_type: String, // e.g., 'EMAIL_PASSWORD', 'GOOGLE'
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))] // Client provides plaintext password, which will be hashed
    pub password: Option<String>, // Nullable if using OAuth/SSO where password isn't directly provided
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
}

// DTO for updating an existing User
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 1, max = 255))]
    pub auth_provider_id: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub auth_provider_type: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8))]
    pub password: Option<String>, // Client provides plaintext new password if updating
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
    // last_login_at, created_at, updated_at are managed by the DB/service
}