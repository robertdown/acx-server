use serde::{Deserialize, Serialize};
use validator::Validate;

// DTO for creating a new Currency
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateCurrencyDto {
    #[validate(length(equal = 3))] // ISO 4217 code, e.g., 'USD'
    pub code: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 10))]
    pub symbol: Option<String>,
    // created_by will be system user
}

// DTO for updating an existing Currency
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateCurrencyDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 10))]
    pub symbol: Option<String>,
    pub is_active: Option<bool>,
    // updated_by will be system user
}