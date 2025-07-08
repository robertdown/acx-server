use sqlx::{query_as, PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;
use tracing::info;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;

use crate::{
    error::AppError,
    models::{
        transaction::{Transaction, TransactionType},
        journal_entry::{JournalEntry, JournalEntryType}, // Assuming JournalEntry and its DTOs are defined
        dto::transaction_dto::{CreateTransactionDto, UpdateTransactionDto},
        dto::journal_entry_dto::{CreateJournalEntryDto}, // Assuming CreateJournalEntryDto is defined
    },
};

/// Retrieves a list of transactions for a specific tenant.
pub async fn list_transactions(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Transaction>, AppError> {
    info!("Service: Listing transactions for tenant ID: {}", tenant_id);

    let transactions = query_as!(
        Transaction,
        r#"
        SELECT
            id, tenant_id, transaction_date, description, type as "r#type!: TransactionType",
            category_id, tags_json, amount, currency_code, is_reconciled, reconciliation_date,
            notes, source_document_url, created_at, created_by, updated_at, updated_by
        FROM transactions
        WHERE tenant_id = $1
        ORDER BY transaction_date DESC, created_at DESC
        "#,
        tenant_id
    )
    .fetch_all(pool)
    .await?;

    Ok(transactions)
}

/// Retrieves a single transaction by ID for a specific tenant.
pub async fn get_transaction_by_id(
    pool: &PgPool,
    tenant_id: Uuid,
    transaction_id: Uuid,
) -> Result<Transaction, AppError> {
    info!("Service: Getting transaction with ID: {} for tenant ID: {}", transaction_id, tenant_id);

    let transaction = query_as!(
        Transaction,
        r#"
        SELECT
            id, tenant_id, transaction_date, description, type as "r#type!: TransactionType",
            category_id, tags_json, amount, currency_code, is_reconciled, reconciliation_date,
            notes, source_document_url, created_at, created_by, updated_at, updated_by
        FROM transactions
        WHERE id = $1 AND tenant_id = $2
        "#,
        transaction_id,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Transaction with ID {} not found for tenant {}", transaction_id, tenant_id)))?;

    Ok(transaction)
}

/// Creates a new transaction along with its associated journal entries.
/// This operation is wrapped in a database transaction to ensure atomicity.
pub async fn create_transaction(
    pool: &PgPool,
    tenant_id: Uuid,
    created_by_user_id: Uuid,
    dto: CreateTransactionDto,
) -> Result<Transaction, AppError> {
    info!("Service: Creating new transaction for tenant ID {}", tenant_id);

    // Start a database transaction
    let mut db_tx = pool.begin().await?;

    // --- 1. Create the main transaction record ---
    let tags_json: Option<JsonValue> = if let Some(tags) = dto.tags {
        Some(serde_json::to_value(&tags).map_err(|e| AppError::InternalError(format!("Failed to serialize tags: {}", e)))?)
    } else {
        None
    };

    let new_transaction = query_as!(
        Transaction,
        r#"
        INSERT INTO transactions (
            tenant_id, transaction_date, description, type, category_id,
            tags_json, amount, currency_code, is_reconciled, reconciliation_date,
            notes, source_document_url, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
        RETURNING
            id, tenant_id, transaction_date, description, type as "r#type!: TransactionType", category_id,
            tags_json, amount, currency_code, is_reconciled, reconciliation_date,
            notes, source_document_url, created_at, created_by, updated_at, updated_by
        "#,
        tenant_id,
        dto.transaction_date,
        dto.description,
        dto.r#type as TransactionType, // Cast enum to string for DB
        dto.category_id,
        tags_json,
        dto.amount,
        dto.currency_code,
        dto.is_reconciled.unwrap_or(false), // Default to false if not provided
        dto.reconciliation_date,
        dto.notes,
        dto.source_document_url,
        created_by_user_id,
    )
    .fetch_one(&mut *db_tx) // Use the database transaction
    .await?;

    // --- 2. Create associated journal entries ---
    // For simplicity, this example assumes journal entries are provided directly.
    // In a real accounting system, for 'INCOME', 'EXPENSE', 'TRANSFER' types,
    // the journal entries might be auto-generated based on the transaction type
    // and the primary account involved, with only one side provided by the user.
    // For 'JOURNAL_ENTRY' type, both sides would be explicitly provided.
    // This boilerplate supports explicit provision for now.
    for entry_dto in dto.journal_entries {
        // Basic validation: Ensure account exists and is valid for tenant
        let account_exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
            entry_dto.account_id, tenant_id
        )
        .fetch_one(&mut *db_tx)
        .await?
        .exists
        .unwrap_or(false);

        if !account_exists {
            db_tx.rollback().await?; // Rollback if any account is invalid
            return Err(AppError::ValidationError(format!("Account ID {} is invalid or inactive for tenant {}", entry_dto.account_id, tenant_id)));
        }

        sqlx::query!(
            r#"
            INSERT INTO journal_entries (
                transaction_id, account_id, entry_type, amount, currency_code,
                exchange_rate, converted_amount, memo, created_by, updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            "#,
            new_transaction.id,
            entry_dto.account_id,
            entry_dto.entry_type as JournalEntryType, // Cast enum to string for DB
            entry_dto.amount,
            entry_dto.currency_code,
            entry_dto.exchange_rate,
            entry_dto.converted_amount,
            entry_dto.memo,
            created_by_user_id,
        )
        .execute(&mut *db_tx) // Use the database transaction
        .await?;
    }

    // --- 3. Commit the transaction ---
    db_tx.commit().await?;

    Ok(new_transaction)
}

/// Updates an existing transaction for a specific tenant.
/// Note: Updating a transaction, especially its amount or type, often requires
/// complex logic to adjust or reverse associated journal entries.
/// This implementation provides a basic update for metadata.
pub async fn update_transaction(
    pool: &PgPool,
    tenant_id: Uuid,
    transaction_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateTransactionDto,
) -> Result<Transaction, AppError> {
    info!("Service: Updating transaction with ID: {} for tenant ID: {}", transaction_id, tenant_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(transaction_date) = dto.transaction_date {
        update_cols.push(format!("transaction_date = ${}", param_idx));
        update_values.push(Box::new(transaction_date));
        param_idx += 1;
    }
    if let Some(description) = dto.description {
        update_cols.push(format!("description = ${}", param_idx));
        update_values.push(Box::new(description));
        param_idx += 1;
    }
    if let Some(r#type) = dto.r#type {
        update_cols.push(format!("type = ${}", param_idx));
        update_values.push(Box::new(r#type as TransactionType));
        param_idx += 1;
    }
    if let Some(category_id) = dto.category_id {
        update_cols.push(format!("category_id = ${}", param_idx));
        update_values.push(Box::new(category_id));
        param_idx += 1;
    }
    if let Some(tags) = dto.tags {
        let tags_json = serde_json::to_value(&tags).map_err(|e| AppError::InternalError(format!("Failed to serialize tags: {}", e)))?;
        update_cols.push(format!("tags_json = ${}", param_idx));
        update_values.push(Box::new(tags_json));
        param_idx += 1;
    }
    if let Some(amount) = dto.amount {
        update_cols.push(format!("amount = ${}", param_idx));
        update_values.push(Box::new(amount));
        param_idx += 1;
    }
    if let Some(currency_code) = dto.currency_code {
        update_cols.push(format!("currency_code = ${}", param_idx));
        update_values.push(Box::new(currency_code));
        param_idx += 1;
    }
    if let Some(is_reconciled) = dto.is_reconciled {
        update_cols.push(format!("is_reconciled = ${}", param_idx));
        update_values.push(Box::new(is_reconciled));
        param_idx += 1;
    }
    if let Some(reconciliation_date) = dto.reconciliation_date {
        update_cols.push(format!("reconciliation_date = ${}", param_idx));
        update_values.push(Box::new(reconciliation_date));
        param_idx += 1;
    }
    if let Some(notes) = dto.notes {
        update_cols.push(format!("notes = ${}", param_idx));
        update_values.push(Box::new(notes));
        param_idx += 1;
    }
    if let Some(source_document_url) = dto.source_document_url {
        update_cols.push(format!("source_document_url = ${}", param_idx));
        update_values.push(Box::new(source_document_url));
        param_idx += 1;
    }

    // Always update updated_at and updated_by
    update_cols.push(format!("updated_at = NOW()"));
    update_cols.push(format!("updated_by = ${}", param_idx));
    update_values.push(Box::new(updated_by_user_id));
    param_idx += 1;

    if update_cols.is_empty() {
        return Err(AppError::BadRequest("No fields provided for update".to_string()));
    }

    let update_clause = update_cols.join(", ");
    let query_str = format!(
        r#"
        UPDATE transactions
        SET {}
        WHERE id = ${} AND tenant_id = ${}
        RETURNING
            id, tenant_id, transaction_date, description, type as "r#type!: TransactionType",
            category_id, tags_json, amount, currency_code, is_reconciled, reconciliation_date,
            notes, source_document_url, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx, param_idx + 1 // transaction_id and tenant_id will be the last parameters
    );

    let mut query = sqlx::query_as::<_, Transaction>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind transaction_id and tenant_id last
    query = query.bind(transaction_id);
    query = query.bind(tenant_id);

    let updated_transaction = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Transaction with ID {} not found or not owned by tenant {}", transaction_id, tenant_id)))?;

    Ok(updated_transaction)
}

/// Deletes a transaction by ID for a specific tenant.
/// Note: Deleting a transaction requires also deleting its associated journal entries
/// to maintain data integrity. This operation is wrapped in a database transaction.
pub async fn delete_transaction(
    pool: &PgPool,
    tenant_id: Uuid,
    transaction_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deleting transaction with ID: {} for tenant ID: {}", transaction_id, tenant_id);

    let mut db_tx = pool.begin().await?;

    // First, delete associated journal entries
    let journal_entries_deleted = sqlx::query!(
        r#"
        DELETE FROM journal_entries
        WHERE transaction_id = $1
        "#,
        transaction_id
    )
    .execute(&mut *db_tx)
    .await?
    .rows_affected();

    info!("Deleted {} journal entries for transaction {}", journal_entries_deleted, transaction_id);

    // Then, delete the transaction itself
    let transaction_deleted = sqlx::query!(
        r#"
        DELETE FROM transactions
        WHERE id = $1 AND tenant_id = $2
        "#,
        transaction_id,
        tenant_id
    )
    .execute(&mut *db_tx)
    .await?
    .rows_affected();

    if transaction_deleted == 0 {
        db_tx.rollback().await?; // Rollback if transaction not found/deleted
        return Err(AppError::NotFound(format!("Transaction with ID {} not found or not owned by tenant {}", transaction_id, tenant_id)));
    }

    db_tx.commit().await?; // Commit if both deletions are successful

    Ok(())
}