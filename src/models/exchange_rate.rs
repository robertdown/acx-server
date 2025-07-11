use chrono::{DateTime, NaiveDate, Utoc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>, // Nullable
    pub base_currency_code: String,
    pub target_currency_code: String,
    pub rate: Decimal,          // NUMERIC(18,6)
    pub rate_date: NaiveDate,   // DATE
    pub source: Option<String>, // Nullable
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}
