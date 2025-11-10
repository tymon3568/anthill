//! Service trait definitions for inventory service
//!
//! This module contains trait definitions for business logic operations.
//! No implementations here - pure interfaces.

pub mod category;
pub mod product;

// Re-export main types for convenience
pub use category::CategoryService;
pub use product::ProductService;
