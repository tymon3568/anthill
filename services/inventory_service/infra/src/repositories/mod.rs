//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;

// Re-export repositories for convenience
pub use category::CategoryRepositoryImpl;
