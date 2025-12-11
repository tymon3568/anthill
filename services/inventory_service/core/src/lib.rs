//! Inventory Service Core
//!
//! This crate contains the business logic, domain models, and trait definitions
//! for the inventory service. It has zero infrastructure dependencies.
//!
//! ## Architecture
//!
//! - `domains/`: Domain entities and business logic
//! - `dto/`: Data Transfer Objects for API communication
//! - `models/`: Domain models including delivery orders and items
//! - `repositories/`: Repository trait definitions (no implementations)
//! - `services/`: Service trait definitions (no implementations)

pub mod domains;
pub mod dto;
pub mod events;
pub mod models;
pub mod repositories;
pub mod services;

// Re-export commonly used types
pub use domains::category::{Category, CategoryNode};
pub use dto::category::{
    CategoryCreateRequest, CategoryResponse, CategoryTreeResponse, CategoryUpdateRequest,
};
pub use repositories::category::CategoryRepository;
pub use services::category::CategoryService;

// Re-export replenishment types
pub use domains::replenishment::{
    CreateReorderRule, ReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
pub use repositories::replenishment::ReorderRuleRepository;
pub use services::replenishment::ReplenishmentService;

// Re-export shared error types
pub use shared_error::AppError;

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;
