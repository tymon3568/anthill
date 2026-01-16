//! Landed Cost DTOs for API communication
//!
//! Data transfer objects for landed cost operations,
//! supporting create, add-line, compute, post, and get operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domains::inventory::landed_cost::{
    AllocationMethod, CostType, LandedCostStatus, TargetType,
};

// ============================================================================
// Response DTOs
// ============================================================================

/// Landed cost document response DTO
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandedCostDto {
    pub landed_cost_id: Uuid,
    pub tenant_id: Uuid,
    pub reference: Option<String>,
    pub status: LandedCostStatus,
    pub grn_id: Option<Uuid>,
    pub notes: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Total of all cost lines in cents
    pub total_amount_cents: i64,
    /// Number of cost lines
    pub line_count: i64,
}

/// Landed cost line response DTO
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandedCostLineDto {
    pub landed_cost_line_id: Uuid,
    pub landed_cost_id: Uuid,
    pub cost_type: CostType,
    pub description: Option<String>,
    pub amount_cents: i64,
    pub allocation_method: AllocationMethod,
    pub created_at: DateTime<Utc>,
}

/// Landed cost allocation response DTO
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandedCostAllocationDto {
    pub landed_cost_allocation_id: Uuid,
    pub landed_cost_id: Uuid,
    pub landed_cost_line_id: Uuid,
    pub target_type: TargetType,
    pub target_id: Uuid,
    pub allocated_amount_cents: i64,
    pub created_at: DateTime<Utc>,
}

/// Full landed cost response with lines and allocations
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandedCostDetailDto {
    #[serde(flatten)]
    pub landed_cost: LandedCostDto,
    pub lines: Vec<LandedCostLineDto>,
    pub allocations: Vec<LandedCostAllocationDto>,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new landed cost document
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct CreateLandedCostRequest {
    /// Optional reference number
    pub reference: Option<String>,
    /// Optional link to goods receipt
    pub grn_id: Option<Uuid>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Request to add a cost line to a landed cost
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct AddLandedCostLineRequest {
    /// Type of cost
    pub cost_type: CostType,
    /// Optional description
    pub description: Option<String>,
    /// Amount in cents (must be positive)
    pub amount_cents: i64,
    /// Allocation method (defaults to by_value)
    #[serde(default)]
    pub allocation_method: AllocationMethod,
}

/// Request to compute allocations for a landed cost
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeAllocationsRequest {
    /// Target type for allocation
    #[serde(default = "default_target_type")]
    pub target_type: TargetType,
}

fn default_target_type() -> TargetType {
    TargetType::GrnItem
}

/// Request to post a landed cost
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct PostLandedCostRequest {
    /// Optional idempotency key for retry safety
    pub idempotency_key: Option<String>,
}

/// Request to get landed cost by ID (internal use)
#[derive(Debug, Clone)]
pub struct GetLandedCostRequest {
    pub tenant_id: Uuid,
    pub landed_cost_id: Uuid,
}

/// Request to list landed costs
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct ListLandedCostsRequest {
    /// Filter by status
    pub status: Option<LandedCostStatus>,
    /// Filter by GRN ID
    pub grn_id: Option<Uuid>,
    /// Pagination limit
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Pagination offset
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Response for list of landed costs
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct ListLandedCostsResponse {
    pub items: Vec<LandedCostDto>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Response for compute allocations
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct ComputeAllocationsResponse {
    pub landed_cost_id: Uuid,
    pub allocations_count: i64,
    pub total_allocated_cents: i64,
    pub allocations: Vec<LandedCostAllocationDto>,
}

/// Response for post landed cost
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct PostLandedCostResponse {
    pub landed_cost_id: Uuid,
    pub status: LandedCostStatus,
    pub posted_at: DateTime<Utc>,
    /// Number of valuation adjustments created
    pub adjustments_created: i64,
}
