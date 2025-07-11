use crate::models::account_type::AccountNormalBalance;
use serde::{Deserialize, Serialize};
use validator::Validate; // Import the enum

// DTO for creating a new AccountType
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateAccountTypeDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub normal_balance: AccountNormalBalance, // Use the enum
                                              // created_by will be system user
}

// DTO for updating an existing AccountType
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateAccountTypeDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub normal_balance: Option<AccountNormalBalance>, // Use the enum
    pub is_active: Option<bool>,
    // updated_by will be system user
}
