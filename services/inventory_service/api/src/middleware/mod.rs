//! Custom middleware for the inventory service

pub mod auth;

// Re-export middleware for convenience
pub use auth::auth_middleware;
