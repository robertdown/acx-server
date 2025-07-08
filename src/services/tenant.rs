use sqlx::{query_as, PgPool};
use uuid::Uuid;
use tracing::info;
use chrono::Utc;

use crate::{
    error::AppError,
    models::{
        tenant::Tenant,
        dto::tenant_dto::{CreateTenantDto, UpdateTenantDto},
    },
};

/// Retrieves a list of all active tenants.
pub async fn list_tenants(pool: &PgPool) -> Result<Vec<Tenant>, AppError> {
    info!("Service: Listing all active tenants.");

    let tenants = query_as!(
        Tenant,
        r#"
        SELECT
            id, name, industry, base_currency_code, fiscal_year_end_month, is_active,
            created_at, created_by, updated_at, updated_by
        FROM tenants
        WHERE is_active = TRUE
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(tenants)
}

/// Retrieves a single tenant by ID.
pub async fn get_tenant_by_id(pool: &PgPool, tenant_id: Uuid) -> Result<Tenant, AppError> {
    info!("Service: Getting tenant with ID: {}", tenant_id);

    let tenant = query_as!(
        Tenant,
        r#"
        SELECT
            id, name, industry, base_currency_code, fiscal_year_end_month, is_active,
            created_at, created_by, updated_at, updated_by
        FROM tenants
        WHERE id = $1 AND is_active = TRUE
        "#,
        tenant_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Tenant with ID {} not found", tenant_id)))?;

    Ok(tenant)
}

/// Creates a new tenant.
/// `created_by_user_id` should come from the authenticated system administrator or initial setup process.
pub async fn create_tenant(
    pool: &PgPool,
    created_by_user_id: Uuid,
    dto: CreateTenantDto,
) -> Result<Tenant, AppError> {
    info!("Service: Creating new tenant with name: {}", dto.name);

    let new_tenant = query_as!(
        Tenant,
        r#"
        INSERT INTO tenants (
            name, industry, base_currency_code, fiscal_year_end_month,
            is_active, created_by, updated_by
        )
        VALUES ($1, $2, $3, $4, TRUE, $5, $5)
        RETURNING
            id, name, industry, base_currency_code, fiscal_year_end_month, is_active,
            created_at, created_by, updated_at, updated_by
        "#,
        dto.name,
        dto.industry,
        dto.base_currency_code,
        dto.fiscal_year_end_month,
        created_by_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(new_tenant)
}

/// Updates an existing tenant.
/// `updated_by_user_id` should come from the authenticated system administrator.
pub async fn update_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
    updated_by_user_id: Uuid,
    dto: UpdateTenantDto,
) -> Result<Tenant, AppError> {
    info!("Service: Updating tenant with ID: {}", tenant_id);

    let mut update_cols: Vec<String> = Vec::new();
    let mut update_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_idx = 1;

    if let Some(name) = dto.name {
        update_cols.push(format!("name = ${}", param_idx));
        update_values.push(Box::new(name));
        param_idx += 1;
    }
    if let Some(industry) = dto.industry {
        update_cols.push(format!("industry = ${}", param_idx));
        update_values.push(Box::new(industry));
        param_idx += 1;
    }
    if let Some(base_currency_code) = dto.base_currency_code {
        update_cols.push(format!("base_currency_code = ${}", param_idx));
        update_values.push(Box::new(base_currency_code));
        param_idx += 1;
    }
    if let Some(fiscal_year_end_month) = dto.fiscal_year_end_month {
        update_cols.push(format!("fiscal_year_end_month = ${}", param_idx));
        update_values.push(Box::new(fiscal_year_end_month));
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
        UPDATE tenants
        SET {}
        WHERE id = ${}
        RETURNING
            id, name, industry, base_currency_code, fiscal_year_end_month, is_active,
            created_at, created_by, updated_at, updated_by
        "#,
        update_clause, param_idx // tenant_id will be the last parameter
    );

    let mut query = sqlx::query_as::<_, Tenant>(&query_str);

    for val in update_values {
        query = query.bind(val);
    }
    // Bind tenant_id last
    query = query.bind(tenant_id);

    let updated_tenant = query
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Tenant with ID {} not found", tenant_id)))?;

    Ok(updated_tenant)
}

/// Deactivates a tenant (soft delete).
/// `updated_by_user_id` should come from the authenticated system administrator.
pub async fn deactivate_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
    updated_by_user_id: Uuid,
) -> Result<(), AppError> {
    info!("Service: Deactivating tenant with ID: {}", tenant_id);

    let affected_rows = sqlx::query!(
        r#"
        UPDATE tenants
        SET
            is_active = FALSE,
            updated_at = NOW(),
            updated_by = $2
        WHERE id = $1 AND is_active = TRUE
        "#,
        tenant_id,
        updated_by_user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected_rows == 0 {
        return Err(AppError::NotFound(format!("Tenant with ID {} not found or already inactive", tenant_id)));
    }

    Ok(())
}