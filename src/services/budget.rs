use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    error::AppError,
    models::{
        budget::Budget,
        dto::budget_dto::{CreateBudgetDto, UpdateBudgetDto},
    },
};

/// Retrieves a list of budgets for a specific tenant.
pub async fn list_budgets(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Budget>, AppError> {
    info!("Service: Listing budgets for tenant ID: {}", tenant_id);

    let budgets = query_as!(
        Budget,
        r#"
        SELECT
            id, tenant_id, name, start_date, end_date, currency_code,
            is_active, created_at, created_by, updated_at, updated_by
        FROM budgets
        WHERE tenant_id = $1 AND is_active = TRUE
        ORDER BY start_date DESC, name
        "#,
        tenant_id
    )
    .fetch_all(pool)
    .await?;

    Ok(budgets)
}

/// Retrieves a single budget by ID for a specific tenant.
pub async fn get_budget_by_id(
    pool: &PgPool,
    tenant_id: Uuid,
    budget_id: Uuid,
) -> Result<Budget, AppError> {
    info!("Service: Getting budget with ID: {} for tenant ID: {}", budget_id, tenant_id);

    let budget = query_as!(
        Budget,
        r#"
        SELECT
            id, tenant_id, name, start_date, end_date, currency_code,
            is_active, created_at, created_by, updated_at, updated_by
        FROM budgets
        WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE
        "#,
        budget_id,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Budget with ID {} not found for tenant {}", budget_id, tenant_id)))?;

    Ok(budget)
}

/// Creates a new budget for a specific tenant.
pub async fn create_budget(
    pool: &PgPool,
    tenant_id: Uuid,
    created_by_user_id: Uuid,
    dto: CreateBudgetDto,
) -> Result<Budget, AppError> {
    info!("Service: Creating new budget '{}' for tenant ID {}", dto.name, tenant_id);

    // Basic validation: Ensure end_date is not before start_date
    if dto.end_date < dto.start_date {
        return Err(AppError::BadRequest("End date cannot be before start date".to_string()));
    }

    let new_budget = query_as!(
        Budget,
        r#"
        INSERT INTO budgets (
            tenant_id, name, start_date, end_date, currency_code,
            is_active, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, TRUE, $6, $6)
        RETURNING
            id, tenant_id, name, start_date, end_date, currency_code,
            is_active, created_at, created_by, updated_at, updated_by
        "#,
        tenant_id,
        dto.name,
        dto.start_date,
        dto.end_date,
        dto.currency_code,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_budget)
}

/// Updates an existing budget for a specific tenant.
pub async fn update_budget(
    pool: &PgPool,
    tenant_id: Uuid,
    budget_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateBudgetDto,
) -> Result<Budget, AppError> {
    info!("Service: Updating budget with ID: {} for tenant ID: {}", budget_id, tenant_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(name) = dto.name {
        update_cols.push(format!("name = ${}", param_idx));
        update_values.push(Box::new(name));
        param_idx += 1;
    }
    if let Some(start_date) = dto.start_date {
        update_cols.push(format!("start_date = ${}", param_idx));
        update_values.push(Box::new(start_date));
        param_idx += 1;
    }
    if let Some(end_date) = dto.end_date {
        update_cols.push(format!("end_date = ${}", param_idx));
        update_values.push(Box::new(end_date));
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

    // Check for date consistency if both are provided or updated
    if let (Some(start), Some(end)) = (dto.start_date, dto.end_date) {
        if end < start {
            return Err(AppError::BadRequest("Updated end date cannot be before updated start date".to_string()));
        }
    } else if dto.start_date.is_some() || dto.end_date.is_some() {
        // If only one date is updated, fetch current values to validate
        let current_budget = get_budget_by_id(pool, tenant_id, budget_id).await?;
        let effective_start_date = dto.start_date.unwrap_or(current_budget.start_date);
        let effective_end_date = dto.end_date.unwrap_or(current_budget.end_date);
        if effective_end_date < effective_start_date {
            return Err(AppError::BadRequest("Resulting end date cannot be before resulting start date".to_string()));
        }
    }


    let update_clause = update_cols.join(", ");
    let query_str = format!(
        r#"
        UPDATE budgets
        SET {}
        WHERE id = ${} AND tenant_id = ${}
        RETURNING
            id, tenant_id, name, start_date, end_date, currency_code,
            is_active, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx, param_idx + 1 // budget_id and tenant_id will be the last parameters
    );

    let mut query = sqlx::query_as::<_, Budget>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind budget_id and tenant_id last
    query = query.bind(budget_id);
    query = query.bind(tenant_id);

    let updated_budget = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Budget with ID {} not found or not owned by tenant {}", budget_id, tenant_id)))?;

    Ok(updated_budget)
}

/// Deactivates a budget (soft delete) for a specific tenant.
/// Note: This does not cascade to budget line items; they remain associated but implicitly inactive.
pub async fn deactivate_budget(
    pool: &PgPool,
    tenant_id: Uuid,
    budget_id: Uuid,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating budget with ID: {} for tenant ID: {}", budget_id, tenant_id);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE budgets
        SET
            is_active = FALSE,
            updated_at = NOW(),
            updated_by = $3
        WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE
        "#,
        budget_id,
        tenant_id,
        updated_by_user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Budget with ID {} not found or already inactive for tenant {}", budget_id, tenant_id)));
    }

    Ok(())
}