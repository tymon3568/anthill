//! Stock Adjustment Data Transfer Objects
//!
//! This module contains request and response DTOs for stock adjustment operations,
//! following the 3-crate pattern with zero infrastructure dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Adjustment Status Enum
// ============================================================================

/// Status of an adjustment document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AdjustmentStatus {
    /// Draft - can be edited, lines can be added
    #[default]
    Draft,
    /// Posted - adjustment has been executed, inventory adjusted
    Posted,
    /// Cancelled - draft was cancelled before posting
    Cancelled,
}

impl std::fmt::Display for AdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustmentStatus::Draft => write!(f, "draft"),
            AdjustmentStatus::Posted => write!(f, "posted"),
            AdjustmentStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl AdjustmentStatus {
    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: AdjustmentStatus) -> bool {
        match (self, new_status) {
            // Draft can transition to Posted or Cancelled
            (AdjustmentStatus::Draft, AdjustmentStatus::Posted) => true,
            (AdjustmentStatus::Draft, AdjustmentStatus::Cancelled) => true,
            // Posted and Cancelled are terminal states
            _ => false,
        }
    }
}

// ============================================================================
// Adjustment Type Enum
// ============================================================================

/// Type of stock adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AdjustmentType {
    /// Increase stock quantity
    Increase,
    /// Decrease stock quantity
    #[default]
    Decrease,
}

impl std::fmt::Display for AdjustmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustmentType::Increase => write!(f, "increase"),
            AdjustmentType::Decrease => write!(f, "decrease"),
        }
    }
}

// ============================================================================
// Adjustment Reason Codes
// ============================================================================

/// Standard reason codes for adjusting inventory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AdjustmentReasonCode {
    /// Product is damaged
    Damaged,
    /// Product is lost/missing
    Lost,
    /// Product found (for increases)
    Found,
    /// Physical count correction
    CountCorrection,
    /// System error correction
    SystemCorrection,
    /// Product has expired
    Expired,
    /// Suspected theft
    Theft,
    /// Used for promotion/sample
    Promotion,
    /// Returned to stock
    ReturnToStock,
    /// Other reason (use notes field for details)
    Other,
}

impl std::fmt::Display for AdjustmentReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustmentReasonCode::Damaged => write!(f, "damaged"),
            AdjustmentReasonCode::Lost => write!(f, "lost"),
            AdjustmentReasonCode::Found => write!(f, "found"),
            AdjustmentReasonCode::CountCorrection => write!(f, "count_correction"),
            AdjustmentReasonCode::SystemCorrection => write!(f, "system_correction"),
            AdjustmentReasonCode::Expired => write!(f, "expired"),
            AdjustmentReasonCode::Theft => write!(f, "theft"),
            AdjustmentReasonCode::Promotion => write!(f, "promotion"),
            AdjustmentReasonCode::ReturnToStock => write!(f, "return_to_stock"),
            AdjustmentReasonCode::Other => write!(f, "other"),
        }
    }
}

// ============================================================================
// Domain Entities
// ============================================================================

/// Stock adjustment document header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentDocument {
    /// Adjustment document ID
    pub adjustment_id: Uuid,
    /// Tenant ID for multi-tenancy isolation
    pub tenant_id: Uuid,
    /// Optional reference number (e.g., ADJ-2026-0001)
    pub reference: Option<String>,
    /// Document status
    pub status: AdjustmentStatus,
    /// Warehouse ID where adjustment applies
    pub warehouse_id: Uuid,
    /// Optional notes
    pub notes: Option<String>,
    /// User who created the document
    pub created_by: Option<Uuid>,
    /// User who posted the document
    pub posted_by: Option<Uuid>,
    /// When the document was posted
    pub posted_at: Option<DateTime<Utc>>,
    /// User who cancelled the document
    pub cancelled_by: Option<Uuid>,
    /// When the document was cancelled
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Stock adjustment line item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentLine {
    /// Line ID
    pub adjustment_line_id: Uuid,
    /// Tenant ID for multi-tenancy isolation
    pub tenant_id: Uuid,
    /// Parent adjustment document ID
    pub adjustment_id: Uuid,
    /// Product being adjusted
    pub product_id: Uuid,
    /// Optional variant ID
    pub variant_id: Option<Uuid>,
    /// Type of adjustment (increase/decrease)
    pub adjustment_type: AdjustmentType,
    /// Quantity to adjust (must be > 0)
    pub qty: i64,
    /// Reason code
    pub reason_code: AdjustmentReasonCode,
    /// Additional notes
    pub reason_notes: Option<String>,
    /// Optional location within warehouse
    pub location_id: Option<Uuid>,
    /// Optional lot ID if lot-tracked
    pub lot_id: Option<Uuid>,
    /// Optional serial ID if serial-tracked
    pub serial_id: Option<Uuid>,
    /// Stock move ID created on posting (if posted)
    pub posted_stock_move_id: Option<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new adjustment document (draft)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateAdjustmentRequest {
    /// Optional reference number
    pub reference: Option<String>,
    /// Warehouse ID (required)
    pub warehouse_id: Uuid,
    /// Optional notes
    pub notes: Option<String>,
    /// Initial lines (optional - can add later)
    #[validate(nested)]
    pub lines: Option<Vec<AdjustmentLineInput>>,
}

/// A single line to add to an adjustment document
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentLineInput {
    /// Product being adjusted
    pub product_id: Uuid,
    /// Optional variant ID
    pub variant_id: Option<Uuid>,
    /// Type of adjustment (increase/decrease)
    pub adjustment_type: AdjustmentType,
    /// Quantity to adjust (must be > 0)
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub qty: i64,
    /// Reason code
    pub reason_code: AdjustmentReasonCode,
    /// Additional notes
    pub reason_notes: Option<String>,
    /// Optional location within warehouse
    pub location_id: Option<Uuid>,
    /// Optional lot ID if lot-tracked
    pub lot_id: Option<Uuid>,
    /// Optional serial ID if serial-tracked
    pub serial_id: Option<Uuid>,
}

/// Request to add lines to an adjustment document
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AddAdjustmentLinesRequest {
    /// Lines to add
    #[validate(length(min = 1, message = "At least one line is required"))]
    #[validate(nested)]
    pub lines: Vec<AdjustmentLineInput>,
}

/// Request to post an adjustment document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PostAdjustmentRequest {
    /// Optional idempotency key for retry safety
    pub idempotency_key: Option<String>,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response for adjustment document operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentDocumentResponse {
    /// The adjustment document
    pub adjustment: AdjustmentDocument,
}

/// Response for adjustment document with lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentDocumentWithLinesResponse {
    /// The adjustment document
    pub adjustment: AdjustmentDocument,
    /// Associated lines
    pub lines: Vec<AdjustmentLine>,
}

/// Query parameters for listing adjustment documents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
pub struct AdjustmentListQuery {
    /// Filter by warehouse
    pub warehouse_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<AdjustmentStatus>,
    /// Filter by reason code
    pub reason_code: Option<AdjustmentReasonCode>,
    /// Search by reference or notes
    pub search: Option<String>,
    /// Filter by date from
    pub from_date: Option<DateTime<Utc>>,
    /// Filter by date to
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Items per page (default: 50, max: 100)
    pub limit: Option<u32>,
}

/// Response for listing adjustment documents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentListResponse {
    /// List of adjustment documents
    pub adjustments: Vec<AdjustmentDocument>,
    /// Total count for pagination
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// Summary statistics for adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AdjustmentSummary {
    /// Total number of adjustments
    pub total_adjustments: u64,
    /// Number of increase adjustments
    pub total_increases: u64,
    /// Number of decrease adjustments
    pub total_decreases: u64,
    /// Net quantity change
    pub net_change: i64,
}

// ============================================================================
// Validation Logic (pure domain, no infrastructure)
// ============================================================================

/// Validation error for adjustment operations
#[derive(Debug, Clone)]
pub struct AdjustmentValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for AdjustmentValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for AdjustmentValidationError {}

/// Validate an adjustment line input
pub fn validate_adjustment_line(
    line: &AdjustmentLineInput,
) -> Result<(), AdjustmentValidationError> {
    if line.qty <= 0 {
        return Err(AdjustmentValidationError {
            field: "qty".to_string(),
            message: "Quantity must be greater than 0".to_string(),
        });
    }
    Ok(())
}

/// Validate status transition
pub fn validate_status_transition(
    current: AdjustmentStatus,
    new: AdjustmentStatus,
) -> Result<(), AdjustmentValidationError> {
    if !current.can_transition_to(new) {
        return Err(AdjustmentValidationError {
            field: "status".to_string(),
            message: format!("Cannot transition from {} to {}", current, new),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_transitions() {
        assert!(AdjustmentStatus::Draft.can_transition_to(AdjustmentStatus::Posted));
        assert!(AdjustmentStatus::Draft.can_transition_to(AdjustmentStatus::Cancelled));
        assert!(!AdjustmentStatus::Posted.can_transition_to(AdjustmentStatus::Draft));
        assert!(!AdjustmentStatus::Posted.can_transition_to(AdjustmentStatus::Cancelled));
        assert!(!AdjustmentStatus::Cancelled.can_transition_to(AdjustmentStatus::Draft));
        assert!(!AdjustmentStatus::Cancelled.can_transition_to(AdjustmentStatus::Posted));
    }

    #[test]
    fn test_validate_adjustment_line_valid() {
        let line = AdjustmentLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            adjustment_type: AdjustmentType::Decrease,
            qty: 10,
            reason_code: AdjustmentReasonCode::Damaged,
            reason_notes: Some("Found broken during inspection".to_string()),
            location_id: None,
            lot_id: None,
            serial_id: None,
        };
        assert!(validate_adjustment_line(&line).is_ok());
    }

    #[test]
    fn test_validate_adjustment_line_zero_qty() {
        let line = AdjustmentLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            adjustment_type: AdjustmentType::Decrease,
            qty: 0,
            reason_code: AdjustmentReasonCode::Damaged,
            reason_notes: None,
            location_id: None,
            lot_id: None,
            serial_id: None,
        };
        assert!(validate_adjustment_line(&line).is_err());
    }

    #[test]
    fn test_adjustment_type_display() {
        assert_eq!(AdjustmentType::Increase.to_string(), "increase");
        assert_eq!(AdjustmentType::Decrease.to_string(), "decrease");
    }

    #[test]
    fn test_reason_code_display() {
        assert_eq!(AdjustmentReasonCode::Damaged.to_string(), "damaged");
        assert_eq!(AdjustmentReasonCode::CountCorrection.to_string(), "count_correction");
    }
}
