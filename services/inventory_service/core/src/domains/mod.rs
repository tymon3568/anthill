//! Domain models for inventory service
//!
//! This module contains the core domain entities and business logic.

pub mod category;
pub mod inventory;

// Re-export main types for convenience
pub use category::{Category, CategoryBreadcrumb, CategoryNode};
