use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// DTO for creating a new Account
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateAccountDto {
    pub account_type_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 50))]
    pub account_code: Option<String>, // Optional
    pub description: Option<String>,
    #[validate(length(equal = 3))]
    pub currency_code: String,
    // tenant_id and created_by will be derived from context
}

// DTO for updating an existing Account
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateAccountDto {
    pub account_type_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 50))]
    pub account_code: Option<String>,
    pub description: Option<String>,
    #[validate(length(equal = 3))]
    pub currency_code: Option<String>,
    pub is_active: Option<bool>,
    // updated_by will be derived from context
}
