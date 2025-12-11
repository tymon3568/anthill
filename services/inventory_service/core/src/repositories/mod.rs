//! Repository implementations
//!
//! This module contains PostgreSQL implementations of the repository traits.

pub mod category;
pub mod event;
pub mod picking_method;
pub mod putaway;
pub mod quality;
pub mod removal_strategy;
pub mod replenishment;

/// Transaction abstraction to avoid sqlx dependency in core
pub mod transaction {
    use async_trait::async_trait;
    use shared_error::AppError;

    /// Abstract transaction trait for database operations
    #[async_trait]
    pub trait Transaction: Send + Sync {
        /// Commit the transaction
        async fn commit(self) -> Result<(), AppError>;
    }
}
pub mod delivery_order;
pub mod lot_serial;
pub mod product;
pub mod receipt;
pub mod reconciliation;
pub mod rma;
pub mod stock;
pub mod stock_take;
pub mod transfer;
pub mod valuation;
pub mod warehouse;

// Re-export repository traits for convenience
pub use category::CategoryRepository;
pub use delivery_order::{
    DeliveryOrderItemRepository, DeliveryOrderRepository, InventoryRepository,
};
pub use event::EventRepository;
pub use lot_serial::LotSerialRepository;
pub use picking_method::PickingMethodRepository;
pub use product::ProductRepository;
pub use putaway::{PutawayRepository, PutawayService};
pub use quality::QualityControlPointRepository;
pub use receipt::ReceiptRepository;
pub use reconciliation::{StockReconciliationItemRepository, StockReconciliationRepository};
pub use removal_strategy::RemovalStrategyRepository;
pub use replenishment::ReorderRuleRepository;
pub use rma::{RmaItemRepository, RmaRepository};
pub use stock::{InventoryLevelRepository, StockMoveRepository};
pub use stock_take::{StockTakeLineRepository, StockTakeRepository};
pub use transfer::{TransferItemRepository, TransferRepository};
pub use valuation::ValuationRepository;
pub use warehouse::WarehouseRepository;
