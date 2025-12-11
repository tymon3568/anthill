use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Reorder rule for automated stock replenishment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReorderRule {
    pub rule_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Option<Uuid>,
    pub reorder_point: i64,
    pub min_quantity: i64,
    pub max_quantity: i64,
    pub lead_time_days: i32,
    pub safety_stock: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// DTO for creating a new reorder rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateReorderRule {
    pub product_id: Uuid,
    pub warehouse_id: Option<Uuid>,
    pub reorder_point: i64,
    pub min_quantity: i64,
    pub max_quantity: i64,
    pub lead_time_days: i32,
    pub safety_stock: i64,
}

/// DTO for updating a reorder rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct UpdateReorderRule {
    pub reorder_point: Option<i64>,
    pub min_quantity: Option<i64>,
    pub max_quantity: Option<i64>,
    pub lead_time_days: Option<i32>,
    pub safety_stock: Option<i64>,
}

/// Result of replenishment check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReplenishmentCheckResult {
    pub product_id: Uuid,
    pub warehouse_id: Option<Uuid>,
    pub current_quantity: i64,
    pub projected_quantity: i64,
    pub reorder_point: i64,
    pub suggested_order_quantity: i64,
    pub needs_replenishment: bool,
    pub action_taken: Option<String>,
}
