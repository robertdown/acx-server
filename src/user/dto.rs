use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::user::models::User;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    pub auth_provider_id: String,

    pub auth_provider_type: String,

    #[validate(email)]
    pub email: String,

    pub password: Option<String>,

    #[validate(length(min = 2, message = "First name must be at least 2 characters"))]
    pub first_name: String,

    #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
    pub last_name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    pub password: Option<String>,
    #[validate(length(min = 2, message = "First name must be at least 2 characters"))]
    pub first_name: Option<String>,
    #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub auth_provider_id: String,
    pub auth_provider_type: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            auth_provider_id: user.auth_provider_id,
            auth_provider_type: user.auth_provider_type,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_active: user.is_active,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
