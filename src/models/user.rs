use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub auth_provider_id: String,
    pub auth_provider_type: String,
    pub email: String,
    pub password_hash: Option<String>, // Nullable
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>, // Nullable
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}