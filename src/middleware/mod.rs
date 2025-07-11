//! Custom Tower middleware for the Forge application.
//!
//! This module contains reusable middleware components for cross-cutting concerns
//! such as authentication, logging, and potentially rate limiting or CORS.

pub mod auth; // For authentication middleware (e.g., JWT validation)
pub mod logging; // For request logging (though Tower-HTTP's TraceLayer is often sufficient)
// pub mod rate_limiting; // Example for future use