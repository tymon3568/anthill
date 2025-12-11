//! Inventory Service Infrastructure
//!
//! This crate contains the infrastructure implementations for the inventory service.
//! It provides concrete implementations of repositories and services.
//!
//! ## Architecture
//!
//! - `repositories/`: PostgreSQL repository implementations
//! - `services/`: Service implementations with business logic

pub mod repositories;
pub mod services;

/// Helper type for infra-internal transaction operations
pub type InfraTx<'a> = &'a mut sqlx::Transaction<'a, sqlx::Postgres>;

// Re-export main implementations for convenience
pub use repositories::category::CategoryRepositoryImpl;
pub use repositories::product::ProductRepositoryImpl;
pub use repositories::putaway::PgPutawayRepository;
pub use repositories::quality::PgQualityControlPointRepository;
pub use repositories::removal_strategy::RemovalStrategyRepositoryImpl;
pub use repositories::replenishment::PgReorderRuleRepository;
pub use repositories::valuation::ValuationRepositoryImpl;
pub use services::cache::{RedisCache, SharedCache, SharedInventoryCache, SharedProductCache};
pub use services::product::ProductServiceImpl;
pub use services::putaway::PgPutawayService;
pub use services::quality::PgQualityControlPointService;
pub use services::receipt::ReceiptServiceImpl;
pub use services::removal_strategy::RemovalStrategyServiceImpl;
pub use services::replenishment::PgReplenishmentService;
pub use services::CategoryServiceImpl;
pub use services::PgRmaService;
pub use services::PgStockReconciliationService;
pub use services::PgStockTakeService;
pub use services::PgTransferService;
pub use services::ValuationServiceImpl;
