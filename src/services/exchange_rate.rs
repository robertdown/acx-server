use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;
use chrono::NaiveDate;

use crate::{
    error::AppError,
    models::{
        exchange_rate::ExchangeRate,
        dto::exchange_rate_dto::{CreateExchangeRateDto, UpdateExchangeRateDto},
    },
};
use rust_decimal::Decimal;


/// Retrieves a list of exchange rates for a given tenant or system-wide.
pub async fn list_exchange_rates(pool: &PgPool, tenant_id: Option<Uuid>) -> Result<Vec<ExchangeRate>, AppError> {
    info!("Service: Listing exchange rates for tenant ID: {:?}", tenant_id);

    let rates = query_as!(
        ExchangeRate,
        r#"
        SELECT
            id, tenant_id, base_currency_code, target_currency_code, rate, rate_date,
            source, created_at, created_by, updated_at, updated_by
        FROM exchange_rates
        WHERE
            ($1::uuid IS NULL AND tenant_id IS NULL) OR
            ($1::uuid IS NOT NULL AND tenant_id = $1)
        ORDER BY rate_date DESC, base_currency_code, target_currency_code
        "#,
        tenant_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rates)
}

/// Retrieves a single exchange rate by ID.
pub async fn get_exchange_rate_by_id(pool: &PgPool, rate_id: Uuid) -> Result<ExchangeRate, AppError> {
    info!("Service: Getting exchange rate with ID: {}", rate_id);

    let rate = query_as!(
        ExchangeRate,
        r#"
        SELECT
            id, tenant_id, base_currency_code, target_currency_code, rate, rate_date,
            source, created_at, created_by, updated_at, updated_by
        FROM exchange_rates
        WHERE id = $1
        "#,
        rate_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Exchange rate with ID {} not found", rate_id)))?;

    Ok(rate)
}

/// Retrieves the latest exchange rate for a given currency pair, optionally tenant-specific.
pub async fn get_latest_exchange_rate(
    pool: &PgPool,
    tenant_id: Option<Uuid>,
    base_currency_code: &str,
    target_currency_code: &str,
) -> Result<ExchangeRate, AppError> {
    info!(
        "Service: Getting latest exchange rate for tenant {:?}, base: {}, target: {}",
        tenant_id, base_currency_code, target_currency_code
    );

    let rate = query_as!(
        ExchangeRate,
        r#"
        SELECT
            id, tenant_id, base_currency_code, target_currency_code, rate, rate_date,
            source, created_at, created_by, updated_at, updated_by
        FROM exchange_rates
        WHERE
            ($1::uuid IS NULL AND tenant_id IS NULL) OR
            ($1::uuid IS NOT NULL AND tenant_id = $1)
            AND base_currency_code = $2
            AND target_currency_code = $3
        ORDER BY rate_date DESC, created_at DESC
        LIMIT 1
        "#,
        tenant_id,
        base_currency_code,
        target_currency_code
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(format!(
            "No exchange rate found for tenant {:?}, base {} to target {}",
            tenant_id, base_currency_code, target_currency_code
        ))
    })?;

    Ok(rate)
}


/// Creates a new exchange rate.
pub async fn create_exchange_rate(
    pool: &PgPool,
    created_by_user_id: Uuid,
    dto: CreateExchangeRateDto,
) -> Result<ExchangeRate, AppError> {
    info!("Service: Creating new exchange rate.");

    let new_rate = query_as!(
        ExchangeRate,
        r#"
        INSERT INTO exchange_rates (
            tenant_id, base_currency_code, target_currency_code, rate, rate_date,
            source, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        RETURNING
            id, tenant_id, base_currency_code, target_currency_code, rate, rate_date,
            source, created_at, created_by, updated_at, updated_by
        "#,
        dto.tenant_id,
        dto.base_currency_code,
        dto.target_currency_code,
        dto.rate,
        dto.rate_date,
        dto.source,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_rate)
}

/// Updates an existing exchange rate.
pub async fn update_exchange_rate(
    pool: &PgPool,
    rate_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateExchangeRateDto,
) -> Result<ExchangeRate, AppError> {
    info!("Service: Updating exchange rate with ID: {}", rate_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(rate) = dto.rate {
        update_cols.push(format!("rate = ${}", param_idx));
        update_values.push(Box::new(rate));
        param_idx += 1;
    }
    if let Some(rate_date) = dto.rate_date {
        update_cols.push(format!("rate_date = ${}", param_idx));
        update_values.push(Box::new(rate_date));
        param_idx += 1;
    }
    if let Some(source) = dto.source {
        update_cols.push(format!("source = ${}", param_idx));
        update_values.push(Box::new(source));
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
        UPDATE exchange_rates
        SET {}
        WHERE id = ${}
        RETURNING
            id, tenant_id, base_currency_code, target_currency_code, rate, rate_date,
            source, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx // rate_id will be the last parameter
    );

    let mut query = sqlx::query_as::<_, ExchangeRate>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind rate_id last
    query = query.bind(rate_id);

    let updated_rate = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Exchange rate with ID {} not found", rate_id)))?;

    Ok(updated_rate)
}

/// Deletes an exchange rate. (Soft delete not applicable here, as rates are historical data)
pub async fn delete_exchange_rate(pool: &PgPool, rate_id: Uuid) -> Result<(), AppError> {
    info!("Service: Deleting exchange rate with ID: {}", rate_id);

    let affected_rows = sqlx::query!(
        r#"
        DELETE FROM exchange_rates
        WHERE id = $1
        "#,
        rate_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Exchange rate with ID {} not found", rate_id)));
    }

    Ok(())
}