//! Landed Cost domain models.
//!
//! This module defines the core domain entities for landed cost management,
//! including documents, cost lines, and allocations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a landed cost document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LandedCostStatus {
    /// Document is in draft state, can be modified
    Draft,
    /// Document has been posted, allocations are applied
    Posted,
    /// Document has been cancelled
    Cancelled,
}

impl LandedCostStatus {
    /// Convert status to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Posted => "posted",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parse status from database string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "posted" => Some(Self::Posted),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

impl std::fmt::Display for LandedCostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Method for allocating landed costs to receipt items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AllocationMethod {
    /// Allocate proportionally based on line value
    ByValue,
    /// Allocate proportionally based on quantity
    ByQuantity,
    /// Allocate equally across all lines
    Equal,
}

impl AllocationMethod {
    /// Convert method to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ByValue => "by_value",
            Self::ByQuantity => "by_quantity",
            Self::Equal => "equal",
        }
    }

    /// Parse method from database string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "by_value" => Some(Self::ByValue),
            "by_quantity" => Some(Self::ByQuantity),
            "equal" => Some(Self::Equal),
            _ => None,
        }
    }
}

impl std::fmt::Display for AllocationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Common cost types for landed costs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum CostType {
    /// Freight/shipping costs
    Freight,
    /// Customs duties
    Customs,
    /// Handling fees
    Handling,
    /// Insurance costs
    Insurance,
    /// Other miscellaneous costs
    Other,
}

impl CostType {
    /// Convert cost type to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Freight => "freight",
            Self::Customs => "customs",
            Self::Handling => "handling",
            Self::Insurance => "insurance",
            Self::Other => "other",
        }
    }

    /// Parse cost type from database string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "freight" => Some(Self::Freight),
            "customs" => Some(Self::Customs),
            "handling" => Some(Self::Handling),
            "insurance" => Some(Self::Insurance),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

impl std::fmt::Display for CostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A landed cost document.
///
/// Groups additional costs that need to be allocated to receipt lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostDocument {
    /// Unique document identifier
    pub document_id: Uuid,
    /// Tenant identifier
    pub tenant_id: Uuid,
    /// Human-readable document number (e.g., LC-2026-00001)
    pub document_number: String,
    /// External reference number (e.g., vendor invoice)
    pub reference_number: Option<String>,
    /// Document status
    pub status: LandedCostStatus,
    /// Associated goods receipt ID
    pub receipt_id: Uuid,
    /// Total cost amount in cents
    pub total_cost_amount: i64,
    /// Currency code (ISO 4217)
    pub currency_code: String,
    /// Allocation method for distributing costs
    pub allocation_method: AllocationMethod,
    /// Document date
    pub document_date: DateTime<Utc>,
    /// When the document was posted (if posted)
    pub posted_at: Option<DateTime<Utc>>,
    /// Additional notes
    pub notes: Option<String>,
    /// User who created the document
    pub created_by: Uuid,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// A cost line within a landed cost document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostLine {
    /// Unique line identifier
    pub line_id: Uuid,
    /// Tenant identifier
    pub tenant_id: Uuid,
    /// Parent document identifier
    pub document_id: Uuid,
    /// Type of cost
    pub cost_type: String,
    /// Description of the cost
    pub description: Option<String>,
    /// Cost amount in cents
    pub amount: i64,
    /// Vendor reference for this cost
    pub vendor_reference: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// An allocation record showing how costs were distributed to a receipt item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostAllocation {
    /// Unique allocation identifier
    pub allocation_id: Uuid,
    /// Tenant identifier
    pub tenant_id: Uuid,
    /// Parent document identifier
    pub document_id: Uuid,
    /// Receipt item receiving the allocation
    pub receipt_item_id: Uuid,
    /// Amount allocated in cents
    pub allocated_amount: i64,
    /// Unit cost before allocation
    pub original_unit_cost: i64,
    /// Unit cost after allocation
    pub new_unit_cost: i64,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Summary of a landed cost document with its lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostDocumentWithLines {
    /// The document
    pub document: LandedCostDocument,
    /// Cost lines
    pub lines: Vec<LandedCostLine>,
    /// Allocations (only present if document is posted)
    pub allocations: Option<Vec<LandedCostAllocation>>,
}
