use axum::{
    extract::{Path, State, Json},
    routing::{get, post, put, delete},
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::info;

use crate::{
    error::AppError,
    models::dto::tenant_dto::{CreateTenantRequest, UpdateTenantRequest, TenantResponse},
    services::tenant,
    // Placeholder for authentication context
    utils::auth_middleware::get_current_user_id, // This utility would provide the user_id from auth
    api::user_handlers::AppState, // Import AppState from user_handlers or a common api::mod
};


// Function to create a router specifically for tenant-related routes
pub fn tenant_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants))
        .route("/", post(create_tenant))
        .route("/:id", get(get_tenant_by_id))
        .route("/:id", put(update_tenant))
        .route("/:id", delete(deactivate_tenant))
}

/// GET /tenants
/// Lists all active tenants.
/// Requires current_user_id for filtering tenants that the user has access to.
async fn list_tenants(
    State(AppState { pool, .. }): State<AppState>,
) -> Result<Json<Vec<TenantResponse>>, AppError> {
    info!("Handler: Listing tenants");
    // In a multi-tenant app, this would typically be `list_tenants_for_user`
    // requiring `current_user_id` from auth context.
    let tenants = tenant::list_tenants(&pool).await?;
    let tenant_responses: Vec<TenantResponse> = tenants.into_iter().map(TenantResponse::from).collect();
    Ok(Json(tenant_responses))
}

/// GET /tenants/:id
/// Retrieves a single tenant by ID.
async fn get_tenant_by_id(
    State(AppState { pool, .. }): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantResponse>, AppError> {
    info!("Handler: Getting tenant by ID: {}", tenant_id);
    let found_tenant = tenant::get_tenant_by_id(&pool, tenant_id).await?;
    Ok(Json(TenantResponse::from(found_tenant)))
}

/// POST /tenants
/// Creates a new tenant.
async fn create_tenant(
    State(AppState { pool, .. }): State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<TenantResponse>), AppError> {
    info!("Handler: Creating new tenant with name: {}", req.name);

    // Placeholder: Get current user ID from authentication context
    let created_by_user_id = get_current_user_id();

    let new_tenant = tenant::create_tenant(
        &pool,
        req.name,
        req.industry,
        req.base_currency_code,
        created_by_user_id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(TenantResponse::from(new_tenant))))
}

/// PUT /tenants/:id
/// Updates an existing tenant.
async fn update_tenant(
    State(AppState { pool, .. }): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<UpdateTenantRequest>,
) -> Result<Json<TenantResponse>, AppError> {
    info!("Handler: Updating tenant with ID: {}", tenant_id);

    // Placeholder: Get current user ID from authentication context
    let updated_by_user_id = get_current_user_id();

    let updated_tenant = tenant::update_tenant(
        &pool,
        tenant_id,
        req.name,
        req.industry,
        req.base_currency_code,
        req.is_active,
        updated_by_user_id,
    )
    .await?;

    Ok(Json(TenantResponse::from(updated_tenant)))
}

/// DELETE /tenants/:id
/// Deactivates a tenant (soft delete).
async fn deactivate_tenant(
    State(AppState { pool, .. }): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    info!("Handler: Deactivating tenant with ID: {}", tenant_id);

    // Placeholder: Get current user ID from authentication context
    let updated_by_user_id = get_current_user_id();

    tenant::deactivate_tenant(&pool, tenant_id, updated_by_user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}