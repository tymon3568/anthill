//! Service trait definitions for inventory service
//!
//! This module contains trait definitions for business logic operations.
//! No implementations here - pure interfaces.

pub mod category;
pub mod product;
pub mod valuation;

// Re-export main types for convenience
pub use category::CategoryService;
pub use product::ProductService;
pub use valuation::ValuationService;
