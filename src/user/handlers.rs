use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::app_state::AppState; // Assuming AppState is defined in src/app_state.rs
use crate::error::AppError; // Importing our custom AppError
use crate::user::dto::{CreateUserRequest, UpdateUserRequest, UserResponse}; // Importing DTOs
use crate::user::service as user; // Importing our user service

/// Creates a router for user-related API endpoints.
///
/// All routes defined here will be nested under `/api/v1/users` in `main.rs`.
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users)) // GET /api/v1/users
        .route("/", post(create_user)) // POST /api/v1/users
        .route("/:id", get(get_user_by_id)) // GET /api/v1/users/:id
        .route("/:id", put(update_user)) // PUT /api/v1/users/:id
        .route("/:id", delete(deactivate_user)) // DELETE /api/v1/users/:id (soft delete)
}

/// GET /api/v1/users
/// Lists all active users.
async fn list_users(
    State(AppState { pool, .. }): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    info!("Handler: Listing all users");
    let users = user::list_users(&pool).await?;
    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(user_responses))
}

/// GET /api/v1/users/:id
/// Retrieves a single user by their ID.
async fn get_user_by_id(
    State(AppState { pool, .. }): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    info!("Handler: Getting user by ID: {}", user_id);
    let found_user = user::get_user_by_id(&pool, user_id).await?;
    Ok(Json(UserResponse::from(found_user)))
}

/// POST /api/v1/users
/// Creates a new user.
async fn create_user(
    State(AppState { pool, .. }): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    info!("Handler: Creating new user with email: {}", req.email);
    let new_user = user::create_user(&pool, req).await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(new_user))))
}

/// PUT /api/v1/users/:id
/// Updates an existing user's information.
async fn update_user(
    State(AppState { pool, .. }): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    info!("Handler: Updating user with ID: {}", user_id);
    let updated_user = user::update_user(&pool, user_id, req).await?;
    Ok(Json(UserResponse::from(updated_user)))
}

/// DELETE /api/v1/users/:id
/// Deactivates a user (soft delete by setting `is_active` to false).
async fn deactivate_user(
    State(AppState { pool, .. }): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    info!("Handler: Deactivating user with ID: {}", user_id);
    user::deactivate_user(&pool, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
