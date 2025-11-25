//! Stock reconciliation repository traits
//!
//! This module defines the repository traits for reconciliation operations.

use async_trait::async_trait;
use sqlx::Transaction;
use uuid::Uuid;

use crate::domains::inventory::reconciliation::{
    CycleType, ReconciliationStatus, StockReconciliation, StockReconciliationItem,
};
use shared_error::AppError;

/// Represents a count update for a reconciliation item
#[derive(Debug, Clone)]
pub struct ReconciliationItemCountUpdate {
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub location_id: Option<Uuid>,
    pub counted_quantity: i64,
    pub unit_cost: Option<f64>,
    pub counted_by: Uuid,
    pub notes: Option<String>,
}

/// Repository trait for reconciliation operations
#[async_trait]
pub trait StockReconciliationRepository: Send + Sync {
    /// Create a new reconciliation
    async fn create(
        &self,
        tenant_id: Uuid,
        reconciliation: &StockReconciliation,
    ) -> Result<StockReconciliation, AppError>;

    /// Create a new reconciliation within transaction
    async fn create_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        reconciliation: &StockReconciliation,
    ) -> Result<StockReconciliation, AppError>;

    /// Find reconciliation by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<Option<StockReconciliation>, AppError>;

    /// Update reconciliation status
    async fn update_status(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        status: ReconciliationStatus,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Finalize reconciliation
    async fn finalize(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        completed_at: chrono::DateTime<chrono::Utc>,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Finalize reconciliation within transaction
    async fn finalize_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        completed_at: chrono::DateTime<chrono::Utc>,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Approve reconciliation
    async fn approve(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        approved_by: Uuid,
        approved_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError>;

    /// Delete reconciliation
    async fn delete(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError>;

    /// List reconciliations with filtering
    async fn list(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<ReconciliationStatus>,
        cycle_type: Option<CycleType>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<StockReconciliation>, AppError>;

    /// Count reconciliations with filtering
    async fn count(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<ReconciliationStatus>,
        cycle_type: Option<CycleType>,
    ) -> Result<i64, AppError>;
}

/// Repository trait for reconciliation item operations
#[async_trait]
pub trait StockReconciliationItemRepository: Send + Sync {
    /// Create reconciliation items from current inventory levels based on cycle type and filters
    async fn create_from_inventory(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        cycle_type: CycleType,
        warehouse_id: Option<Uuid>,
        location_filter: Option<serde_json::Value>,
        product_filter: Option<serde_json::Value>,
    ) -> Result<Vec<StockReconciliationItem>, AppError>;

    /// Create reconciliation items from current inventory levels within transaction
    async fn create_from_inventory_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        cycle_type: CycleType,
        warehouse_id: Option<Uuid>,
        location_filter: Option<serde_json::Value>,
        product_filter: Option<serde_json::Value>,
    ) -> Result<Vec<StockReconciliationItem>, AppError>;

    /// Find items by reconciliation ID
    async fn find_by_reconciliation_id(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<Vec<StockReconciliationItem>, AppError>;

    /// Find item by composite key
    async fn find_by_key(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Option<StockReconciliationItem>, AppError>;

    /// Update counted quantity for an item
    async fn update_count(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
        counted_quantity: i64,
        unit_cost: Option<f64>,
        counted_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), AppError>;

    /// Batch update counts for multiple items
    async fn batch_update_counts(
        &self,
        tenant_id: Uuid,
        counts: &[ReconciliationItemCountUpdate],
    ) -> Result<(), AppError>;

    /// Get variance analysis for a reconciliation
    async fn get_variance_analysis(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<(Vec<StockReconciliationItem>, i64, i64), AppError>;

    /// Delete item
    async fn delete(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        product_id: Uuid,
        warehouse_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError>;
}
