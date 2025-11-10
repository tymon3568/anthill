//! Service implementations
//!
//! This module contains implementations of the service traits with business logic.

pub mod category;
pub mod product;

#[cfg(test)]
mod category_tests;

// Re-export services for convenience
pub use category::CategoryServiceImpl;
pub use product::ProductServiceImpl;
