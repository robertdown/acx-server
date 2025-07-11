use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct AccountType {
    pub id: Uuid,
    pub name: String,
    pub normal_balance: String, // Consider an enum here: AccountNormalBalance
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

// Optional: Enum for normal_balance for better type safety
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
pub enum AccountNormalBalance {
    DEBIT,
    CREDIT,
}

// Implementing From<AccountNormalBalance> for String and vice-versa for SQLx
impl From<AccountNormalBalance> for String {
    fn from(balance: AccountNormalBalance) -> Self {
        match balance {
            AccountNormalBalance::DEBIT => "DEBIT".to_string(),
            AccountNormalBalance::CREDIT => "CREDIT".to_string(),
        }
    }
}

impl std::str::FromStr for AccountNormalBalance {
    type Err = String; // Or a more specific error type
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEBIT" => Ok(AccountNormalBalance::DEBIT),
            "CREDIT" => Ok(AccountNormalBalance::CREDIT),
            _ => Err(format!("'{}' is not a valid AccountNormalBalance", s)),
        }
    }
}

// This allows sqlx to decode from String and encode to String
impl sqlx::Type<sqlx::Postgres> for AccountNormalBalance {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AccountNormalBalance {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse().map_err(Into::into) // Convert String error to BoxDynError
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AccountNormalBalance {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.to_string(), buf)
    }
}
