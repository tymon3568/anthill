//! Event definitions for the inventory service
//!
//! This module contains event types and structures used in the transactional outbox pattern.
//! Events are published to NATS for reliable messaging.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Inventory updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryUpdatedEvent {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub warehouse_id: Uuid,
    pub location_id: Option<Uuid>,
    pub lot_serial_id: Option<Uuid>,
    pub quantity_change: i64, // positive for increase, negative for decrease
    pub reason: String,       // e.g., "receipt", "shipment", "adjustment"
    pub reference_id: Uuid,   // ID of the transaction that caused the change
    pub reference_type: String, // e.g., "receipt", "delivery_order", "adjustment"
}

/// Goods receipt validated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptValidatedEvent {
    pub receipt_id: Uuid,
    pub items: Vec<ReceiptItemEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItemEvent {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_received: i64,
    pub warehouse_id: Uuid,
    pub location_id: Option<Uuid>,
    pub lot_serial_id: Option<Uuid>,
}

/// Delivery order shipped event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderShippedEvent {
    pub delivery_order_id: Uuid,
    pub items: Vec<DeliveryItemEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryItemEvent {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_shipped: i64,
    pub warehouse_id: Uuid,
    pub location_id: Option<Uuid>,
    pub lot_serial_id: Option<Uuid>,
}

/// Stock adjustment event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAdjustmentEvent {
    pub adjustment_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub warehouse_id: Uuid,
    pub location_id: Option<Uuid>,
    pub lot_serial_id: Option<Uuid>,
    pub quantity_adjusted: i64,
    pub reason: String,
}

/// Event type constants
pub mod event_types {
    pub const INVENTORY_UPDATED: &str = "inventory.updated";
    pub const GOODS_RECEIPT_VALIDATED: &str = "goods_receipt.validated";
    pub const DELIVERY_ORDER_SHIPPED: &str = "delivery_order.shipped";
    pub const STOCK_ADJUSTMENT: &str = "stock.adjustment";
}
