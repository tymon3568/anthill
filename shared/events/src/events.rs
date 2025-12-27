use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_type: String,
    pub data: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> EventEnvelope<T> {
    pub fn new(event_type: &str, data: T) -> Self {
        Self {
            event_type: event_type.to_string(),
            data,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfirmedEvent {
    pub order_id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Uuid,
    pub expected_delivery_date: Option<chrono::DateTime<chrono::Utc>>,
    pub items: Vec<OrderItem>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub quantity: i64,
    pub unit_price: i64,
    pub line_total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderTriggeredEvent {
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Option<Uuid>,
    pub current_quantity: i64,
    pub projected_quantity: i64,
    /// Effective reorder point (base reorder_point + safety_stock).
    pub reorder_point: i64,
    pub suggested_order_quantity: i64,
    pub rule_id: Uuid,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
}
