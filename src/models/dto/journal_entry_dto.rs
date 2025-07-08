use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use rust_decimal::Decimal;
use crate::models::journal_entry::JournalEntryType; // Import the enum

// DTO for creating a new JournalEntry
// Note: transaction_id would typically be provided by the service creating the overall transaction,
// not directly by the client in this DTO unless it's for a specific scenario.
// For composite transaction creation, a Transaction DTO might embed multiple JournalEntry DTOs.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateJournalEntryDto {
    pub account_id: Uuid,
    pub entry_type: JournalEntryType, // Use the enum
    #[validate(range(min = 0.0))] // Amount must be non-negative
    pub amount: Decimal,
    #[validate(length(equal = 3))]
    pub currency_code: String,
    pub exchange_rate: Option<Decimal>,
    pub converted_amount: Option<Decimal>,
    pub memo: Option<String>,
    // transaction_id, created_by will be derived from context/parent operation
}

// DTO for updating an existing JournalEntry
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateJournalEntryDto {
    pub account_id: Option<Uuid>,
    pub entry_type: Option<JournalEntryType>, // Use the enum
    #[validate(range(min = 0.0))]
    pub amount: Option<Decimal>,
    #[validate(length(equal = 3))]
    pub currency_code: Option<String>,
    pub exchange_rate: Option<Decimal>,
    pub converted_amount: Option<Decimal>,
    pub memo: Option<String>,
    // updated_by will be derived from context
}