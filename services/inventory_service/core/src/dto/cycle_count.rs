//! Cycle Counting Data Transfer Objects
//!
//! This module contains request and response DTOs for cycle counting operations,
//! following the 3-crate pattern with zero infrastructure dependencies.
//!
//! Cycle counting extends the existing stock take functionality with:
//! - Schedule-based count sessions
//! - Scope filtering (warehouse/location/product/category)
//! - As-of snapshot semantics
//! - Reconciliation with stock adjustments

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Cycle Count Status Enum
// ============================================================================

/// Status of a cycle count session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CycleCountStatus {
    /// Draft - session created but counting not started
    #[default]
    Draft,
    /// In Progress - counting is active
    InProgress,
    /// Ready to Reconcile - all counts submitted, pending reconciliation
    ReadyToReconcile,
    /// Reconciled - adjustments generated and applied
    Reconciled,
    /// Cancelled - session was cancelled
    Cancelled,
}

impl std::fmt::Display for CycleCountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CycleCountStatus::Draft => write!(f, "draft"),
            CycleCountStatus::InProgress => write!(f, "in_progress"),
            CycleCountStatus::ReadyToReconcile => write!(f, "ready_to_reconcile"),
            CycleCountStatus::Reconciled => write!(f, "reconciled"),
            CycleCountStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl CycleCountStatus {
    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: CycleCountStatus) -> bool {
        match (self, new_status) {
            // Draft can transition to InProgress or Cancelled
            (CycleCountStatus::Draft, CycleCountStatus::InProgress) => true,
            (CycleCountStatus::Draft, CycleCountStatus::Cancelled) => true,
            // InProgress can transition to ReadyToReconcile or Cancelled
            (CycleCountStatus::InProgress, CycleCountStatus::ReadyToReconcile) => true,
            (CycleCountStatus::InProgress, CycleCountStatus::Cancelled) => true,
            // ReadyToReconcile can transition to Reconciled or back to InProgress
            (CycleCountStatus::ReadyToReconcile, CycleCountStatus::Reconciled) => true,
            (CycleCountStatus::ReadyToReconcile, CycleCountStatus::InProgress) => true,
            // Reconciled and Cancelled are terminal states
            _ => false,
        }
    }

    /// Check if this status allows submitting counts
    pub fn allows_counting(&self) -> bool {
        matches!(self, CycleCountStatus::Draft | CycleCountStatus::InProgress)
    }

    /// Check if this status allows editing lines
    pub fn allows_line_edits(&self) -> bool {
        matches!(self, CycleCountStatus::Draft)
    }
}

// ============================================================================
// Line Status Enum
// ============================================================================

/// Status of a cycle count line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CycleCountLineStatus {
    /// Open - not yet counted
    #[default]
    Open,
    /// Counted - count has been submitted
    Counted,
    /// Skipped - intentionally skipped
    Skipped,
}

impl std::fmt::Display for CycleCountLineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CycleCountLineStatus::Open => write!(f, "open"),
            CycleCountLineStatus::Counted => write!(f, "counted"),
            CycleCountLineStatus::Skipped => write!(f, "skipped"),
        }
    }
}

// ============================================================================
// Count Type Enum
// ============================================================================

/// Type of stock count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CountType {
    /// Full physical inventory count
    #[default]
    Full,
    /// Cycle count (partial, scheduled)
    Cycle,
    /// Spot check (ad-hoc verification)
    Spot,
}

impl std::fmt::Display for CountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CountType::Full => write!(f, "full"),
            CountType::Cycle => write!(f, "cycle"),
            CountType::Spot => write!(f, "spot"),
        }
    }
}

// ============================================================================
// Domain Entities
// ============================================================================

/// Cycle count session (extends stock take concept)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CycleCountSession {
    /// Unique session ID
    pub cycle_count_id: Uuid,
    /// Tenant ID for multi-tenancy isolation
    pub tenant_id: Uuid,
    /// Auto-generated session number
    pub session_number: String,
    /// Optional schedule ID that triggered this count
    pub schedule_id: Option<Uuid>,
    /// Warehouse being counted
    pub warehouse_id: Uuid,
    /// Location scope (if specified, only this location subtree)
    pub location_id: Option<Uuid>,
    /// Snapshot reference timestamp
    pub as_of: DateTime<Utc>,
    /// Session status
    pub status: CycleCountStatus,
    /// Count type
    pub count_type: CountType,
    /// Optional notes
    pub notes: Option<String>,
    /// User who created the session
    pub created_by: Option<Uuid>,
    /// User who closed/reconciled the session
    pub closed_by: Option<Uuid>,
    /// When the session was closed
    pub closed_at: Option<DateTime<Utc>>,
    /// Generated adjustment ID (after reconciliation)
    pub adjustment_id: Option<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Cycle count line item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CycleCountLine {
    /// Line ID
    pub line_id: Uuid,
    /// Tenant ID for multi-tenancy isolation
    pub tenant_id: Uuid,
    /// Parent cycle count session ID
    pub cycle_count_id: Uuid,
    /// Product being counted
    pub product_id: Uuid,
    /// Optional variant ID
    pub variant_id: Option<Uuid>,
    /// Location being counted
    pub location_id: Uuid,
    /// Optional lot ID if lot-tracked
    pub lot_id: Option<Uuid>,
    /// Optional serial ID if serial-tracked
    pub serial_id: Option<Uuid>,
    /// Expected quantity (computed as-of snapshot)
    pub expected_qty: i64,
    /// Actual counted quantity (entered by counter)
    pub counted_qty: Option<i64>,
    /// Difference (counted - expected)
    pub difference_qty: Option<i64>,
    /// Line status
    pub line_status: CycleCountLineStatus,
    /// User who performed the count
    pub counted_by: Option<Uuid>,
    /// When the count was submitted
    pub counted_at: Option<DateTime<Utc>>,
    /// Optional notes
    pub notes: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new cycle count session
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateCycleCountRequest {
    /// Optional schedule ID to associate with
    pub schedule_id: Option<Uuid>,
    /// Warehouse to count (required if schedule_id not provided)
    pub warehouse_id: Option<Uuid>,
    /// Location root for scope (optional, defaults to entire warehouse)
    pub location_id: Option<Uuid>,
    /// Product filter (optional)
    pub product_id: Option<Uuid>,
    /// Category filter (optional)
    pub category_id: Option<Uuid>,
    /// Include lots in line generation (default: false)
    #[serde(default)]
    pub include_lots: bool,
    /// As-of timestamp (default: now)
    pub as_of: Option<DateTime<Utc>>,
    /// Count type (default: cycle)
    #[serde(default)]
    pub count_type: CountType,
    /// Optional notes
    pub notes: Option<String>,
}

/// Request to generate or refresh count lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GenerateLinesRequest {
    /// Product filter (optional, to add lines for specific product)
    pub product_id: Option<Uuid>,
    /// Category filter (optional)
    pub category_id: Option<Uuid>,
    /// Include lots (default: false)
    #[serde(default)]
    pub include_lots: bool,
    /// Replace existing lines (only allowed in Draft status)
    #[serde(default)]
    pub replace_existing: bool,
}

/// A single count submission
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CountSubmission {
    /// Line ID being counted
    pub line_id: Uuid,
    /// Counted quantity
    #[validate(range(min = 0, message = "Counted quantity cannot be negative"))]
    pub counted_qty: i64,
    /// Optional notes
    pub notes: Option<String>,
}

/// Request to submit counts
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SubmitCountsRequest {
    /// List of count submissions
    #[validate(length(min = 1, message = "At least one count must be submitted"))]
    #[validate(nested)]
    pub counts: Vec<CountSubmission>,
}

/// Request to skip lines
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SkipLinesRequest {
    /// Line IDs to skip
    #[validate(length(min = 1, message = "At least one line ID required"))]
    pub line_ids: Vec<Uuid>,
    /// Reason for skipping
    pub reason: Option<String>,
}

/// Request to reconcile cycle count
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReconcileRequest {
    /// Idempotency key for retry safety
    pub idempotency_key: Option<String>,
    /// Force reconcile even if there are movements after as_of
    /// (MVP: default false, returns error if movements detected)
    #[serde(default)]
    pub force: bool,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response for cycle count session operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CycleCountResponse {
    /// The cycle count session
    pub cycle_count: CycleCountSession,
}

/// Response for cycle count with lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CycleCountWithLinesResponse {
    /// The cycle count session
    pub cycle_count: CycleCountSession,
    /// Associated lines
    pub lines: Vec<CycleCountLine>,
    /// Summary statistics
    pub summary: CycleCountSummary,
}

/// Summary statistics for a cycle count
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CycleCountSummary {
    /// Total number of lines
    pub total_lines: u32,
    /// Number of lines counted
    pub counted_lines: u32,
    /// Number of lines skipped
    pub skipped_lines: u32,
    /// Number of lines with differences
    pub lines_with_variance: u32,
    /// Total positive variance quantity
    pub total_positive_variance: i64,
    /// Total negative variance quantity
    pub total_negative_variance: i64,
}

/// Query parameters for listing cycle counts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
pub struct CycleCountListQuery {
    /// Filter by warehouse
    pub warehouse_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<CycleCountStatus>,
    /// Filter by count type
    pub count_type: Option<CountType>,
    /// Filter by date from
    pub from_date: Option<DateTime<Utc>>,
    /// Filter by date to
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Items per page (default: 50, max: 100)
    pub limit: Option<u32>,
}

/// Response for listing cycle counts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CycleCountListResponse {
    /// List of cycle count sessions
    pub cycle_counts: Vec<CycleCountSession>,
    /// Total count for pagination
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// Reconciliation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReconcileResponse {
    /// The reconciled cycle count
    pub cycle_count: CycleCountSession,
    /// Generated adjustment ID
    pub adjustment_id: Uuid,
    /// Number of stock moves created
    pub moves_created: u32,
    /// Lines adjusted
    pub lines_adjusted: Vec<LineAdjustment>,
}

/// Individual line adjustment details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct LineAdjustment {
    /// Line ID
    pub line_id: Uuid,
    /// Product ID
    pub product_id: Uuid,
    /// Location ID
    pub location_id: Uuid,
    /// Expected quantity
    pub expected_qty: i64,
    /// Counted quantity
    pub counted_qty: i64,
    /// Adjustment quantity (counted - expected)
    pub adjustment_qty: i64,
    /// Generated stock move ID
    pub stock_move_id: Uuid,
}

// ============================================================================
// Validation Logic (pure domain, no infrastructure)
// ============================================================================

/// Validation error for cycle count operations
#[derive(Debug, Clone)]
pub struct CycleCountValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for CycleCountValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validate status transition
pub fn validate_status_transition(
    current: CycleCountStatus,
    new: CycleCountStatus,
) -> Result<(), CycleCountValidationError> {
    if !current.can_transition_to(new) {
        return Err(CycleCountValidationError {
            field: "status".to_string(),
            message: format!("Cannot transition from {} to {}", current, new),
        });
    }
    Ok(())
}

/// Validate count submission
pub fn validate_count_submission(
    submission: &CountSubmission,
) -> Result<(), CycleCountValidationError> {
    if submission.counted_qty < 0 {
        return Err(CycleCountValidationError {
            field: "counted_qty".to_string(),
            message: "Counted quantity cannot be negative".to_string(),
        });
    }
    Ok(())
}

/// Calculate difference between counted and expected quantities
pub fn calculate_difference(expected_qty: i64, counted_qty: i64) -> i64 {
    counted_qty - expected_qty
}

/// Calculate summary statistics from lines
pub fn calculate_summary(lines: &[CycleCountLine]) -> CycleCountSummary {
    let total_lines = lines.len() as u32;
    let counted_lines = lines
        .iter()
        .filter(|l| l.line_status == CycleCountLineStatus::Counted)
        .count() as u32;
    let skipped_lines = lines
        .iter()
        .filter(|l| l.line_status == CycleCountLineStatus::Skipped)
        .count() as u32;

    let mut lines_with_variance = 0u32;
    let mut total_positive_variance = 0i64;
    let mut total_negative_variance = 0i64;

    for line in lines {
        if let Some(diff) = line.difference_qty {
            if diff != 0 {
                lines_with_variance += 1;
                if diff > 0 {
                    total_positive_variance += diff;
                } else {
                    total_negative_variance += diff;
                }
            }
        }
    }

    CycleCountSummary {
        total_lines,
        counted_lines,
        skipped_lines,
        lines_with_variance,
        total_positive_variance,
        total_negative_variance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_transitions_from_draft() {
        assert!(CycleCountStatus::Draft.can_transition_to(CycleCountStatus::InProgress));
        assert!(CycleCountStatus::Draft.can_transition_to(CycleCountStatus::Cancelled));
        assert!(!CycleCountStatus::Draft.can_transition_to(CycleCountStatus::ReadyToReconcile));
        assert!(!CycleCountStatus::Draft.can_transition_to(CycleCountStatus::Reconciled));
    }

    #[test]
    fn test_status_transitions_from_in_progress() {
        assert!(CycleCountStatus::InProgress.can_transition_to(CycleCountStatus::ReadyToReconcile));
        assert!(CycleCountStatus::InProgress.can_transition_to(CycleCountStatus::Cancelled));
        assert!(!CycleCountStatus::InProgress.can_transition_to(CycleCountStatus::Draft));
        assert!(!CycleCountStatus::InProgress.can_transition_to(CycleCountStatus::Reconciled));
    }

    #[test]
    fn test_status_transitions_from_ready_to_reconcile() {
        assert!(CycleCountStatus::ReadyToReconcile.can_transition_to(CycleCountStatus::Reconciled));
        assert!(CycleCountStatus::ReadyToReconcile.can_transition_to(CycleCountStatus::InProgress));
        assert!(!CycleCountStatus::ReadyToReconcile.can_transition_to(CycleCountStatus::Draft));
        assert!(!CycleCountStatus::ReadyToReconcile.can_transition_to(CycleCountStatus::Cancelled));
    }

    #[test]
    fn test_terminal_states() {
        // Reconciled is terminal
        assert!(!CycleCountStatus::Reconciled.can_transition_to(CycleCountStatus::Draft));
        assert!(!CycleCountStatus::Reconciled.can_transition_to(CycleCountStatus::InProgress));
        assert!(!CycleCountStatus::Reconciled.can_transition_to(CycleCountStatus::ReadyToReconcile));
        assert!(!CycleCountStatus::Reconciled.can_transition_to(CycleCountStatus::Cancelled));

        // Cancelled is terminal
        assert!(!CycleCountStatus::Cancelled.can_transition_to(CycleCountStatus::Draft));
        assert!(!CycleCountStatus::Cancelled.can_transition_to(CycleCountStatus::InProgress));
        assert!(!CycleCountStatus::Cancelled.can_transition_to(CycleCountStatus::ReadyToReconcile));
        assert!(!CycleCountStatus::Cancelled.can_transition_to(CycleCountStatus::Reconciled));
    }

    #[test]
    fn test_allows_counting() {
        assert!(CycleCountStatus::Draft.allows_counting());
        assert!(CycleCountStatus::InProgress.allows_counting());
        assert!(!CycleCountStatus::ReadyToReconcile.allows_counting());
        assert!(!CycleCountStatus::Reconciled.allows_counting());
        assert!(!CycleCountStatus::Cancelled.allows_counting());
    }

    #[test]
    fn test_allows_line_edits() {
        assert!(CycleCountStatus::Draft.allows_line_edits());
        assert!(!CycleCountStatus::InProgress.allows_line_edits());
        assert!(!CycleCountStatus::ReadyToReconcile.allows_line_edits());
        assert!(!CycleCountStatus::Reconciled.allows_line_edits());
        assert!(!CycleCountStatus::Cancelled.allows_line_edits());
    }

    #[test]
    fn test_validate_count_submission_valid() {
        let submission = CountSubmission {
            line_id: Uuid::new_v4(),
            counted_qty: 100,
            notes: None,
        };
        assert!(validate_count_submission(&submission).is_ok());
    }

    #[test]
    fn test_validate_count_submission_zero() {
        let submission = CountSubmission {
            line_id: Uuid::new_v4(),
            counted_qty: 0,
            notes: None,
        };
        // Zero is valid (could mean no stock)
        assert!(validate_count_submission(&submission).is_ok());
    }

    #[test]
    fn test_validate_count_submission_negative() {
        let submission = CountSubmission {
            line_id: Uuid::new_v4(),
            counted_qty: -10,
            notes: None,
        };
        let result = validate_count_submission(&submission);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().field, "counted_qty");
    }

    #[test]
    fn test_calculate_difference() {
        assert_eq!(calculate_difference(100, 110), 10);
        assert_eq!(calculate_difference(100, 90), -10);
        assert_eq!(calculate_difference(100, 100), 0);
        assert_eq!(calculate_difference(0, 50), 50);
        assert_eq!(calculate_difference(50, 0), -50);
    }

    #[test]
    fn test_calculate_summary() {
        let lines = vec![
            CycleCountLine {
                line_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                cycle_count_id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                variant_id: None,
                location_id: Uuid::new_v4(),
                lot_id: None,
                serial_id: None,
                expected_qty: 100,
                counted_qty: Some(110),
                difference_qty: Some(10),
                line_status: CycleCountLineStatus::Counted,
                counted_by: None,
                counted_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            CycleCountLine {
                line_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                cycle_count_id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                variant_id: None,
                location_id: Uuid::new_v4(),
                lot_id: None,
                serial_id: None,
                expected_qty: 50,
                counted_qty: Some(40),
                difference_qty: Some(-10),
                line_status: CycleCountLineStatus::Counted,
                counted_by: None,
                counted_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            CycleCountLine {
                line_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                cycle_count_id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                variant_id: None,
                location_id: Uuid::new_v4(),
                lot_id: None,
                serial_id: None,
                expected_qty: 200,
                counted_qty: None,
                difference_qty: None,
                line_status: CycleCountLineStatus::Open,
                counted_by: None,
                counted_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            CycleCountLine {
                line_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                cycle_count_id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                variant_id: None,
                location_id: Uuid::new_v4(),
                lot_id: None,
                serial_id: None,
                expected_qty: 75,
                counted_qty: None,
                difference_qty: None,
                line_status: CycleCountLineStatus::Skipped,
                counted_by: None,
                counted_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let summary = calculate_summary(&lines);

        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.counted_lines, 2);
        assert_eq!(summary.skipped_lines, 1);
        assert_eq!(summary.lines_with_variance, 2);
        assert_eq!(summary.total_positive_variance, 10);
        assert_eq!(summary.total_negative_variance, -10);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(CycleCountStatus::Draft.to_string(), "draft");
        assert_eq!(CycleCountStatus::InProgress.to_string(), "in_progress");
        assert_eq!(CycleCountStatus::ReadyToReconcile.to_string(), "ready_to_reconcile");
        assert_eq!(CycleCountStatus::Reconciled.to_string(), "reconciled");
        assert_eq!(CycleCountStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_line_status_display() {
        assert_eq!(CycleCountLineStatus::Open.to_string(), "open");
        assert_eq!(CycleCountLineStatus::Counted.to_string(), "counted");
        assert_eq!(CycleCountLineStatus::Skipped.to_string(), "skipped");
    }

    #[test]
    fn test_count_type_display() {
        assert_eq!(CountType::Full.to_string(), "full");
        assert_eq!(CountType::Cycle.to_string(), "cycle");
        assert_eq!(CountType::Spot.to_string(), "spot");
    }
}
