//! Category HTTP handlers
//!
//! This module contains the Axum handlers for category management endpoints.

pub mod category;

// Re-export handlers for convenience
pub use category::CategoryHandler;
pub use category::CategoryHandlerState;
