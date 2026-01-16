//! Landed Cost domain entities
//!
//! Core business entities for landed cost allocation,
//! supporting freight, customs, handling, and insurance cost allocation
//! to goods receipts for accurate inventory valuation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a landed cost document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LandedCostStatus {
    #[default]
    Draft,
    Posted,
    Cancelled,
}

impl std::fmt::Display for LandedCostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Posted => write!(f, "posted"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<String> for LandedCostStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "draft" => Self::Draft,
            "posted" => Self::Posted,
            "cancelled" => Self::Cancelled,
            _ => Self::Draft,
        }
    }
}

/// Type of cost being allocated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum CostType {
    Freight,
    Customs,
    Handling,
    Insurance,
    Other,
}

impl std::fmt::Display for CostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Freight => write!(f, "freight"),
            Self::Customs => write!(f, "customs"),
            Self::Handling => write!(f, "handling"),
            Self::Insurance => write!(f, "insurance"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl From<String> for CostType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "freight" => Self::Freight,
            "customs" => Self::Customs,
            "handling" => Self::Handling,
            "insurance" => Self::Insurance,
            _ => Self::Other,
        }
    }
}

/// Method for allocating costs to target items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AllocationMethod {
    /// Allocate proportionally by value (default for MVP)
    #[default]
    ByValue,
    /// Allocate proportionally by quantity (future)
    ByQuantity,
    /// Allocate proportionally by weight (future)
    ByWeight,
    /// Allocate proportionally by volume (future)
    ByVolume,
}

impl std::fmt::Display for AllocationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByValue => write!(f, "by_value"),
            Self::ByQuantity => write!(f, "by_quantity"),
            Self::ByWeight => write!(f, "by_weight"),
            Self::ByVolume => write!(f, "by_volume"),
        }
    }
}

impl From<String> for AllocationMethod {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "by_value" => Self::ByValue,
            "by_quantity" => Self::ByQuantity,
            "by_weight" => Self::ByWeight,
            "by_volume" => Self::ByVolume,
            _ => Self::ByValue,
        }
    }
}

/// Type of target for cost allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum TargetType {
    /// Goods receipt item
    GrnItem,
    /// Stock move
    StockMove,
}

impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GrnItem => write!(f, "grn_item"),
            Self::StockMove => write!(f, "stock_move"),
        }
    }
}

impl From<String> for TargetType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "grn_item" => Self::GrnItem,
            "stock_move" => Self::StockMove,
            _ => Self::GrnItem,
        }
    }
}

/// Landed cost document entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCost {
    pub tenant_id: Uuid,
    pub landed_cost_id: Uuid,
    pub reference: Option<String>,
    pub status: LandedCostStatus,
    pub grn_id: Option<Uuid>,
    pub notes: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl LandedCost {
    /// Create a new draft landed cost
    pub fn new(tenant_id: Uuid, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            tenant_id,
            landed_cost_id: Uuid::now_v7(),
            reference: None,
            status: LandedCostStatus::Draft,
            grn_id: None,
            notes: None,
            posted_at: None,
            posted_by: None,
            created_by,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Check if the document can be modified
    pub fn is_modifiable(&self) -> bool {
        self.status == LandedCostStatus::Draft && self.deleted_at.is_none()
    }

    /// Check if the document can be posted
    pub fn can_post(&self) -> bool {
        self.status == LandedCostStatus::Draft && self.deleted_at.is_none()
    }

    /// Mark as posted
    pub fn post(&mut self, posted_by: Uuid) {
        self.status = LandedCostStatus::Posted;
        self.posted_at = Some(Utc::now());
        self.posted_by = Some(posted_by);
        self.updated_at = Utc::now();
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.status = LandedCostStatus::Cancelled;
        self.updated_at = Utc::now();
    }
}

/// Landed cost line entity (individual cost component)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostLine {
    pub tenant_id: Uuid,
    pub landed_cost_line_id: Uuid,
    pub landed_cost_id: Uuid,
    pub cost_type: CostType,
    pub description: Option<String>,
    pub amount_cents: i64,
    pub allocation_method: AllocationMethod,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl LandedCostLine {
    /// Create a new cost line
    pub fn new(
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        cost_type: CostType,
        amount_cents: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            tenant_id,
            landed_cost_line_id: Uuid::now_v7(),
            landed_cost_id,
            cost_type,
            description: None,
            amount_cents,
            allocation_method: AllocationMethod::default(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

/// Landed cost allocation entity (computed allocation to target)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LandedCostAllocation {
    pub tenant_id: Uuid,
    pub landed_cost_allocation_id: Uuid,
    pub landed_cost_id: Uuid,
    pub landed_cost_line_id: Uuid,
    pub target_type: TargetType,
    pub target_id: Uuid,
    pub allocated_amount_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl LandedCostAllocation {
    /// Create a new allocation
    pub fn new(
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        landed_cost_line_id: Uuid,
        target_type: TargetType,
        target_id: Uuid,
        allocated_amount_cents: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            tenant_id,
            landed_cost_allocation_id: Uuid::now_v7(),
            landed_cost_id,
            landed_cost_line_id,
            target_type,
            target_id,
            allocated_amount_cents,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

/// Target item for allocation computation
#[derive(Debug, Clone)]
pub struct AllocationTarget {
    pub target_type: TargetType,
    pub target_id: Uuid,
    pub value_cents: i64,
}
