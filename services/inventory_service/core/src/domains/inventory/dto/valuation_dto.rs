//! Valuation DTOs for API communication
//!
//! Data transfer objects for inventory valuation operations,
//! supporting FIFO, AVCO, and Standard costing methods.

use crate::domains::inventory::valuation::ValuationMethod;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current valuation information for a product
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationDto {
    pub valuation_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub valuation_method: ValuationMethod,
    pub current_unit_cost: Option<i64>, // In cents
    pub total_quantity: i64,
    pub total_value: i64,           // In cents
    pub standard_cost: Option<i64>, // In cents, only for standard method
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Cost layer for FIFO valuation
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationLayerDto {
    pub layer_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i64,
    pub unit_cost: i64,   // In cents
    pub total_value: i64, // In cents
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Historical valuation record
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationHistoryDto {
    pub history_id: Uuid,
    pub valuation_id: Uuid,
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub valuation_method: ValuationMethod,
    pub unit_cost: Option<i64>,
    pub total_quantity: i64,
    pub total_value: i64,
    pub standard_cost: Option<i64>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub change_reason: Option<String>,
}

/// Request to get current valuation
#[derive(Debug, Clone, Deserialize)]
pub struct GetValuationRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
}

/// Request to set valuation method
#[derive(Debug, Clone, Deserialize)]
pub struct SetValuationMethodRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub valuation_method: ValuationMethod,
}

/// Request to set standard cost
#[derive(Debug, Clone, Deserialize)]
pub struct SetStandardCostRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub standard_cost: i64, // In cents
}

/// Request to get valuation layers (for FIFO)
#[derive(Debug, Clone, Deserialize)]
pub struct GetValuationLayersRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
}

/// Request to get valuation history
#[derive(Debug, Clone, Deserialize)]
pub struct GetValuationHistoryRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Response for valuation layers
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct ValuationLayersResponse {
    pub layers: Vec<ValuationLayerDto>,
}

/// Response for valuation history
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct ValuationHistoryResponse {
    pub history: Vec<ValuationHistoryDto>,
    pub total_count: i64,
}

/// Request to perform cost adjustment
#[derive(Debug, Clone, Deserialize)]
pub struct CostAdjustmentRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub adjustment_amount: i64, // In cents, positive or negative
    pub reason: String,
}

/// Request to revalue inventory
#[derive(Debug, Clone, Deserialize)]
pub struct RevaluationRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub new_unit_cost: i64, // In cents
    pub reason: String,
}
