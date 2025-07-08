use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use sqlx::FromRow;
use rust_decimal::Decimal;
use serde_json::Value as JsonValue; // For JSONB

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub transaction_date: NaiveDate,
    pub description: String,
    pub r#type: String, // 'type' is a Rust keyword
    pub category_id: Option<Uuid>, // Nullable
    pub tags_json: Option<JsonValue>, // Nullable for JSONB
    pub amount: Decimal, // NUMERIC(18,2)
    pub currency_code: String,
    pub is_reconciled: bool,
    pub reconciliation_date: Option<NaiveDate>, // Nullable
    pub notes: Option<String>, // Nullable
    pub source_document_url: Option<String>, // Nullable
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

// Optional: Enum for transaction_type for better type safety
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
    JournalEntry,
    OpeningBalance,
    Adjustment,
}

// Implement FromStr, sqlx::Type, Decode, Encode for TransactionType similarly
impl std::str::FromStr for TransactionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INCOME" => Ok(TransactionType::Income),
            "EXPENSE" => Ok(TransactionType::Expense),
            "TRANSFER" => Ok(TransactionType::Transfer),
            "JOURNAL_ENTRY" => Ok(TransactionType::JournalEntry),
            "OPENING_BALANCE" => Ok(TransactionType::OpeningBalance),
            "ADJUSTMENT" => Ok(TransactionType::Adjustment),
            _ => Err(format!("'{}' is not a valid TransactionType", s)),
        }
    }
}

impl From<TransactionType> for String {
    fn from(tt: TransactionType) -> Self {
        match tt {
            TransactionType::Income => "INCOME".to_string(),
            TransactionType::Expense => "EXPENSE".to_string(),
            TransactionType::Transfer => "TRANSFER".to_string(),
            TransactionType::JournalEntry => "JOURNAL_ENTRY".to_string(),
            TransactionType::OpeningBalance => "OPENING_BALANCE".to_string(),
            TransactionType::Adjustment => "ADJUSTMENT".to_string(),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for TransactionType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TransactionType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for TransactionType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.to_string(), buf)
    }
}