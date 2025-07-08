use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;
use rust_decimal::Decimal;

use crate::{
    error::AppError,
    models::{
        journal_entry::{JournalEntry, JournalEntryType},
        dto::journal_entry_dto::{CreateJournalEntryDto, UpdateJournalEntryDto},
    },
};

/// Retrieves a list of journal entries for a specific transaction.
pub async fn list_journal_entries_for_transaction(
    pool: &PgPool,
    tenant_id: Uuid, // Used to verify transaction ownership
    transaction_id: Uuid,
) -> Result<Vec<JournalEntry>, AppError> {
    info!("Service: Listing journal entries for transaction ID: {}", transaction_id);

    // First, verify the transaction belongs to the tenant
    let transaction_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM transactions WHERE id = $1 AND tenant_id = $2)",
        transaction_id,
        tenant_id
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    if !transaction_exists {
        return Err(AppError::NotFound(format!("Transaction with ID {} not found for tenant {}", transaction_id, tenant_id)));
    }

    let entries = query_as!(
        JournalEntry,
        r#"
        SELECT
            id, transaction_id, account_id, entry_type as "entry_type!: JournalEntryType",
            amount, currency_code, exchange_rate, converted_amount, memo,
            created_at, created_by, updated_at, updated_by
        FROM journal_entries
        WHERE transaction_id = $1
        ORDER BY created_at
        "#,
        transaction_id
    )
    .fetch_all(pool)
    .await?;

    Ok(entries)
}

/// Retrieves a single journal entry by ID.
pub async fn get_journal_entry_by_id(
    pool: &PgPool,
    tenant_id: Uuid, // Used to verify transaction ownership
    journal_entry_id: Uuid,
) -> Result<JournalEntry, AppError> {
    info!("Service: Getting journal entry with ID: {}", journal_entry_id);

    let entry = query_as!(
        JournalEntry,
        r#"
        SELECT
            je.id, je.transaction_id, je.account_id, je.entry_type as "entry_type!: JournalEntryType",
            je.amount, je.currency_code, je.exchange_rate, je.converted_amount, je.memo,
            je.created_at, je.created_by, je.updated_at, je.updated_by
        FROM journal_entries je
        JOIN transactions t ON je.transaction_id = t.id
        WHERE je.id = $1 AND t.tenant_id = $2
        "#,
        journal_entry_id,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Journal entry with ID {} not found for tenant {}", journal_entry_id, tenant_id)))?;

    Ok(entry)
}

/// Creates a new journal entry.
/// This is typically called internally by transaction creation, but exposed here for direct use.
pub async fn create_journal_entry(
    pool: &PgPool,
    tenant_id: Uuid, // Used to verify transaction ownership and account ownership
    created_by_user_id: Uuid,
    transaction_id: Uuid, // The transaction this entry belongs to
    dto: CreateJournalEntryDto,
) -> Result<JournalEntry, AppError> {
    info!("Service: Creating new journal entry for transaction ID: {}", transaction_id);

    // Verify transaction exists and belongs to tenant
    let transaction_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM transactions WHERE id = $1 AND tenant_id = $2)",
        transaction_id,
        tenant_id
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    if !transaction_exists {
        return Err(AppError::NotFound(format!("Transaction with ID {} not found for tenant {}", transaction_id, tenant_id)));
    }

    // Verify account exists and belongs to tenant
    let account_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
        dto.account_id, tenant_id
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    if !account_exists {
        return Err(AppError::ValidationError(format!("Account ID {} is invalid or inactive for tenant {}", dto.account_id, tenant_id)));
    }

    let new_entry = query_as!(
        JournalEntry,
        r#"
        INSERT INTO journal_entries (
            transaction_id, account_id, entry_type, amount, currency_code,
            exchange_rate, converted_amount, memo, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        RETURNING
            id, transaction_id, account_id, entry_type as "entry_type!: JournalEntryType",
            amount, currency_code, exchange_rate, converted_amount, memo,
            created_at, created_by, updated_at, updated_by
        "#,
        transaction_id,
        dto.account_id,
        dto.entry_type as JournalEntryType,
        dto.amount,
        dto.currency_code,
        dto.exchange_rate,
        dto.converted_amount,
        dto.memo,
        created_by_user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(new_entry)
}

/// Updates an existing journal entry.
/// Note: Changing core financial aspects of a journal entry for a posted transaction
/// might require creating an adjusting entry rather than directly modifying it.
/// This service allows modification of memo, exchange_rate, converted_amount.
pub async fn update_journal_entry(
    pool: &PgPool,
    tenant_id: Uuid, // Used to verify transaction ownership
    journal_entry_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateJournalEntryDto,
) -> Result<JournalEntry, AppError> {
    info!("Service: Updating journal entry with ID: {}", journal_entry_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    // Only allow updating certain fields (e.g., memo, exchange_rate, converted_amount)
    // Changing account_id, entry_type, amount would typically require new adjusting entries
    // or a full transaction reversal/re-creation in a robust accounting system.
    if let Some(memo) = dto.memo {
        update_cols.push(format!("memo = ${}", param_idx));
        update_values.push(Box::new(memo));
        param_idx += 1;
    }
    if let Some(exchange_rate) = dto.exchange_rate {
        update_cols.push(format!("exchange_rate = ${}", param_idx));
        update_values.push(Box::new(exchange_rate));
        param_idx += 1;
    }
    if let Some(converted_amount) = dto.converted_amount {
        update_cols.push(format!("converted_amount = ${}", param_idx));
        update_values.push(Box::new(converted_amount));
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
        UPDATE journal_entries je
        SET {}
        FROM transactions t
        WHERE je.id = ${} AND je.transaction_id = t.id AND t.tenant_id = ${}
        RETURNING
            je.id, je.transaction_id, je.account_id, je.entry_type as "entry_type!: JournalEntryType",
            je.amount, je.currency_code, je.exchange_rate, je.converted_amount, je.memo,
            je.created_at, je.created_by, je.updated_at, je.updated_by
        "#,
        update_clause, param_idx, param_idx + 1 // journal_entry_id and tenant_id will be the last parameters
    );

    let mut query = sqlx::query_as::<_, JournalEntry>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind journal_entry_id and tenant_id last
    query = query.bind(journal_entry_id);
    query = query.bind(tenant_id);

    let updated_entry = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Journal entry with ID {} not found or not owned by tenant {}", journal_entry_id, tenant_id)))?;

    Ok(updated_entry)
}

/// Deletes a journal entry.
/// Note: Deleting a journal entry directly can break the double-entry balance of its parent transaction.
/// This operation should be used with extreme caution, typically only for draft transactions,
/// or as part of a larger transaction modification/reversal logic.
pub async fn delete_journal_entry(
    pool: &PgPool,
    tenant_id: Uuid, // Used to verify transaction ownership
    journal_entry_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deleting journal entry with ID: {}", journal_entry_id);

    let affected_rows = sqlx::query!(
        r#"
        DELETE FROM journal_entries je
        USING transactions t
        WHERE je.id = $1 AND je.transaction_id = t.id AND t.tenant_id = $2
        "#,
        journal_entry_id,
        tenant_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Journal entry with ID {} not found or not owned by tenant {}", journal_entry_id, tenant_id)));
    }

    Ok(())
}