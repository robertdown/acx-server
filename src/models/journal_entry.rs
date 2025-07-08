use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use rust_decimal::Decimal;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub entry_type: String, // Consider an enum here: JournalEntryType
    pub amount: Decimal, // NUMERIC(18,2)
    pub currency_code: String,
    pub exchange_rate: Option<Decimal>, // Nullable NUMERIC(18,6)
    pub converted_amount: Option<Decimal>, // Nullable NUMERIC(18,2)
    pub memo: Option<String>, // Nullable
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

// Optional: Enum for entry_type for better type safety
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JournalEntryType {
    Debit,
    Credit,
}

// Implement FromStr, sqlx::Type, Decode, Encode for JournalEntryType similarly
impl std::str::FromStr for JournalEntryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEBIT" => Ok(JournalEntryType::Debit),
            "CREDIT" => Ok(JournalEntryType::Credit),
            _ => Err(format!("'{}' is not a valid JournalEntryType", s)),
        }
    }
}

impl From<JournalEntryType> for String {
    fn from(jet: JournalEntryType) -> Self {
        match jet {
            JournalEntryType::Debit => "DEBIT".to_string(),
            JournalEntryType::Credit => "CREDIT".to_string(),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for JournalEntryType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for JournalEntryType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for JournalEntryType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.to_string(), buf)
    }
}