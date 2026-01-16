//! Valuation DTOs for API communication
//!
//! Data transfer objects for inventory valuation operations,
//! supporting FIFO, AVCO, and Standard costing methods.

use crate::domains::inventory::valuation::{ValuationMethod, ValuationScopeType};
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

// ============================================
// Valuation Settings DTOs
// ============================================

/// Valuation settings data transfer object
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationSettingsDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub scope_type: ValuationScopeType,
    pub scope_id: Option<Uuid>,
    pub method: ValuationMethod,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to get tenant default valuation settings
#[derive(Debug, Clone, Deserialize)]
pub struct GetTenantValuationSettingsRequest {
    pub tenant_id: Uuid,
}

/// Request to get effective valuation method for a product
/// Returns the method considering the hierarchy: product > category > tenant
#[derive(Debug, Clone, Deserialize)]
pub struct GetEffectiveValuationMethodRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub category_id: Option<Uuid>,
}

/// Response for effective valuation method
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveValuationMethodResponse {
    pub method: ValuationMethod,
    pub source: ValuationScopeType,
    pub source_id: Option<Uuid>,
}

/// Request to set tenant default valuation method
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct SetTenantValuationMethodRequest {
    pub tenant_id: Uuid,
    pub method: ValuationMethod,
}

/// Request to set category valuation method override
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct SetCategoryValuationMethodRequest {
    pub tenant_id: Uuid,
    pub category_id: Uuid,
    pub method: ValuationMethod,
}

/// Request to set product valuation method override
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct SetProductValuationMethodRequest {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub method: ValuationMethod,
}

/// Request to delete a valuation settings override
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteValuationSettingsRequest {
    pub tenant_id: Uuid,
    pub scope_type: ValuationScopeType,
    pub scope_id: Option<Uuid>,
}

/// Request to list all valuation settings for a tenant
#[derive(Debug, Clone, Deserialize)]
pub struct ListValuationSettingsRequest {
    pub tenant_id: Uuid,
    pub scope_type: Option<ValuationScopeType>,
}

/// Response for listing valuation settings
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct ValuationSettingsListResponse {
    pub settings: Vec<ValuationSettingsDto>,
}
