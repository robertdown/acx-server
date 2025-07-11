// src/error.rs

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult}; // Important for the `?` operator
use sqlx::Error as SqlxError;

#[derive(Debug)] // Derive Debug trait
pub enum AppError {
    DatabaseError(String),
    NotFound(String),
    Validation(String),
    InternalServerError(String),
}

// Implement Display trait for AppError to provide user-friendly error messages
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

// Implement the standard Error trait for AppError.
// This allows AppError to be treated as a general error type,
// which is required for the `?` operator and `Box<dyn Error>`.
impl Error for AppError {}

// Implement IntoResponse for AppError to convert it into an HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", msg),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Validation error: {}", msg),
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", msg),
            ),
        };

        // Create a JSON response for the error
        (
            status,
            axum::Json(serde_json::json!({
                "error": error_message
            })),
        )
            .into_response()
    }
}

impl From<SqlxError> for AppError {
    fn from(error: SqlxError) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}