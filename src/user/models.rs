use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid; // Using chrono for date/time, Utc for TIMESTAMPTZ

#[derive(Debug, FromRow, Clone)] // Derive FromRow for SQLX mapping
pub struct User {
    pub id: Uuid,
    pub auth_provider_id: String,
    pub auth_provider_type: String,
    pub email: String,
    pub password_hash: Option<String>, // Nullable
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>, // Nullable TIMESTAMPTZ
    pub created_at: DateTime<Utc>,            // TIMESTAMPTZ
    pub updated_at: DateTime<Utc>,            // TIMESTAMPTZ
}
