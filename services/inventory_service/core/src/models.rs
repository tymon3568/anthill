use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
pub enum DeliveryOrderStatus {
    Draft,
    Confirmed,
    PartiallyPicked,
    Picked,
    Packed,
    PartiallyShipped,
    Shipped,
    Cancelled,
}

impl fmt::Display for DeliveryOrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DeliveryOrderStatus::Draft => "Draft",
            DeliveryOrderStatus::Confirmed => "Confirmed",
            DeliveryOrderStatus::PartiallyPicked => "PartiallyPicked",
            DeliveryOrderStatus::Picked => "Picked",
            DeliveryOrderStatus::Packed => "Packed",
            DeliveryOrderStatus::PartiallyShipped => "PartiallyShipped",
            DeliveryOrderStatus::Shipped => "Shipped",
            DeliveryOrderStatus::Cancelled => "Cancelled",
        };
        f.write_str(s)
    }
}

impl FromStr for DeliveryOrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(DeliveryOrderStatus::Draft),
            "Confirmed" => Ok(DeliveryOrderStatus::Confirmed),
            "PartiallyPicked" => Ok(DeliveryOrderStatus::PartiallyPicked),
            "Picked" => Ok(DeliveryOrderStatus::Picked),
            "Packed" => Ok(DeliveryOrderStatus::Packed),
            "PartiallyShipped" => Ok(DeliveryOrderStatus::PartiallyShipped),
            "Shipped" => Ok(DeliveryOrderStatus::Shipped),
            "Cancelled" => Ok(DeliveryOrderStatus::Cancelled),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrder {
    pub delivery_id: Uuid,
    pub tenant_id: Uuid,
    pub delivery_number: String,
    pub reference_number: Option<String>,
    pub warehouse_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub status: DeliveryOrderStatus,
    pub delivery_date: DateTime<Utc>,
    pub expected_ship_date: Option<DateTime<Utc>>,
    pub actual_ship_date: Option<DateTime<Utc>>,
    pub shipping_method: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub shipping_cost: Option<i64>, // in cents
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub total_quantity: i64,
    pub total_value: i64,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderItem {
    pub delivery_item_id: Uuid,
    pub delivery_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub ordered_quantity: i64,
    pub picked_quantity: i64,
    pub delivered_quantity: i64,
    pub uom_id: Option<Uuid>,
    pub batch_number: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub unit_price: Option<i64>,
    pub line_total: Option<i64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeliveryOrderRequest {
    pub reference_number: Option<String>,
    pub warehouse_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub delivery_date: DateTime<Utc>,
    pub expected_ship_date: Option<DateTime<Utc>>,
    pub shipping_method: Option<String>,
    pub carrier: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateDeliveryOrderItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeliveryOrderItemRequest {
    pub product_id: Uuid,
    pub ordered_quantity: i64,
    pub unit_price: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderResponse {
    pub delivery_id: Uuid,
    pub tenant_id: Uuid,
    pub delivery_number: String,
    pub reference_number: Option<String>,
    pub warehouse_id: Uuid,
    pub order_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub status: DeliveryOrderStatus,
    pub delivery_date: DateTime<Utc>,
    pub expected_ship_date: Option<DateTime<Utc>>,
    pub actual_ship_date: Option<DateTime<Utc>>,
    pub shipping_method: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub shipping_cost: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub total_quantity: i64,
    pub total_value: i64,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOrderItemResponse {
    pub delivery_item_id: Uuid,
    pub delivery_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub ordered_quantity: i64,
    pub picked_quantity: i64,
    pub delivered_quantity: i64,
    pub uom_id: Option<Uuid>,
    pub batch_number: Option<String>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub unit_price: Option<i64>,
    pub line_total: Option<i64>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMove {
    pub move_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub source_location_id: Option<Uuid>,
    pub destination_location_id: Option<Uuid>,
    pub move_type: String,
    pub quantity: i64,
    pub unit_cost: Option<i64>,
    pub total_cost: Option<i64>,
    pub reference_type: String,
    pub reference_id: Uuid,
    pub idempotency_key: String,
    pub move_date: DateTime<Utc>,
    pub move_reason: Option<String>,
    pub batch_info: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLevel {
    pub inventory_id: Uuid,
    pub tenant_id: Uuid,
    pub warehouse_id: Uuid,
    pub product_id: Uuid,
    pub available_quantity: i64,
    pub reserved_quantity: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockMoveRequest {
    pub product_id: Uuid,
    pub source_location_id: Option<Uuid>,
    pub destination_location_id: Option<Uuid>,
    pub move_type: String,
    pub quantity: i64,
    pub unit_cost: Option<i64>,
    pub reference_type: String,
    pub reference_id: Uuid,
    pub idempotency_key: String,
    pub move_reason: Option<String>,
    pub batch_info: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}
