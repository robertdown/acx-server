use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;
use validator::Validate;
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use crate::models::transaction::TransactionType; // Import the enum

// DTO for creating a new Transaction
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTransactionDto {
    pub transaction_date: NaiveDate,
    #[validate(length(min = 1))]
    pub description: String,
    pub r#type: TransactionType, // Use the enum
    pub category_id: Option<Uuid>,
    // For tags_json, clients might send an array of UUID strings
    pub tags: Option<Vec<Uuid>>, // Changed from JsonValue for better type safety
    #[validate(range(min = 0.01))] // Amount must be positive
    pub amount: Decimal,
    #[validate(length(equal = 3))]
    pub currency_code: String,
    pub is_reconciled: Option<bool>, // Client can optionally specify, defaults to FALSE
    pub reconciliation_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub source_document_url: Option<String>,
    // tenant_id and created_by will be derived from context
}

// DTO for updating an existing Transaction
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTransactionDto {
    pub transaction_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub r#type: Option<TransactionType>, // Use the enum
    pub category_id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>, // Changed from JsonValue for better type safety
    #[validate(range(min = 0.01))]
    pub amount: Option<Decimal>,
    #[validate(length(equal = 3))]
    pub currency_code: Option<String>,
    pub is_reconciled: Option<bool>,
    pub reconciliation_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub source_document_url: Option<String>,
    // updated_by will be derived from context
}