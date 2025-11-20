use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Delivery Order status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryOrderStatus {
    Draft,
    Confirmed,
    PartiallyShipped,
    Shipped,
    Cancelled,
    Reserved, // Added for stock reservation
}

/// Delivery Order entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrder {
    pub delivery_id: Uuid,
    pub tenant_id: Uuid,
    pub delivery_number: String,
    pub reference_number: Option<String>,
    pub warehouse_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub status: DeliveryOrderStatus,
    pub delivery_date: chrono::NaiveDate,
    pub expected_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub shipping_method: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub shipping_cost: Option<i64>, // in cents
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub total_quantity: i32,
    pub total_value: i64, // in cents
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Delivery Order Item entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderItem {
    pub delivery_item_id: Uuid,
    pub tenant_id: Uuid,
    pub delivery_id: Uuid,
    pub product_id: Uuid,
    pub ordered_quantity: i32,
    pub picked_quantity: i32,
    pub delivered_quantity: i32,
    pub unit_price: i64, // in cents
    pub line_total: i64, // in cents
    pub batch_number: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub uom_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Delivery Order creation request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeliveryOrderRequest {
    pub tenant_id: Uuid,
    pub reference_number: Option<String>,
    pub warehouse_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub expected_ship_date: Option<chrono::NaiveDate>,
    pub shipping_method: Option<String>,
    pub carrier: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateDeliveryOrderItemRequest>,
}

/// Delivery Order Item creation request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeliveryOrderItemRequest {
    pub product_id: Uuid,
    pub ordered_quantity: i32,
    pub unit_price: i64,
    pub batch_number: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub uom_id: Option<Uuid>,
    pub notes: Option<String>,
}

/// Delivery Order response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderResponse {
    pub delivery_id: Uuid,
    pub tenant_id: Uuid,
    pub delivery_number: String,
    pub reference_number: Option<String>,
    pub warehouse_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub status: DeliveryOrderStatus,
    pub delivery_date: chrono::NaiveDate,
    pub expected_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub shipping_method: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub shipping_cost: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub total_quantity: i32,
    pub total_value: i64,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<DeliveryOrderItemResponse>,
}

/// Delivery Order Item response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderItemResponse {
    pub delivery_item_id: Uuid,
    pub tenant_id: Uuid,
    pub delivery_id: Uuid,
    pub product_id: Uuid,
    pub ordered_quantity: i32,
    pub picked_quantity: i32,
    pub delivered_quantity: i32,
    pub unit_price: i64,
    pub line_total: i64,
    pub batch_number: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub uom_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
