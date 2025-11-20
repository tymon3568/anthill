//! Event definitions for the Anthill platform
//!
//! This module contains all event types used for inter-service communication
//! via NATS messaging.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Order confirmed event
/// Published when an order transitions to confirmed status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfirmedEvent {
    /// Unique order identifier
    pub order_id: Uuid,
    /// Tenant context
    pub tenant_id: Uuid,
    /// Customer identifier
    pub customer_id: Option<Uuid>,
    /// Order items with product and quantity details
    pub items: Vec<OrderItem>,
    /// Expected delivery date
    pub expected_delivery_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Shipping address
    pub shipping_address: Option<Address>,
    /// Additional notes
    pub notes: Option<String>,
}

/// Order item details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    /// Product identifier
    pub product_id: Uuid,
    /// Ordered quantity
    pub quantity: i32,
    /// Unit price in cents
    pub unit_price: i64,
    /// Line total in cents
    pub line_total: i64,
}

/// Shipping address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Event envelope for NATS messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    /// Event type identifier
    pub event_type: String,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event version for backward compatibility
    pub version: String,
    /// Event data
    pub data: T,
}

impl<T> EventEnvelope<T> {
    pub fn new(event_type: &str, data: T) -> Self {
        Self {
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now(),
            version: "1.0".to_string(),
            data,
        }
    }
}
