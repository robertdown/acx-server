use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;

use crate::{
    error::AppError,
    models::{
        account_type::{AccountType, AccountNormalBalance},
        dto::account_type_dto::{CreateAccountTypeDto, UpdateAccountTypeDto},
    },
};

/// Retrieves a list of all active account types.
pub async fn list_account_types(pool: &PgPool) -> Result<Vec<AccountType>, AppError> {
    info!("Service: Listing all active account types.");

    let account_types = query_as!(
        AccountType,
        r#"
        SELECT
            id, name, normal_balance as "normal_balance!: AccountNormalBalance", -- Explicit cast for enum
            is_active, created_at, created_by, updated_at, updated_by
        FROM account_types
        WHERE is_active = TRUE
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(account_types)
}

/// Retrieves a single account type by ID.
pub async fn get_account_type_by_id(pool: &PgPool, account_type_id: Uuid) -> Result<AccountType, AppError> {
    info!("Service: Getting account type with ID: {}", account_type_id);

    let account_type = query_as!(
        AccountType,
        r#"
        SELECT
            id, name, normal_balance as "normal_balance!: AccountNormalBalance",
            is_active, created_at, created_by, updated_at, updated_by
        FROM account_types
        WHERE id = $1 AND is_active = TRUE
        "#,
        account_type_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Account type with ID {} not found", account_type_id)))?;

    Ok(account_type)
}

/// Creates a new account type.
/// `created_by_user_id` should come from an authenticated system administrator.
pub async fn create_account_type(
    pool: &PgPool,
    created_by_user_id: Uuid,
    dto: CreateAccountTypeDto,
) -> Result<AccountType, AppError> {
    info!("Service: Creating new account type with name: {}", dto.name);

    let new_account_type = query_as!(
        AccountType,
        r#"
        INSERT INTO account_types (
            name, normal_balance, is_active, created_by, updated_by
        )
        VALUES ($1, $2, TRUE, $3, $3)
        RETURNING
            id, name, normal_balance as "normal_balance!: AccountNormalBalance",
            is_active, created_at, created_by, updated_at, updated_by
        "#,
        dto.name,
        dto.normal_balance as AccountNormalBalance, // Cast to enum for query
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_account_type)
}

/// Updates an existing account type.
/// `updated_by_user_id` should come from an authenticated system administrator.
pub async fn update_account_type(
    pool: &PgPool,
    account_type_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateAccountTypeDto,
) -> Result<AccountType, AppError> {
    info!("Service: Updating account type with ID: {}", account_type_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(name) = dto.name {
        update_cols.push(format!("name = ${}", param_idx));
        update_values.push(Box::new(name));
        param_idx += 1;
    }
    if let Some(normal_balance) = dto.normal_balance {
        update_cols.push(format!("normal_balance = ${}", param_idx));
        update_values.push(Box::new(normal_balance as AccountNormalBalance)); // Cast enum for binding
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
        UPDATE account_types
        SET {}
        WHERE id = ${}
        RETURNING
            id, name, normal_balance as "normal_balance!: AccountNormalBalance",
            is_active, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx // account_type_id will be the last parameter
    );

    let mut query = sqlx::query_as::<_, AccountType>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind account_type_id last
    query = query.bind(account_type_id);

    let updated_account_type = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account type with ID {} not found", account_type_id)))?;

    Ok(updated_account_type)
}

/// Deactivates an account type (soft delete).
/// `updated_by_user_id` should come from an authenticated system administrator.
pub async fn deactivate_account_type(
    pool: &PgPool,
    account_type_id: Uuid,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating account type with ID: {}", account_type_id);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE account_types
        SET
            is_active = FALSE,
            updated_at = NOW(),
            updated_by = $2
        WHERE id = $1 AND is_active = TRUE
        "#,
        account_type_id,
        updated_by_user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Account type with ID {} not found or already inactive", account_type_id)));
    }

    Ok(())
}