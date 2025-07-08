use serde::{Deserialize, Serialize};
use validator::Validate;

// DTO for creating a new Tenant
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTenantDto {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 100))]
    pub industry: Option<String>,
    #[validate(length(equal = 3))] // ISO 4217 code, e.g., 'USD'
    pub base_currency_code: String,
    #[validate(range(min = 1, max = 12))]
    pub fiscal_year_end_month: i32,
    // created_by will be derived from authenticated user
}

// DTO for updating an existing Tenant
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTenantDto {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 100))]
    pub industry: Option<String>,
    #[validate(length(equal = 3))]
    pub base_currency_code: Option<String>,
    #[validate(range(min = 1, max = 12))]
    pub fiscal_year_end_month: Option<i32>,
    pub is_active: Option<bool>,
    // updated_by will be derived from authenticated user
}