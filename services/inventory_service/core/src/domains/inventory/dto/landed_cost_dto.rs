//! Landed Cost DTOs.
//!
//! Data transfer objects for landed cost operations.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domains::inventory::landed_cost::{
    AllocationMethod, LandedCostAllocation, LandedCostDocument, LandedCostDocumentWithLines,
    LandedCostLine, LandedCostStatus,
};

// =============================================================================
// Request DTOs
// =============================================================================

/// Request to create a new landed cost document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateLandedCostDocumentRequest {
    /// Associated goods receipt ID
    pub receipt_id: Uuid,
    /// External reference number (optional)
    pub reference_number: Option<String>,
    /// Allocation method (defaults to by_value)
    #[serde(default = "default_allocation_method")]
    pub allocation_method: AllocationMethod,
    /// Currency code (defaults to VND)
    #[serde(default = "default_currency")]
    pub currency_code: String,
    /// Additional notes
    pub notes: Option<String>,
    /// Initial cost lines (optional, can add later)
    #[serde(default)]
    pub lines: Vec<CreateLandedCostLineRequest>,
}

fn default_allocation_method() -> AllocationMethod {
    AllocationMethod::ByValue
}

fn default_currency() -> String {
    "VND".to_string()
}

/// Request to create a cost line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateLandedCostLineRequest {
    /// Type of cost (freight, customs, handling, insurance, other)
    pub cost_type: String,
    /// Description of the cost
    pub description: Option<String>,
    /// Cost amount in cents
    pub amount: i64,
    /// Vendor reference for this cost
    pub vendor_reference: Option<String>,
}

/// Request to update a landed cost document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateLandedCostDocumentRequest {
    /// External reference number
    pub reference_number: Option<String>,
    /// Allocation method
    pub allocation_method: Option<AllocationMethod>,
    /// Additional notes
    pub notes: Option<String>,
}

/// Request to update a cost line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateLandedCostLineRequest {
    /// Type of cost
    pub cost_type: Option<String>,
    /// Description of the cost
    pub description: Option<String>,
    /// Cost amount in cents
    pub amount: Option<i64>,
    /// Vendor reference
    pub vendor_reference: Option<String>,
}

// NOTE: AddLandedCostLineRequest was removed - use CreateLandedCostLineRequest instead.
// Both DTOs had identical fields, so we consolidated to reduce code duplication.

/// Request to post a landed cost document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PostLandedCostRequest {
    /// Document ID to post
    pub document_id: Uuid,
}

/// Request to list landed cost documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListLandedCostDocumentsRequest {
    /// Filter by status
    pub status: Option<LandedCostStatus>,
    /// Filter by receipt ID
    pub receipt_id: Option<Uuid>,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i32,
    /// Items per page
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

/// Request to get allocation preview before posting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetAllocationPreviewRequest {
    /// Document ID to preview
    pub document_id: Uuid,
}

// =============================================================================
// Response DTOs
// =============================================================================

/// Response for a single landed cost document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostDocumentDto {
    /// Document details
    #[serde(flatten)]
    pub document: LandedCostDocument,
}

/// Response for a landed cost document with lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostDocumentWithLinesDto {
    /// Document with lines
    #[serde(flatten)]
    pub data: LandedCostDocumentWithLines,
}

/// Response for a list of landed cost documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostDocumentListResponse {
    /// List of documents
    pub documents: Vec<LandedCostDocument>,
    /// Total count
    pub total: i64,
    /// Current page
    pub page: i32,
    /// Items per page
    pub page_size: i32,
}

/// Response for a cost line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostLineDto {
    /// Line details
    #[serde(flatten)]
    pub line: LandedCostLine,
}

/// Allocation preview item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllocationPreviewItem {
    /// Receipt item ID
    pub receipt_item_id: Uuid,
    /// Product ID
    pub product_id: Uuid,
    /// Product name (if available)
    pub product_name: Option<String>,
    /// Original quantity
    pub quantity: i64,
    /// Original line value
    pub original_line_value: i64,
    /// Original unit cost
    pub original_unit_cost: i64,
    /// Allocated amount
    pub allocated_amount: i64,
    /// New unit cost after allocation
    pub new_unit_cost: i64,
}

/// Response for allocation preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllocationPreviewResponse {
    /// Document ID
    pub document_id: Uuid,
    /// Total cost to allocate
    pub total_cost_amount: i64,
    /// Allocation method
    pub allocation_method: AllocationMethod,
    /// Preview of allocations
    pub allocations: Vec<AllocationPreviewItem>,
}

/// Response for posting a landed cost document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PostLandedCostResponse {
    /// Document ID
    pub document_id: Uuid,
    /// New status
    pub status: LandedCostStatus,
    /// Posted timestamp
    pub posted_at: chrono::DateTime<chrono::Utc>,
    /// Created allocations
    pub allocations: Vec<LandedCostAllocation>,
    /// Number of receipt items affected
    pub items_affected: i32,
}

impl From<LandedCostDocument> for LandedCostDocumentDto {
    fn from(document: LandedCostDocument) -> Self {
        Self { document }
    }
}

impl From<LandedCostDocumentWithLines> for LandedCostDocumentWithLinesDto {
    fn from(data: LandedCostDocumentWithLines) -> Self {
        Self { data }
    }
}

impl From<LandedCostLine> for LandedCostLineDto {
    fn from(line: LandedCostLine) -> Self {
        Self { line }
    }
}
