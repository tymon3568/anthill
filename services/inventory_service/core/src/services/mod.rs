//! Service trait definitions for inventory service
//!
//! This module contains trait definitions for business logic operations.
//! No implementations here - pure interfaces.

pub mod cache;
pub mod category;
pub mod cycle_count;
pub mod delivery;
pub mod distributed_lock;
pub mod inventory;
pub mod landed_cost;
pub mod lot_serial;

pub mod picking_method;
pub mod product;
pub mod quality;
pub mod receipt;
pub mod reconciliation;
pub mod removal_strategy;
pub mod replenishment;
pub mod reports;
pub mod rma;
pub mod scrap;
pub mod stock_take;
pub mod transfer;
pub mod valuation;

// Re-export main types for convenience
pub use category::CategoryService;
pub use distributed_lock::DistributedLockService;
pub use picking_method::PickingMethodService;
pub use quality::QualityControlPointService;
// pub use delivery::DeliveryService;
pub use cache::{CacheService, InventoryCache, ProductCache};
pub use inventory::InventoryService;
pub use lot_serial::LotSerialService;
pub use product::ProductService;
pub use receipt::ReceiptService;
pub use reconciliation::StockReconciliationService;
pub use removal_strategy::RemovalStrategyService;
pub use replenishment::ReplenishmentService;
pub use rma::RmaService;
pub use stock_take::StockTakeService;
pub use transfer::TransferService;
pub use valuation::ValuationService;

// New services for MVP P1 features
pub use cycle_count::CycleCountingService;
pub use landed_cost::LandedCostService;
pub use reports::ReportsService;
pub use scrap::ScrapService;
