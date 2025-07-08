use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;

use crate::{
    error::AppError,
    models::{
        category::{Category, CategoryType},
        dto::category_dto::{CreateCategoryDto, UpdateCategoryDto},
    },
};

/// Retrieves a list of categories for a specific tenant.
pub async fn list_categories(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Category>, AppError> {
    info!("Service: Listing categories for tenant ID: {}", tenant_id);

    let categories = query_as!(
        Category,
        r#"
        SELECT
            id, tenant_id, name, description, type as "r#type!: CategoryType", -- Cast for enum
            parent_category_id, is_active, created_at, created_by, updated_at, updated_by
        FROM categories
        WHERE tenant_id = $1 AND is_active = TRUE
        ORDER BY name
        "#,
        tenant_id
    )
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

/// Retrieves a single category by ID for a specific tenant.
pub async fn get_category_by_id(
    pool: &PgPool,
    tenant_id: Uuid,
    category_id: Uuid,
) -> Result<Category, AppError> {
    info!("Service: Getting category with ID: {} for tenant ID: {}", category_id, tenant_id);

    let category = query_as!(
        Category,
        r#"
        SELECT
            id, tenant_id, name, description, type as "r#type!: CategoryType",
            parent_category_id, is_active, created_at, created_by, updated_at, updated_by
        FROM categories
        WHERE id = $1 AND tenant_id = $2 AND is_active = TRUE
        "#,
        category_id,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Category with ID {} not found for tenant {}", category_id, tenant_id)))?;

    Ok(category)
}

/// Creates a new category for a specific tenant.
pub async fn create_category(
    pool: &PgPool,
    tenant_id: Uuid,
    created_by_user_id: Uuid,
    dto: CreateCategoryDto,
) -> Result<Category, AppError> {
    info!("Service: Creating new category with name: {} for tenant ID {}", dto.name, tenant_id);

    let new_category = query_as!(
        Category,
        r#"
        INSERT INTO categories (
            tenant_id, name, description, type, parent_category_id,
            is_active, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, TRUE, $6, $6)
        RETURNING
            id, tenant_id, name, description, type as "r#type!: CategoryType",
            parent_category_id, is_active, created_at, created_by, updated_at, updated_by
        "#,
        tenant_id,
        dto.name,
        dto.description,
        dto.r#type as CategoryType, // Cast to enum for query
        dto.parent_category_id,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_category)
}

/// Updates an existing category for a specific tenant.
pub async fn update_category(
    pool: &PgPool,
    tenant_id: Uuid,
    category_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateCategoryDto,
) -> Result<Category, AppError> {
    info!("Service: Updating category with ID: {} for tenant ID: {}", category_id, tenant_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(name) = dto.name {
        update_cols.push(format!("name = ${}", param_idx));
        update_values.push(Box::new(name));
        param_idx += 1;
    }
    if let Some(description) = dto.description {
        update_cols.push(format!("description = ${}", param_idx));
        update_values.push(Box::new(description));
        param_idx += 1;
    }
    if let Some(r#type) = dto.r#type {
        update_cols.push(format!("type = ${}", param_idx));
        update_values.push(Box::new(r#type as CategoryType)); // Cast enum for binding
        param_idx += 1;
    }
    if let Some(parent_category_id) = dto.parent_category_id {
        update_cols.push(format!("parent_category_id = ${}", param_idx));
        update_values.push(Box::new(parent_category_id));
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
        UPDATE categories
        SET {}
        WHERE id = ${} AND tenant_id = ${}
        RETURNING
            id, tenant_id, name, description, type as "r#type!: CategoryType",
            parent_category_id, is_active, created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx, param_idx + 1 // category_id and tenant_id will be the last parameters
    );

    let mut query = sqlx::query_as::<_, Category>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind category_id and tenant_id last
    query = query.bind(category_id);
    query = query.bind(tenant_id);

    let updated_category = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Category with ID {} not found or not owned by tenant {}", category_id, tenant_id)))?;

    Ok(updated_category)
}

/// Deactivates a category (soft delete) for a specific tenant.
pub async fn deactivate_category(
    pool: &PgPool,
    tenant_id: Uuid,
    category_id: Uuid,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating category with ID: {} for tenant ID: {}", category_id, tenant_id);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE categories
        SET
            is_active