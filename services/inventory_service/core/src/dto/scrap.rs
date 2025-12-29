//! Scrap Management Data Transfer Objects
//!
//! This module contains request and response DTOs for scrap management operations,
//! following the 3-crate pattern with zero infrastructure dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Scrap Status Enum
// ============================================================================

/// Status of a scrap document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum ScrapStatus {
    /// Draft - can be edited, lines can be added
    #[default]
    Draft,
    /// Posted - scrap has been executed, inventory adjusted
    Posted,
    /// Cancelled - draft was cancelled before posting
    Cancelled,
}

impl std::fmt::Display for ScrapStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapStatus::Draft => write!(f, "draft"),
            ScrapStatus::Posted => write!(f, "posted"),
            ScrapStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl ScrapStatus {
    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: ScrapStatus) -> bool {
        match (self, new_status) {
            // Draft can transition to Posted or Cancelled
            (ScrapStatus::Draft, ScrapStatus::Posted) => true,
            (ScrapStatus::Draft, ScrapStatus::Cancelled) => true,
            // Posted and Cancelled are terminal states
            _ => false,
        }
    }
}

// ============================================================================
// Scrap Reason Codes
// ============================================================================

/// Standard reason codes for scrapping inventory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum ScrapReasonCode {
    /// Product is damaged
    Damaged,
    /// Product has expired
    Expired,
    /// Product is lost/missing
    Lost,
    /// Failed quality control
    QualityFail,
    /// Obsolete/discontinued product
    Obsolete,
    /// Other reason (use notes field for details)
    Other,
}

impl std::fmt::Display for ScrapReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapReasonCode::Damaged => write!(f, "damaged"),
            ScrapReasonCode::Expired => write!(f, "expired"),
            ScrapReasonCode::Lost => write!(f, "lost"),
            ScrapReasonCode::QualityFail => write!(f, "quality_fail"),
            ScrapReasonCode::Obsolete => write!(f, "obsolete"),
            ScrapReasonCode::Other => write!(f, "other"),
        }
    }
}

// ============================================================================
// Domain Entities
// ============================================================================

/// Scrap document header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScrapDocument {
    /// Scrap document ID
    pub scrap_id: Uuid,
    /// Tenant ID for multi-tenancy isolation
    pub tenant_id: Uuid,
    /// Optional reference number
    pub reference: Option<String>,
    /// Document status
    pub status: ScrapStatus,
    /// Scrap location ID (where scrapped items go)
    pub scrap_location_id: Uuid,
    /// Optional notes
    pub notes: Option<String>,
    /// User who created the document
    pub created_by: Option<Uuid>,
    /// User who posted the document
    pub posted_by: Option<Uuid>,
    /// When the document was posted
    pub posted_at: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Scrap document line item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScrapLine {
    /// Line ID
    pub scrap_line_id: Uuid,
    /// Tenant ID for multi-tenancy isolation
    pub tenant_id: Uuid,
    /// Parent scrap document ID
    pub scrap_id: Uuid,
    /// Product being scrapped
    pub product_id: Uuid,
    /// Optional variant ID
    pub variant_id: Option<Uuid>,
    /// Source location (where items are taken from)
    pub source_location_id: Uuid,
    /// Optional lot ID if lot-tracked
    pub lot_id: Option<Uuid>,
    /// Optional serial ID if serial-tracked
    pub serial_id: Option<Uuid>,
    /// Quantity to scrap (must be > 0)
    pub qty: i64,
    /// Reason code
    pub reason_code: Option<ScrapReasonCode>,
    /// Free-text reason/notes
    pub reason: Option<String>,
    /// Stock move ID created on posting (if posted)
    pub posted_stock_move_id: Option<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new scrap document (draft)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateScrapRequest {
    /// Optional reference number
    pub reference: Option<String>,
    /// Scrap location ID (required)
    pub scrap_location_id: Uuid,
    /// Optional notes
    pub notes: Option<String>,
}

/// A single line to add to a scrap document
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScrapLineInput {
    /// Product being scrapped
    pub product_id: Uuid,
    /// Optional variant ID
    pub variant_id: Option<Uuid>,
    /// Source location (where items are taken from)
    pub source_location_id: Uuid,
    /// Optional lot ID if lot-tracked
    pub lot_id: Option<Uuid>,
    /// Optional serial ID if serial-tracked
    pub serial_id: Option<Uuid>,
    /// Quantity to scrap (must be > 0)
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub qty: i64,
    /// Reason code
    pub reason_code: Option<ScrapReasonCode>,
    /// Free-text reason/notes
    pub reason: Option<String>,
}

/// Request to add or replace lines on a scrap document
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AddScrapLinesRequest {
    /// Lines to add
    #[validate(length(min = 1, message = "At least one line is required"))]
    #[validate(nested)]
    pub lines: Vec<ScrapLineInput>,
}

/// Request to post a scrap document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PostScrapRequest {
    /// Optional idempotency key for retry safety
    pub idempotency_key: Option<String>,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response for scrap document operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScrapDocumentResponse {
    /// The scrap document
    pub scrap: ScrapDocument,
}

/// Response for scrap document with lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScrapDocumentWithLinesResponse {
    /// The scrap document
    pub scrap: ScrapDocument,
    /// Associated lines
    pub lines: Vec<ScrapLine>,
}

/// Query parameters for listing scrap documents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(IntoParams))]
pub struct ScrapListQuery {
    /// Filter by warehouse (via scrap_location)
    pub warehouse_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<ScrapStatus>,
    /// Filter by date from
    pub from_date: Option<DateTime<Utc>>,
    /// Filter by date to
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Items per page (default: 50, max: 100)
    pub limit: Option<u32>,
}

/// Response for listing scrap documents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScrapListResponse {
    /// List of scrap documents
    pub scraps: Vec<ScrapDocument>,
    /// Total count for pagination
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

// ============================================================================
// Validation Logic (pure domain, no infrastructure)
// ============================================================================

/// Validation error for scrap operations
#[derive(Debug, Clone)]
pub struct ScrapValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ScrapValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validate a scrap line input
pub fn validate_scrap_line(line: &ScrapLineInput) -> Result<(), ScrapValidationError> {
    if line.qty <= 0 {
        return Err(ScrapValidationError {
            field: "qty".to_string(),
            message: "Quantity must be greater than 0".to_string(),
        });
    }
    Ok(())
}

/// Validate status transition
pub fn validate_status_transition(
    current: ScrapStatus,
    new: ScrapStatus,
) -> Result<(), ScrapValidationError> {
    if !current.can_transition_to(new) {
        return Err(ScrapValidationError {
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
        assert!(ScrapStatus::Draft.can_transition_to(ScrapStatus::Posted));
        assert!(ScrapStatus::Draft.can_transition_to(ScrapStatus::Cancelled));
        assert!(!ScrapStatus::Posted.can_transition_to(ScrapStatus::Draft));
        assert!(!ScrapStatus::Posted.can_transition_to(ScrapStatus::Cancelled));
        assert!(!ScrapStatus::Cancelled.can_transition_to(ScrapStatus::Draft));
        assert!(!ScrapStatus::Cancelled.can_transition_to(ScrapStatus::Posted));
    }

    #[test]
    fn test_validate_scrap_line_valid() {
        let line = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: 10,
            reason_code: Some(ScrapReasonCode::Damaged),
            reason: Some("Found broken during inspection".to_string()),
        };
        assert!(validate_scrap_line(&line).is_ok());
    }

    #[test]
    fn test_validate_scrap_line_zero_qty() {
        let line = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: 0,
            reason_code: None,
            reason: None,
        };
        let result = validate_scrap_line(&line);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().field, "qty");
    }

    #[test]
    fn test_validate_scrap_line_negative_qty() {
        let line = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: -5,
            reason_code: None,
            reason: None,
        };
        let result = validate_scrap_line(&line);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_status_transition_valid() {
        assert!(validate_status_transition(ScrapStatus::Draft, ScrapStatus::Posted).is_ok());
        assert!(validate_status_transition(ScrapStatus::Draft, ScrapStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_validate_status_transition_invalid() {
        assert!(validate_status_transition(ScrapStatus::Posted, ScrapStatus::Draft).is_err());
        assert!(validate_status_transition(ScrapStatus::Cancelled, ScrapStatus::Posted).is_err());
    }

    #[test]
    fn test_reason_code_display() {
        assert_eq!(ScrapReasonCode::Damaged.to_string(), "damaged");
        assert_eq!(ScrapReasonCode::Expired.to_string(), "expired");
        assert_eq!(ScrapReasonCode::QualityFail.to_string(), "quality_fail");
    }

    #[test]
    fn test_scrap_status_display() {
        assert_eq!(ScrapStatus::Draft.to_string(), "draft");
        assert_eq!(ScrapStatus::Posted.to_string(), "posted");
        assert_eq!(ScrapStatus::Cancelled.to_string(), "cancelled");
    }
}
