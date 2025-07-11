use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// DTO for creating a new ExchangeRate
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateExchangeRateDto {
    pub tenant_id: Option<Uuid>, // Nullable, only provided if tenant-specific rate

    #[validate(length(equal = 3))]
    pub base_currency_code: String,

    #[validate(length(equal = 3))]
    pub target_currency_code: String,

    #[validate(range(min = 0.000001))] // Rate must be greater than 0
    pub rate: Decimal,

    pub rate_date: NaiveDate,

    #[validate(length(max = 100))]
    pub source: Option<String>,
}

// DTO for updating an existing ExchangeRate
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateExchangeRateDto {
    #[validate(range(min = 0.000001))]
    pub rate: Option<Decimal>,

    pub rate_date: Option<NaiveDate>,

    #[validate(length(max = 100))]
    pub source: Option<String>,
}
