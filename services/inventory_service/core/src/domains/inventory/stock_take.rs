//! Stock Take domain entities
//!
//! This module defines the domain entities for stock takes and stock take lines.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a stock take session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StockTake {
    /// Primary key
    pub stock_take_id: Uuid,
    /// Tenant isolation
    pub tenant_id: Uuid,
    /// Auto-generated stock take number
    pub stock_take_number: String,
    /// Warehouse being counted
    pub warehouse_id: Uuid,
    /// Stock take status
    pub status: StockTakeStatus,
    /// When counting started
    pub started_at: Option<DateTime<Utc>>,
    /// When counting completed
    pub completed_at: Option<DateTime<Utc>>,
    /// User who created the stock take
    pub created_by: Uuid,
    /// User who last updated
    pub updated_by: Option<Uuid>,
    /// Additional notes
    pub notes: Option<String>,
    /// Audit timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who deleted
    pub deleted_by: Option<Uuid>,
}

/// Stock take status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum StockTakeStatus {
    Draft,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl fmt::Display for StockTakeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display matches DB values (snake_case) for consistency
        let s = match self {
            StockTakeStatus::Draft => "draft",
            StockTakeStatus::Scheduled => "scheduled",
            StockTakeStatus::InProgress => "in_progress",
            StockTakeStatus::Completed => "completed",
            StockTakeStatus::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for StockTakeStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(StockTakeStatus::Draft),
            "scheduled" => Ok(StockTakeStatus::Scheduled),
            "in_progress" => Ok(StockTakeStatus::InProgress),
            "completed" => Ok(StockTakeStatus::Completed),
            "cancelled" => Ok(StockTakeStatus::Cancelled),
            _ => Err(format!("Unknown stock take status: {}", s)),
        }
    }
}

/// Represents a line item in a stock take
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StockTakeLine {
    /// Primary key
    pub line_id: Uuid,
    /// Tenant isolation
    pub tenant_id: Uuid,
    /// Parent stock take
    pub stock_take_id: Uuid,
    /// Product being counted
    pub product_id: Uuid,
    /// Expected quantity from system
    pub expected_quantity: i64,
    /// Actual counted quantity
    pub actual_quantity: Option<i64>,
    /// Difference (actual - expected)
    pub difference_quantity: Option<i64>,
    /// User who performed the count
    pub counted_by: Option<Uuid>,
    /// When the count was performed
    pub counted_at: Option<DateTime<Utc>>,
    /// Additional notes for this line
    pub notes: Option<String>,
    /// Audit timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who deleted
    pub deleted_by: Option<Uuid>,
}
