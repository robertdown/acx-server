use sqlx::{migrate::Migrator, PgPool, Postgres};
use std::path::Path;
use tracing::info;

/// Connects to the PostgreSQL database and returns a connection pool.
///
/// It also attempts to run database migrations from the `./migrations` directory.
pub async fn setup_database(database_url: &str) -> Result<PgPool, sqlx::Error> {
    // Ensure 'pub' is here
    let pool = PgPool::connect(database_url).await?;

    // Run migrations
    info!("Running database migrations...");
    Migrator::new(Path::new("./migrations"))
        .await?
        .run(&pool)
        .await?;
    info!("Database migrations completed.");

    Ok(pool)
}
