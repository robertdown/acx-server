// src/main.rs (with updated user routes import)

// Standard library imports
use std::error::Error as StdError;
use std::net::SocketAddr; // Alias for StdError to avoid conflict with AppError

// Third-party crates
use axum::{
    response::IntoResponse, // Added for IntoResponse trait from AppError
    Router,
};
use dotenvy::dotenv;
use sqlx::PgPool; // Database connection pool
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level}; // For loading .env file

// Internal modules
mod app_state;
mod db;
mod error;
mod user;

use crate::app_state::AppState; // Import AppState from app_state module
use db::setup_database;
use error::AppError; // This path remains the same

// Update the user_routes import!
use crate::user::handlers::user_routes; // CHANGED: from `crate::api::user_handlers::user_routes`

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Using StdError alias
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing (logging)
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting Forge API server...");

    // Database setup
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let pool = setup_database(&database_url).await.map_err(|e| {
        Box::new(AppError::DatabaseError(format!(
            "Failed to connect to the database: {}",
            e
        )))
    })?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            Box::new(AppError::InternalServerError(format!(
                "Failed to run database migrations: {}",
                e
            )))
        })?;

    // Create AppState
    let app_state = AppState { pool };

    // Build our application routes
    let app = Router::new()
        .nest("/api/v1/users", user_routes())
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    // Run the server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Forge API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;
    tracing::info!("Forge API server stopped gracefully.");

    Ok(())
}
