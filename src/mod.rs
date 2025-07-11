//! Top-level module declarations for the Forge backend application.
//!
//! This file organizes the main components of the application into logical modules,
//! facilitating a clean and maintainable project structure.

pub mod app_state;      // Defines the shared application state (e.g., database pool).
pub mod config;         // Handles application configuration loading.
pub mod db;             // Manages database connection and pooling.
pub mod error;          // Defines custom error types and their conversion to HTTP responses.
// pub mod middleware;     // Houses custom Tower middleware for cross-cutting concerns.
pub mod utils;          // Provides general utility functions and helpers.