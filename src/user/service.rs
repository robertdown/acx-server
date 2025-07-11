// src/services/user.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres};
use tracing::{debug, info};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    user::{
        dto::{CreateUserRequest, UpdateUserRequest, UserResponse},
        models::User,
    },
};

/// Hashes a plain-text password using Argon2.
pub(crate) fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalServerError(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}

/// Verifies a plain-text password against a stored hash.
pub(crate) fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        AppError::InternalServerError(format!("Failed to parse password hash: {}", e))
    })?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Creates a new user in the database.
///
/// Hashes the password before storing it.
pub async fn create_user(pool: &PgPool, req: CreateUserRequest) -> Result<User, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let password_hash = if let Some(pwd) = req.password {
        Some(hash_password(&pwd)?)
    } else {
        None
    };

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (auth_provider_id, auth_provider_type, email, password_hash, first_name, last_name)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, auth_provider_id, auth_provider_type, email, password_hash, first_name, last_name, is_active, last_login_at, created_at, updated_at
        "#,
        req.auth_provider_id,
        req.auth_provider_type,
        req.email,
        password_hash,
        req.first_name,
        req.last_name,
    )
    .fetch_one(pool)
    .await?;

    info!("User created successfully with ID: {}", user.id);
    Ok(user)
}

/// Retrieves a user by their ID.
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, auth_provider_id, auth_provider_type, email, password_hash, first_name, last_name, is_active, last_login_at, created_at, updated_at
        FROM users
        WHERE id = $1 AND is_active = TRUE
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", user_id)))?;

    Ok(user)
}

/// Retrieves a user by their email address.
pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, auth_provider_id, auth_provider_type, email, password_hash, first_name, last_name, is_active, last_login_at, created_at, updated_at
        FROM users
        WHERE email = $1 AND is_active = TRUE
        "#,
        email
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User with email '{}' not found", email)))?;

    Ok(user)
}

/// Lists all active users.
pub async fn list_users(pool: &PgPool) -> Result<Vec<User>, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, auth_provider_id, auth_provider_type, email, password_hash, first_name, last_name, is_active, last_login_at, created_at, updated_at
        FROM users
        WHERE is_active = TRUE
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// Updates an existing user's information.
///
/// Can update password if provided.
pub async fn update_user(
    pool: &PgPool,
    user_id: Uuid,
    req: UpdateUserRequest,
) -> Result<User, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Fetch current user to compare fields and handle partial updates
    let mut current_user = get_user_by_id(pool, user_id).await?;

    let mut password_hash_to_update: Option<String> = None;
    if let Some(new_password) = req.password {
        password_hash_to_update = Some(hash_password(&new_password)?);
    } else {
        // If password is not provided in the request, retain the existing hash
        password_hash_to_update = current_user.password_hash;
    }

    let updated_user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET
            email = COALESCE($1, email),
            password_hash = COALESCE($2, password_hash),
            first_name = COALESCE($3, first_name),
            last_name = COALESCE($4, last_name),
            updated_at = NOW()
        WHERE id = $5
        RETURNING id, auth_provider_id, auth_provider_type, email, password_hash, first_name, last_name, is_active, last_login_at, created_at, updated_at
        "#,
        req.email,
        password_hash_to_update,
        req.first_name,
        req.last_name,
        user_id
    )
    .fetch_one(pool)
    .await?;

    info!("User with ID {} updated successfully", user_id);
    Ok(updated_user)
}

/// Deactivates a user by setting `is_active` to `FALSE`.
pub async fn deactivate_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET is_active = FALSE, updated_at = NOW()
        WHERE id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        )));
    }

    info!("User with ID {} deactivated successfully", user_id);
    Ok(())
}
