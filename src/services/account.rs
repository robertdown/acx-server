use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;

use crate::{
    error::AppError,
    models::{
        account::Account,
        dto::account_dto::{CreateAccountDto, UpdateAccountDto},
    },
};

/// Retrieves a list of accounts for a specific tenant.
pub async fn list_accounts(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Account>, AppError> {
    info!("Service: Listing accounts for tenant ID: {}", tenant_id);

    let accounts = query_as!(
        Account,
        r#"
        SELECT
            id, tenant_id, account_type_id, name, account_code, description,
            currency_code, is_active, created_at, created_by, updated_at, updated_by
        FROM accounts
        WHERE tenant_id = $1 AND is_active = TRUE
        ORDER BY name
        "#,
        tenant_id
    )
    .fetch_all(pool)
    .await?;

    Ok(accounts)
}

/// Retrieves a single account by ID for a specific tenant.
pub async fn get_account_by_id(
    pool: &PgPool,
    tenant_id: Uuid,
    account_id: Uuid,
) -> Result<Account, AppError> {
    info!("Service: Getting account with ID: {} for tenant ID: {}", account_id, tenant_id);

    let account = query_as!(
        Account,
        r#"
        SELECT
            id, tenant_id, account_type_id, name, account_code, description,
            currency_code, is_active, created_at, created_by, updated_at, updated_by
        FROM accounts
        WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE
        "#,
        account_id,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Account with ID {} not found for tenant {}", account_id, tenant_id)))?;

    Ok(account)
}

/// Creates a new account for a specific tenant.
pub async fn create_account(
    pool: &PgPool,
    tenant_id: Uuid,
    created_by_user_id: Uuid,
    dto: CreateAccountDto,
) -> Result<Account, AppError> {
    info!("Service: Creating new account for tenant ID {}", tenant_id);

    let new_account = query_as!(
        Account,
        r#"
        INSERT INTO accounts (
            tenant_id, account_type_id, name, account_code, description,
            currency_code, is_active, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, TRUE, $7, $7)
        RETURNING
            id, tenant_id, account_type_id, name, account_code, description,
            currency_code, is_active, created_at, created_by, updated_at, updated_by
        "#,
        tenant_id,
        dto.account_type_id,
        dto.name,
        dto.account_code,
        dto.description,
        dto.currency_code,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_account)
}

/// Updates an existing account for a specific tenant.
pub async fn update_account(
    pool: &PgPool,
    tenant_id: Uuid,
    account_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateAccountDto,
) -> Result<Account, AppError> {
    info!("Service: Updating account with ID: {} for tenant ID: {}", account_id, tenant_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(account_type_id) = dto.account_type_id {
        update_cols.push(format!("account_type_id = ${}", param_idx));
        update_values.push(Box::new(account_type_id));
        param_idx += 1;
    }
    if let Some(name) = dto.name {
        update_cols.push(format!("name = ${}", param_idx));
        update_values.push(Box::new(name));
        param_idx += 1;
    }
    if let Some(account_code) = dto.account_code {
        update_cols.push(format!("account_code = ${}", param_idx));
        update_values.push(Box::new(account_code));
        param_idx += 1;
    }
    if let Some(description) = dto.description {
        update_cols.push(format!("description = ${}", param_idx));
        update_values.push(Box::new(description));
        param_idx += 1;
    }
    if let Some(currency_code) = dto.currency_code {
        update_cols.push(format!("currency_code = ${}", param_idx));
        update_values.push(Box::new(currency_code));
        param_idx += 1;
    }
    if let Some(is_active) = dto.is_active {
        update_cols.push(format!("is_active = ${}", param_idx));
        update_values.push(Box::new(is_active));
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
        UPDATE accounts
        SET {}
        WHERE id = ${} AND tenant_id = ${}
        RETURNING
            id, tenant_id, account_type_id, name, account_code, description,
            currency_code, is_active, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx, param_idx + 1 // account_id and tenant_id will be the last parameters
    );

    let mut query = sqlx::query_as::<_, Account>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind account_id and tenant_id last
    query = query.bind(account_id);
    query = query.bind(tenant_id);

    let updated_account = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account with ID {} not found or not owned by tenant {}", account_id, tenant_id)))?;

    Ok(updated_account)
}

/// Deactivates an account (soft delete) for a specific tenant.
pub async fn deactivate_account(
    pool: &PgPool,
    tenant_id: Uuid,
    account_id: Uuid,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating account with ID: {} for tenant ID: {}", account_id, tenant_id);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE accounts
        SET
            is_active = FALSE,
            updated_at = NOW(),
            updated_by = $3
        WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE
        "#,
        account_id,
        tenant_id,
        updated_by_user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Account with ID {} not found or already inactive for tenant {}", account_id, tenant_id)));
    }

    Ok(())
}