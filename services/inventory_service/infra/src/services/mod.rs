//! Service implementations
//!
//! This module contains implementations of the service traits with business logic.

pub mod category;

// Re-export services for convenience
pub use category::CategoryServiceImpl;
