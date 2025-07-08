use axum::{
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import our custom modules.
// These `mod` declarations tell Rust to compile and include the code from these directories/files.
mod app_state;
mod config; // We defined this in the folder structure, will hold app configuration
mod db;
mod error;
mod models;
mod routes;
mod services;
mod middleware; // Added for future middleware integration
mod utils;    // Added for future utility integration

// The main entry point for our asynchronous Rust application.
// `#[tokio::main]` macro sets up the Tokio runtime for async operations.
#[tokio::main]
async fn main() {
    // 1. Initialize Tracing (Logging)
    // This sets up a global logger that captures events from `tracing` macros
    // (like `info!`, `debug!`, `error!`).
    tracing_subscriber::registry()
        // Filters log events based on environment variables (e.g., RUST_LOG=debug).
        // Defaults to showing debug logs for our app and info/debug for sqlx/tower-http.
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "forge_backend=debug,sqlx=info,tower_http=debug".into()),
        )
        // Formats log events for console output.
        .with(tracing_subscriber::fmt::layer())
        // Installs the subscriber globally.
        .init();

    // 2. Load Environment Variables from .env file
    // `dotenvy::dotenv().ok()` attempts to load variables from a `.env` file in the project root.
    // `.ok()` prevents crashing if the file doesn't exist (e.g., in production where env vars are set directly).
    dotenvy::dotenv().ok();
    tracing::info!("Environment variables loaded.");

    // 3. Database Connection Pool Setup
    // Retrieves the DATABASE_URL from environment variables.
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    tracing::info!("Connecting to database...");
    let pool = db::connect_to_db(&database_url)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Database connection pool established.");

    // TODO: 4. Load Application Configuration (Future Step, Placeholder)
    // In a real app, you might load other config like JWT secrets here
    // let app_config = config::AppConfig::load().expect("Failed to load application configuration");
    // tracing::info!("Application configuration loaded.");


    // 5. Create Application State
    // This state will be shared across all your API handlers, providing access to the DB pool
    // and potentially other shared resources (like config, or client for external services).
    let app_state = app_state::AppState { pool }; // Add other fields like `config: app_config` later

    // 6. Build the Axum Application Router
    // This defines all the API endpoints and maps them to their respective handler functions.
    let app = Router::new()
        // Merge routes from different modules to organize your API endpoints.
        // We've already generated `currency_routes` and `transaction_routes`.
        // You will add more `merge` calls here as you implement more features.
        .merge(routes::currency::currency_routes())
        .merge(routes::transaction::transaction_routes())
        .merge(routes::user::user_routes())     // Assuming you'll add these next
        .merge(routes::tenant::tenant_routes()) // Assuming you'll add these next
        // TODO: Add more routes here as you implement them for other phases/models
        // .merge(routes::account::account_routes())
        // .merge(routes::category::category_routes())
        // .merge(routes::auth::auth_routes()) // For authentication endpoints

        // TODO: Add global middleware here (e.g., for tracing, authentication, CORS)
        // .layer(middleware::auth::jwt_auth_layer()) // Example: JWT authentication
        // .layer(tower_http::trace::TraceLayer::new_for_http()) // Example: Request tracing

        // Attach the application state to the router so handlers can access it.
        .with_state(app_state);

    // 7. Run the Axum Server
    // Retrieves host and port from environment variables, defaulting if not set.
    let app_host = std::env::var("APP_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let app_port = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>() // Parse the port as an unsigned 16-bit integer
        .expect("APP_PORT must be a valid number");

    let addr = SocketAddr::new(app_host.parse().expect("Invalid APP_HOST"), app_port);
    tracing::info!("Forge backend server listening on {}", addr);

    // Start serving the application. `await` here means the main function will block
    // until the server shuts down (e.g., due to a signal or an unrecoverable error).
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap(); // Unwrapping here will panic if the server encounters a fatal error during startup or runtime.
}

// Dummy function for current_user_id for compilation.
// Replace with actual authentication logic in a `utils/auth_middleware.rs` or similar.
mod utils {
    pub mod auth_middleware {
        use uuid::Uuid;
        pub fn get_current_user_id() -> Uuid {
            // In a real app, this would extract user ID from a JWT, session, etc.
            // For now, return a fixed ID or generate a new one for testing.
            Uuid::new_v4() // Example: generate a new ID every time (not practical for auth)
            // Or return a fixed one for easier testing: Uuid::parse_str("your-test-user-id").unwrap()
        }
    }
}

// Re-export modules to make them accessible
pub mod api;
pub mod models;
pub mod services;
pub mod error; // Make sure your error module is public