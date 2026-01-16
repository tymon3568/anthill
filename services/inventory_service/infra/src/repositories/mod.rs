//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod delivery_order;
pub mod event;
pub mod landed_cost;
pub mod lot_serial;
pub mod picking_method;
pub mod product;
pub mod putaway;
pub mod quality;
pub mod receipt;
pub mod reconciliation;
pub mod removal_strategy;
pub mod replenishment;
pub mod rma;
pub mod stock;
pub mod stock_take;
pub mod transfer;
pub mod valuation;
pub mod warehouse;

// Re-export repositories for convenience
pub use category::CategoryRepositoryImpl;
pub use delivery_order::{
    PgDeliveryOrderItemRepository, PgDeliveryOrderRepository, PgInventoryRepository,
};
pub use event::EventRepositoryImpl;
pub use landed_cost::{
    LandedCostAllocationRepositoryImpl, LandedCostDocumentRepositoryImpl,
    LandedCostLineRepositoryImpl,
};
pub use lot_serial::LotSerialRepositoryImpl;
pub use picking_method::PickingMethodRepositoryImpl;
pub use product::ProductRepositoryImpl;
pub use putaway::PgPutawayRepository;
pub use quality::PgQualityControlPointRepository;
pub use receipt::ReceiptRepositoryImpl;
pub use reconciliation::{PgStockReconciliationItemRepository, PgStockReconciliationRepository};
pub use removal_strategy::RemovalStrategyRepositoryImpl;
pub use replenishment::PgReorderRuleRepository;
pub use rma::{PgRmaItemRepository, PgRmaRepository};
pub use stock::{PgInventoryLevelRepository, PgStockMoveRepository};
pub use stock_take::{PgStockTakeLineRepository, PgStockTakeRepository};
pub use transfer::{PgTransferItemRepository, PgTransferRepository};
pub use valuation::ValuationRepositoryImpl;
pub use warehouse::WarehouseRepositoryImpl;
