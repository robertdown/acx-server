use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;
use rust_decimal::Decimal;

use crate::{
    error::AppError,
    models::{
        budget_line_item::BudgetLineItem,
        dto::budget_line_item_dto::{CreateBudgetLineItemDto, UpdateBudgetLineItemDto},
    },
};

/// Retrieves a list of budget line items for a specific budget.
pub async fn list_budget_line_items(
    pool: &PgPool,
    tenant_id: Uuid, // To verify budget ownership
    budget_id: Uuid,
) -> Result<Vec<BudgetLineItem>, AppError> {
    info!("Service: Listing budget line items for budget ID: {}", budget_id);

    // Verify the budget belongs to the tenant
    let budget_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM budgets WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
        budget_id,
        tenant_id
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    if !budget_exists {
        return Err(AppError::NotFound(format!("Budget with ID {} not found or inactive for tenant {}", budget_id, tenant_id)));
    }

    let line_items = query_as!(
        BudgetLineItem,
        r#"
        SELECT
            id, budget_id, category_id, account_id, budgeted_amount,
            is_active, created_at, created_by, updated_at, updated_by
        FROM budget_line_items
        WHERE budget_id = $1 AND is_active = TRUE
        ORDER BY category_id, account_id
        "#,
        budget_id
    )
    .fetch_all(pool)
    .await?;

    Ok(line_items)
}

/// Retrieves a single budget line item by ID for a specific budget and tenant.
pub async fn get_budget_line_item_by_id(
    pool: &PgPool,
    tenant_id: Uuid, // To verify budget ownership
    budget_line_item_id: Uuid,
) -> Result<BudgetLineItem, AppError> {
    info!("Service: Getting budget line item with ID: {}", budget_line_item_id);

    let line_item = query_as!(
        BudgetLineItem,
        r#"
        SELECT
            bli.id, bli.budget_id, bli.category_id, bli.account_id, bli.budgeted_amount,
            bli.is_active, bli.created_at, bli.created_by, bli.updated_at, bli.updated_by
        FROM budget_line_items bli
        JOIN budgets b ON bli.budget_id = b.id
        WHERE bli.id = $1 AND b.tenant_id = $2 AND bli.is_active = TRUE AND b.is_active = TRUE
        "#,
        budget_line_item_id,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Budget line item with ID {} not found for tenant {}", budget_line_item_id, tenant_id)))?;

    Ok(line_item)
}

/// Creates a new budget line item for a specific budget and tenant.
pub async fn create_budget_line_item(
    pool: &PgPool,
    tenant_id: Uuid, // For ownership verification of budget, category, and account
    created_by_user_id: Uuid,
    budget_id: Uuid,
    dto: CreateBudgetLineItemDto,
) -> Result<BudgetLineItem, AppError> {
    info!("Service: Creating new budget line item for budget ID {}", budget_id);

    // Verify the budget exists and belongs to the tenant
    let budget_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM budgets WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
        budget_id,
        tenant_id
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    if !budget_exists {
        return Err(AppError::NotFound(format!("Budget with ID {} not found or inactive for tenant {}", budget_id, tenant_id)));
    }

    // Verify category ownership (if provided)
    if let Some(category_id) = dto.category_id {
        let category_exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
            category_id, tenant_id
        )
        .fetch_one(pool)
        .await?
        .exists
        .unwrap_or(false);
        if !category_exists {
            return Err(AppError::ValidationError(format!("Category ID {} is invalid or inactive for tenant {}", category_id, tenant_id)));
        }
    }

    // Verify account ownership (if provided)
    if let Some(account_id) = dto.account_id {
        let account_exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
            account_id, tenant_id
        )
        .fetch_one(pool)
        .await?
        .exists
        .unwrap_or(false);
        if !account_exists {
            return Err(AppError::ValidationError(format!("Account ID {} is invalid or inactive for tenant {}", account_id, tenant_id)));
        }
    }

    let new_line_item = query_as!(
        BudgetLineItem,
        r#"
        INSERT INTO budget_line_items (
            budget_id, category_id, account_id, budgeted_amount,
            is_active, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, TRUE, $5, $5)
        RETURNING
            id, budget_id, category_id, account_id, budgeted_amount,
            is_active, created_at, created_by, updated_at, updated_by
        "#,
        budget_id,
        dto.category_id,
        dto.account_id,
        dto.budgeted_amount,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_line_item)
}

/// Updates an existing budget line item for a specific budget and tenant.
pub async fn update_budget_line_item(
    pool: &PgPool,
    tenant_id: Uuid, // For ownership verification of budget
    budget_line_item_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateBudgetLineItemDto,
) -> Result<BudgetLineItem, AppError> {
    info!("Service: Updating budget line item with ID: {}", budget_line_item_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(category_id) = dto.category_id {
        update_cols.push(format!("category_id = ${}", param_idx));
        update_values.push(Box::new(category_id));
        param_idx += 1;
        // Verify category ownership
        let category_exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
            category_id, tenant_id
        )
        .fetch_one(pool)
        .await?
        .exists
        .unwrap_or(false);
        if !category_exists {
            return Err(AppError::ValidationError(format!("Category ID {} is invalid or inactive for tenant {}", category_id, tenant_id)));
        }
    }
    if let Some(account_id) = dto.account_id {
        update_cols.push(format!("account_id = ${}", param_idx));
        update_values.push(Box::new(account_id));
        param_idx += 1;
        // Verify account ownership
        let account_exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE)",
            account_id, tenant_id
        )
        .fetch_one(pool)
        .await?
        .exists
        .unwrap_or(false);
        if !account_exists {
            return Err(AppError::ValidationError(format!("Account ID {} is invalid or inactive for tenant {}", account_id, tenant_id)));
        }
    }
    if let Some(budgeted_amount) = dto.budgeted_amount {
        update_cols.push(format!("budgeted_amount = ${}", param_idx));
        update_values.push(Box::new(budgeted_amount));
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
        UPDATE budget_line_items bli
        SET {}
        FROM budgets b
        WHERE bli.id = ${} AND bli.budget_id = b.id AND b.tenant_id = ${}
        RETURNING
            bli.id, bli.budget_id, bli.category_id, bli.account_id, bli.budgeted_amount,
            bli.is_active, bli.created_at, bli.created_by, bli.updated_at, bli.updated_by
        "#,
        update_clause, param_idx, param_idx + 1 // budget_line_item_id and tenant_id will be the last parameters
    );

    let mut query = sqlx::query_as::<_, BudgetLineItem>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind budget_line_item_id and tenant_id last
    query = query.bind(budget_line_item_id);
    query = query.bind(tenant_id);

    let updated_line_item = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Budget line item with ID {} not found or not owned by tenant {}", budget_line_item_id, tenant_id)))?;

    Ok(updated_line_item)
}

/// Deactivates a budget line item (soft delete) for a specific tenant.
pub async fn deactivate_budget_line_item(
    pool: &PgPool,
    tenant_id: Uuid, // To verify budget ownership
    budget_line_item_id: Uuid,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating budget line item with ID: {}", budget_line_item_id);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE budget_line_items bli
        SET
            is_active = FALSE,
            updated_at = NOW(),
            updated_by = $3
        FROM budgets b
        WHERE bli.id = $1 AND bli.budget_id = b.id AND b.tenant_id = $2 AND bli.is_active = TRUE
        "#,
        budget_line_item_id,
        tenant_id,
        updated_by_user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Budget line item with ID {} not found or already inactive for tenant {}", budget_line_item_id, tenant_id)));
    }

    Ok(())
}