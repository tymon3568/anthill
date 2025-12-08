use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::{
    validate_config_not_empty, validate_picking_method_type,
};

/// Request DTO for creating a new picking method
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreatePickingMethodRequest {
    /// Method name
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    /// Optional description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Picking method type
    #[validate(custom(function = "validate_picking_method_type"))]
    pub method_type: String,

    /// Warehouse ID this method applies to
    pub warehouse_id: Uuid,

    /// Method configuration (JSON)
    #[validate(custom(function = "validate_config_not_empty"))]
    pub config: serde_json::Value,

    /// Whether this should be the default method
    pub is_default: Option<bool>,
}

/// Request DTO for updating a picking method
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct UpdatePickingMethodRequest {
    /// Method name
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,

    /// Description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Method configuration (JSON)
    pub config: Option<serde_json::Value>,

    /// Whether this should be the default method
    pub is_default: Option<bool>,

    /// Whether method is active
    pub is_active: Option<bool>,
}

/// Picking method response DTO
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PickingMethodResponse {
    /// Method ID
    pub method_id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Method metadata
    pub name: String,
    pub description: Option<String>,

    /// Method type
    pub method_type: String,

    /// Warehouse ID
    pub warehouse_id: Uuid,

    /// Configuration
    pub config: serde_json::Value,

    /// Status flags
    pub is_active: bool,
    pub is_default: bool,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Picking optimization request DTO
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PickingOptimizationRequest {
    /// Warehouse ID
    pub warehouse_id: Uuid,

    /// Order IDs to optimize picking for
    #[validate(length(min = 1, max = 1000))]
    pub order_ids: Vec<Uuid>,

    /// Optimization criteria
    #[validate(length(min = 1, max = 10))]
    pub criteria: Vec<String>,

    /// Optional picking method ID to use
    pub method_id: Option<Uuid>,

    /// Additional constraints
    pub constraints: Option<serde_json::Value>,
}

/// Picking plan response DTO
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PickingPlanResponse {
    /// Plan ID
    pub plan_id: Uuid,

    /// Method used
    pub method_id: Uuid,
    pub method_name: String,
    pub method_type: String,

    /// Warehouse ID
    pub warehouse_id: Uuid,

    /// Order IDs included
    pub order_ids: Vec<Uuid>,

    /// Optimized picking tasks
    pub tasks: Vec<PickingTask>,

    /// Optimization metrics
    pub metrics: PickingMetrics,

    /// Generated timestamp
    pub generated_at: DateTime<Utc>,
}

/// Individual picking task
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PickingTask {
    /// Task ID
    pub task_id: Uuid,

    /// Order ID this task belongs to
    pub order_id: Uuid,

    /// Product to pick
    pub product_id: Uuid,
    pub product_code: String,
    pub product_name: String,

    /// Quantity to pick
    pub quantity: i64,

    /// Location to pick from
    pub location_id: Uuid,
    pub location_code: String,

    /// Task sequence/order
    pub sequence: i32,

    /// Estimated time (seconds)
    pub estimated_time_seconds: Option<i32>,
}

/// Picking optimization metrics
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PickingMetrics {
    /// Total distance (meters)
    pub total_distance_meters: Option<f64>,

    /// Total estimated time (seconds)
    pub total_estimated_time_seconds: Option<i32>,

    /// Number of tasks
    pub task_count: u32,

    /// Efficiency score (0-100)
    pub efficiency_score: Option<f64>,

    /// Travel time reduction percentage
    pub travel_time_reduction_percent: Option<f64>,
}

/// Picking plan confirmation request
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPickingPlanRequest {
    /// Plan ID to confirm
    pub plan_id: Uuid,

    /// Optional notes
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
}

impl From<crate::domains::inventory::picking_method::PickingMethod> for PickingMethodResponse {
    fn from(method: crate::domains::inventory::picking_method::PickingMethod) -> Self {
        Self {
            method_id: method.method_id,
            tenant_id: method.tenant_id,
            name: method.name,
            description: method.description,
            method_type: method.method_type,
            warehouse_id: method.warehouse_id,
            config: method.config,
            is_active: method.is_active,
            is_default: method.is_default,
            created_at: method.created_at,
            updated_at: method.updated_at,
        }
    }
}
