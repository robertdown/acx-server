
// src/utils/mod.rs

//! General utility functions and helpers for the Forge application.
//!
//! This module provides common functionalities that are not specific to any
//! particular domain or application layer.

// pub mod auth_middleware; // Placeholder for authentication utility functions (e.g., extracting user ID)
pub mod hashing;         // For password hashing (e.g., using Argon2) - currently in user service, could be moved here
pub mod validation;      // For custom validation logic or helpers (beyond `validator` crate)
// pub mod date_time;       // Example for date/time formatting or manipulation