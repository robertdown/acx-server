use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,      // Nullable
    pub r#type: String,                   // 'type' is a Rust keyword, so we use r#type
    pub parent_category_id: Option<Uuid>, // Nullable
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

// Optional: Enum for category_type for better type safety
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CategoryType {
    Income,
    Expense,
    Transfer,
    Investment,
    Other,
}

// Implement FromStr, sqlx::Type, Decode, Encode for CategoryType similarly to AccountNormalBalance
impl std::str::FromStr for CategoryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INCOME" => Ok(CategoryType::Income),
            "EXPENSE" => Ok(CategoryType::Expense),
            "TRANSFER" => Ok(CategoryType::Transfer),
            "INVESTMENT" => Ok(CategoryType::Investment),
            "OTHER" => Ok(CategoryType::Other),
            _ => Err(format!("'{}' is not a valid CategoryType", s)),
        }
    }
}

impl From<CategoryType> for String {
    fn from(ct: CategoryType) -> Self {
        match ct {
            CategoryType::Income => "INCOME".to_string(),
            CategoryType::Expense => "EXPENSE".to_string(),
            CategoryType::Transfer => "TRANSFER".to_string(),
            CategoryType::Investment => "INVESTMENT".to_string(),
            CategoryType::Other => "OTHER".to_string(),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CategoryType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CategoryType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for CategoryType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.to_string(), buf)
    }
}
