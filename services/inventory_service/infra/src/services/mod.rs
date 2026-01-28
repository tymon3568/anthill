//! Service implementations
//!
//! This module contains implementations of the service traits with business logic.

pub mod cache;
pub mod category;
pub mod cycle_count;
pub mod delivery;
pub mod distributed_lock;
pub mod inventory;
pub mod lot_serial;

pub mod landed_cost;
pub mod picking_method;
pub mod product;
pub mod product_variant;
pub mod putaway;
pub mod quality;
pub mod receipt;
pub mod reconciliation;
pub mod removal_strategy;
pub mod replenishment;
pub mod reports;
pub mod rma;
pub mod scrap;
pub mod stock_levels;
pub mod stock_take;
pub mod transfer;
pub mod valuation;

#[cfg(test)]
mod category_tests;
#[cfg(test)]
mod inventory_tests;
#[cfg(test)]
mod lot_serial_tests;
#[cfg(test)]
mod product_tests;
#[cfg(test)]
mod replenishment_tests;
#[cfg(test)]
mod valuation_tests;

// Re-export services for convenience
pub use cache::{RedisCache, SharedCache, SharedInventoryCache, SharedProductCache};
pub use category::CategoryServiceImpl;
// pub use delivery::DeliveryServiceImpl;
pub use self::picking_method::PickingMethodServiceImpl;
pub use distributed_lock::RedisDistributedLockService;
pub use inventory::InventoryServiceImpl;
pub use landed_cost::LandedCostServiceImpl;
pub use lot_serial::LotSerialServiceImpl;
pub use product::ProductServiceImpl;
pub use product_variant::ProductVariantServiceImpl;
pub use putaway::PgPutawayService;
pub use quality::PgQualityControlPointService;
pub use receipt::ReceiptServiceImpl;
pub use reconciliation::PgStockReconciliationService;
pub use removal_strategy::RemovalStrategyServiceImpl;
pub use replenishment::PgReplenishmentService;
pub use rma::PgRmaService;
pub use stock_take::PgStockTakeService;
pub use transfer::PgTransferService;
pub use valuation::ValuationServiceImpl;

// New services for MVP P1 features
pub use cycle_count::PgCycleCountingService;
pub use reports::PgReportsService;
pub use scrap::PgScrapService;
pub use stock_levels::PgStockLevelsService;
