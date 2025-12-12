use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum PutawayRuleType {
    Product,
    Category,
    Attribute,
    Fifo,
    Fefo,
}

impl fmt::Display for PutawayRuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PutawayRuleType::Product => "product",
            PutawayRuleType::Category => "category",
            PutawayRuleType::Attribute => "attribute",
            PutawayRuleType::Fifo => "fifo",
            PutawayRuleType::Fefo => "fefo",
        };
        f.write_str(s)
    }
}

impl FromStr for PutawayRuleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "product" => Ok(PutawayRuleType::Product),
            "category" => Ok(PutawayRuleType::Category),
            "attribute" => Ok(PutawayRuleType::Attribute),
            "fifo" => Ok(PutawayRuleType::Fifo),
            "fefo" => Ok(PutawayRuleType::Fefo),
            _ => Err(format!("Unknown putaway rule type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum PutawayMatchMode {
    Exact,
    Contains,
    Regex,
}

impl fmt::Display for PutawayMatchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PutawayMatchMode::Exact => "exact",
            PutawayMatchMode::Contains => "contains",
            PutawayMatchMode::Regex => "regex",
        };
        f.write_str(s)
    }
}

impl FromStr for PutawayMatchMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exact" => Ok(PutawayMatchMode::Exact),
            "contains" => Ok(PutawayMatchMode::Contains),
            "regex" => Ok(PutawayMatchMode::Regex),
            _ => Err(format!("Unknown match mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
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
            DeliveryOrderStatus::Draft => "draft",
            DeliveryOrderStatus::Confirmed => "confirmed",
            DeliveryOrderStatus::PartiallyPicked => "partially_picked",
            DeliveryOrderStatus::Picked => "picked",
            DeliveryOrderStatus::Packed => "packed",
            DeliveryOrderStatus::PartiallyShipped => "partially_shipped",
            DeliveryOrderStatus::Shipped => "shipped",
            DeliveryOrderStatus::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

impl FromStr for DeliveryOrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(DeliveryOrderStatus::Draft),
            "confirmed" => Ok(DeliveryOrderStatus::Confirmed),
            "partially_picked" => Ok(DeliveryOrderStatus::PartiallyPicked),
            "picked" => Ok(DeliveryOrderStatus::Picked),
            "packed" => Ok(DeliveryOrderStatus::Packed),
            "partially_shipped" => Ok(DeliveryOrderStatus::PartiallyShipped),
            "shipped" => Ok(DeliveryOrderStatus::Shipped),
            "cancelled" => Ok(DeliveryOrderStatus::Cancelled),
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
    pub expiry_date: Option<chrono::NaiveDate>,
    pub unit_price: Option<i64>,
    pub line_total: Option<i64>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
    pub lot_serial_id: Option<Uuid>,
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
    pub lot_serial_id: Option<Uuid>,
    pub idempotency_key: String,
    pub move_reason: Option<String>,
    pub batch_info: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RmaStatus {
    Draft,
    Approved,
    Received,
    Processed,
    Rejected,
}

impl fmt::Display for RmaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RmaStatus::Draft => "draft",
            RmaStatus::Approved => "approved",
            RmaStatus::Received => "received",
            RmaStatus::Processed => "processed",
            RmaStatus::Rejected => "rejected",
        };
        f.write_str(s)
    }
}

impl FromStr for RmaStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(RmaStatus::Draft),
            "approved" => Ok(RmaStatus::Approved),
            "received" => Ok(RmaStatus::Received),
            "processed" => Ok(RmaStatus::Processed),
            "rejected" => Ok(RmaStatus::Rejected),
            _ => Err(format!("Unknown RMA status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RmaCondition {
    New,
    Used,
    Damaged,
    Defective,
}

impl fmt::Display for RmaCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RmaCondition::New => "new",
            RmaCondition::Used => "used",
            RmaCondition::Damaged => "damaged",
            RmaCondition::Defective => "defective",
        };
        f.write_str(s)
    }
}

impl FromStr for RmaCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new" => Ok(RmaCondition::New),
            "used" => Ok(RmaCondition::Used),
            "damaged" => Ok(RmaCondition::Damaged),
            "defective" => Ok(RmaCondition::Defective),
            _ => Err(format!("Unknown RMA condition: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RmaAction {
    Restock,
    Scrap,
    Refund,
    Exchange,
}

impl fmt::Display for RmaAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RmaAction::Restock => "restock",
            RmaAction::Scrap => "scrap",
            RmaAction::Refund => "refund",
            RmaAction::Exchange => "exchange",
        };
        f.write_str(s)
    }
}

impl FromStr for RmaAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "restock" => Ok(RmaAction::Restock),
            "scrap" => Ok(RmaAction::Scrap),
            "refund" => Ok(RmaAction::Refund),
            "exchange" => Ok(RmaAction::Exchange),
            _ => Err(format!("Unknown RMA action: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "lot_serial_tracking_type", rename_all = "snake_case")]
pub enum LotSerialTrackingType {
    Lot,
    Serial,
}

impl fmt::Display for LotSerialTrackingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LotSerialTrackingType::Lot => "lot",
            LotSerialTrackingType::Serial => "serial",
        };
        f.write_str(s)
    }
}

impl FromStr for LotSerialTrackingType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lot" => Ok(LotSerialTrackingType::Lot),
            "serial" => Ok(LotSerialTrackingType::Serial),
            _ => Err(format!("Unknown tracking type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "lot_serial_status", rename_all = "snake_case")]
pub enum LotSerialStatus {
    Active,
    Expired,
    Quarantined,
    Disposed,
    Reserved,
}

impl fmt::Display for LotSerialStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LotSerialStatus::Active => "active",
            LotSerialStatus::Expired => "expired",
            LotSerialStatus::Quarantined => "quarantined",
            LotSerialStatus::Disposed => "disposed",
            LotSerialStatus::Reserved => "reserved",
        };
        f.write_str(s)
    }
}

impl FromStr for LotSerialStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(LotSerialStatus::Active),
            "expired" => Ok(LotSerialStatus::Expired),
            "quarantined" => Ok(LotSerialStatus::Quarantined),
            "disposed" => Ok(LotSerialStatus::Disposed),
            "reserved" => Ok(LotSerialStatus::Reserved),
            _ => Err(format!("Unknown lot serial status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LotSerial {
    pub lot_serial_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub tracking_type: LotSerialTrackingType,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub initial_quantity: Option<i64>,
    pub remaining_quantity: Option<i64>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub status: LotSerialStatus,
    pub warehouse_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LotSerialLifecycle {
    pub lot_serial: LotSerial,
    /// TODO: Populate from supplier table via purchase order
    pub supplier_name: Option<String>,
    /// TODO: Populate from purchase order table
    pub purchase_order_number: Option<String>,
    /// TODO: Populate from quality documents or attachments
    pub coa_link: Option<String>,
    pub stock_moves: Vec<StockMove>,
    /// TODO: Populate from warehouse table using lot_serial.location_id
    pub current_warehouse_name: Option<String>,
    /// TODO: Populate from location table using lot_serial.location_id
    pub current_location_code: Option<String>,
    /// TODO: Populate from quality_checks table or related records
    pub quality_checks: Vec<serde_json::Value>, // Placeholder for quality check records
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseZone {
    pub tenant_id: Uuid,
    pub zone_id: Uuid,
    pub zone_name: String,
    pub warehouse_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseLocation {
    pub tenant_id: Uuid,
    pub location_id: Uuid,
    pub location_code: String,
    pub zone_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmaRequest {
    pub rma_id: Uuid,
    pub rma_number: String,
    pub tenant_id: Uuid,
    pub customer_id: Uuid,
    pub original_delivery_id: Uuid,
    pub status: RmaStatus,
    pub return_reason: Option<String>,
    pub notes: Option<String>,
    pub total_items: i32,
    pub total_value: i64,
    pub currency_code: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmaItem {
    pub rma_item_id: Uuid,
    pub tenant_id: Uuid,
    pub rma_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_returned: i64,
    pub condition: RmaCondition,
    pub action: RmaAction,
    pub unit_cost: Option<i64>,
    pub line_total: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRmaRequest {
    pub customer_id: Uuid,
    pub original_delivery_id: Uuid,
    pub return_reason: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateRmaItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRmaItemRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_returned: i64,
    pub condition: RmaCondition,
    pub action: RmaAction,
    pub unit_cost: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmaRequestResponse {
    pub rma_id: Uuid,
    pub rma_number: String,
    pub tenant_id: Uuid,
    pub customer_id: Uuid,
    pub original_delivery_id: Uuid,
    pub status: RmaStatus,
    pub return_reason: Option<String>,
    pub notes: Option<String>,
    pub total_items: i32,
    pub total_value: i64,
    pub currency_code: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmaItemResponse {
    pub rma_item_id: Uuid,
    pub tenant_id: Uuid,
    pub rma_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_returned: i64,
    pub condition: RmaCondition,
    pub action: RmaAction,
    pub unit_cost: Option<i64>,
    pub line_total: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveRmaRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveRmaResponse {
    pub rma_id: Uuid,
    pub status: RmaStatus,
    pub approved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveRmaRequest {
    pub received_items: Vec<ReceiveRmaItemRequest>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveRmaItemRequest {
    pub rma_item_id: Uuid,
    pub received_quantity: i64,
    pub condition: RmaCondition,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveRmaResponse {
    pub rma_id: Uuid,
    pub status: RmaStatus,
    pub received_at: DateTime<Utc>,
    pub stock_moves_created: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StorageLocation {
    pub location_id: Uuid,
    pub tenant_id: Uuid,
    pub warehouse_id: Uuid,
    pub location_code: String,
    pub location_type: String,
    pub zone: Option<String>,
    pub aisle: Option<String>,
    pub rack: Option<String>,
    pub level: Option<i32>,
    pub position: Option<i32>,
    pub capacity: Option<i64>,
    pub current_stock: i64,
    pub is_active: bool,
    pub is_quarantine: bool,
    pub is_picking_location: bool,
    pub length_cm: Option<i32>,
    pub width_cm: Option<i32>,
    pub height_cm: Option<i32>,
    pub weight_limit_kg: Option<i32>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PutawayRule {
    pub rule_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sequence: i32,
    pub product_id: Option<Uuid>,
    pub product_category_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub preferred_location_type: Option<String>,
    pub preferred_zone: Option<String>,
    pub preferred_aisle: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub rule_type: PutawayRuleType,
    pub match_mode: PutawayMatchMode,
    pub max_quantity: Option<i64>,
    pub min_quantity: Option<i64>,
    pub priority_score: i32,
    pub is_active: bool,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PutawaySuggestion {
    pub location_id: Uuid,
    pub location_code: String,
    pub warehouse_id: Uuid,
    pub zone: Option<String>,
    pub aisle: Option<String>,
    pub rack: Option<String>,
    pub level: Option<i32>,
    pub position: Option<i32>,
    pub available_capacity: Option<i64>,
    pub current_stock: i64,
    pub score: i32,
    pub rule_applied: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PutawayRequest {
    pub product_id: Uuid,
    pub product_category_id: Option<Uuid>,
    pub quantity: i64,
    pub warehouse_id: Option<Uuid>,
    pub preferred_location_type: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PutawayResponse {
    pub suggestions: Vec<PutawaySuggestion>,
    pub total_quantity: i64,
    pub allocated_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfirmPutawayRequest {
    pub product_id: Uuid,
    pub allocations: Vec<PutawayAllocation>,
    pub reference_type: String,
    pub reference_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PutawayAllocation {
    pub location_id: Uuid,
    pub quantity: i64,
    pub unit_cost: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfirmPutawayResponse {
    pub stock_moves_created: Vec<Uuid>,
    pub total_quantity_putaway: i64,
}
