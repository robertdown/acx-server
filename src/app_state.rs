use sqlx::PgPool;

/// Shared application state accessible by Axum handlers.
///
/// This struct holds dependencies like the database connection pool.
#[derive(Clone)] // Axum requires AppState to be Clone
pub struct AppState {
    pub pool: PgPool,
    // pub config: crate::config::AppConfig, // Uncomment when config is ready
}
