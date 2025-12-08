//! Service implementations
//!
//! This module contains implementations of the service traits with business logic.

pub mod category;
pub mod delivery;
pub mod lot_serial;
pub mod picking_method;
pub mod product;
pub mod putaway;
pub mod quality;
pub mod receipt;
pub mod reconciliation;
pub mod replenishment;
pub mod rma;
pub mod stock_take;
pub mod transfer;
pub mod valuation;

#[cfg(test)]
mod category_tests;

// Re-export services for convenience
pub use category::CategoryServiceImpl;
// pub use delivery::DeliveryServiceImpl;
pub use self::picking_method::PickingMethodServiceImpl;
pub use lot_serial::LotSerialServiceImpl;
pub use product::ProductServiceImpl;
pub use putaway::PgPutawayService;
pub use quality::PgQualityControlPointService;
pub use receipt::ReceiptServiceImpl;
pub use reconciliation::PgStockReconciliationService;
pub use replenishment::PgReplenishmentService;
pub use rma::PgRmaService;
pub use stock_take::PgStockTakeService;
pub use transfer::PgTransferService;
pub use valuation::ValuationServiceImpl;
