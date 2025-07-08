use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;
use chrono::Utc;

use crate::{
    error::AppError,
    models::{
        currency::Currency,
        dto::currency_dto::{CreateCurrencyDto, UpdateCurrencyDto},
    },
};

/// Retrieves a list of all active currencies.
pub async fn list_currencies(pool: &PgPool) -> Result<Vec<Currency>, AppError> {
    info!("Service: Listing all active currencies.");

    let currencies = query_as!(
        Currency,
        r#"
        SELECT
            code, name, symbol, is_active,
            created_at, created_by, updated_at, updated_by
        FROM currencies
        WHERE is_active = TRUE
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(currencies)
}

/// Retrieves a single currency by its code.
pub async fn get_currency_by_code(pool: &PgPool, code: &str) -> Result<Currency, AppError> {
    info!("Service: Getting currency with code: {}", code);

    let currency = query_as!(
        Currency,
        r#"
        SELECT
            code, name, symbol, is_active,
            created_at, created_by, updated_at, updated_by
        FROM currencies
        WHERE code = $1 AND is_active = TRUE
        "#,
        code
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Currency with code {} not found", code)))?;

    Ok(currency)
}

/// Creates a new currency.
/// `created_by_user_id` should come from an authenticated system administrator.
pub async fn create_currency(
    pool: &PgPool,
    created_by_user_id: Uuid,
    dto: CreateCurrencyDto,
) -> Result<Currency, AppError> {
    info!("Service: Creating new currency with code: {}", dto.code);

    let new_currency = query_as!(
        Currency,
        r#"
        INSERT INTO currencies (
            code, name, symbol, is_active, created_by, updated_by
        )
        VALUES ($1, $2, $3, TRUE, $4, $4)
        RETURNING
            code, name, symbol, is_active, created_at, created_by, updated_at, updated_by
        "#,
        dto.code,
        dto.name,
        dto.symbol,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_currency)
}

/// Updates an existing currency.
/// `updated_by_user_id` should come from an authenticated system administrator.
pub async fn update_currency(
    pool: &PgPool,
    code: &str,
    updated_by_user_id: Uuid,
    dto: UpdateCurrencyDto,
) -> Result<Currency, AppError> {
    info!("Service: Updating currency with code: {}", code);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(name) = dto.name {
        update_cols.push(format!("name = ${}", param_idx));
        update_values.push(Box::new(name));
        param_idx += 1;
    }
    if let Some(symbol) = dto.symbol {
        update_cols.push(format!("symbol = ${}", param_idx));
        update_values.push(Box::new(symbol));
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
        UPDATE currencies
        SET {}
        WHERE code = ${}
        RETURNING
            code, name, symbol, is_active, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx // code will be the last parameter
    );

    let mut query = sqlx::query_as::<_, Currency>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind code last
    query = query.bind(code.to_string()); // Bind String explicitly

    let updated_currency = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Currency with code {} not found", code)))?;

    Ok(updated_currency)
}

/// Deactivates a currency (soft delete).
/// `updated_by_user_id` should come from an authenticated system administrator.
pub async fn deactivate_currency(
    pool: &PgPool,
    code: &str,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating currency with code: {}", code);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE currencies
        SET
            is_active = FALSE,
            updated_at = NOW(),
            updated_by = $2
        WHERE code = $1 AND is_active = TRUE
        "#,
        code,
        updated_by_user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Currency with code {} not found or already inactive", code)));
    }

    Ok(())
}