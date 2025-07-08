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
    models::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse},
    services::user,
    // Placeholder for authentication context; in a real app, you'd extract this
    // from a custom Axum extractor based on a JWT or session.
    utils::auth_middleware::get_current_user_id, // This utility would provide the user_id from auth
};

// State struct to hold application-wide dependencies
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    // Add other dependencies like configuration, another service client, etc.
}

// Function to create a router specifically for user-related routes
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:id", get(get_user_by_id))
        .route("/:id", put(update_user))
        .route("/:id", delete(deactivate_user))
}

/// GET /users
/// Lists all active users.
async fn list_users(
    State(AppState { pool, .. }): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    info!("Handler: Listing users");
    let users = user::list_users(&pool).await?;
    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(user_responses))
}

/// GET /users/:id
/// Retrieves a single user by ID.
async fn get_user_by_id(
    State(AppState { pool, .. }): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    info!("Handler: Getting user by ID: {}", user_id);
    let found_user = user::get_user_by_id(&pool, user_id).await?;
    Ok(Json(UserResponse::from(found_user)))
}

/// POST /users
/// Creates a new user.
async fn create_user(
    State(AppState { pool, .. }): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    info!("Handler: Creating new user with email: {}", req.email);

    // In a real application, you would handle password hashing here or in a dedicated auth service
    // before passing it to the service layer. For this example, we'll map directly.
    let password_hash = req.password.as_ref().map(|p| p.to_string()); // Real app: hash this!

    let new_user = user::create_user(
        &pool,
        req.auth_provider_id,
        req.auth_provider_type,
        req.email,
        password_hash,
        req.first_name,
        req.last_name,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(new_user))))
}

/// PUT /users/:id
/// Updates an existing user.
async fn update_user(
    State(AppState { pool, .. }): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    info!("Handler: Updating user with ID: {}", user_id);

    // Placeholder: Get current user ID from authentication context (e.g., JWT)
    // For now, using a dummy function.
    let updated_by_user_id = get_current_user_id();

    // Handle password update: if provided, hash it before passing to service
    let password_hash = req.password.as_ref().map(|p| p.to_string()); // Real app: hash this!

    let updated_user = user::update_user(
        &pool,
        user_id,
        updated_by_user_id,
        req.auth_provider_id,
        req.auth_provider_type,
        req.email,
        password_hash,
        req.first_name,
        req.last_name,
        req.is_active,
    )
    .await?;

    Ok(Json(UserResponse::from(updated_user)))
}

/// DELETE /users/:id
/// Deactivates a user (soft delete).
async fn deactivate_user(
    State(AppState { pool, .. }): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    info!("Handler: Deactivating user with ID: {}", user_id);

    // Placeholder: Get current user ID from authentication context
    let updated_by_user_id = get_current_user_id();

    user::deactivate_user(&pool, user_id, updated_by_user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}