//! Stock Take repository traits
//!
//! This module defines the repository traits for stock take operations.

use async_trait::async_trait;
use sqlx::Transaction;
use uuid::Uuid;

use crate::domains::inventory::stock_take::{StockTake, StockTakeLine, StockTakeStatus};
use shared_error::AppError;

/// Repository trait for stock take operations
#[async_trait]
pub trait StockTakeRepository: Send + Sync {
    /// Create a new stock take
    async fn create(&self, tenant_id: Uuid, stock_take: &StockTake) -> Result<StockTake, AppError>;

    /// Create a new stock take within transaction
    async fn create_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        stock_take: &StockTake,
    ) -> Result<StockTake, AppError>;

    /// Find stock take by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
    ) -> Result<Option<StockTake>, AppError>;

    /// Update stock take status
    async fn update_status(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        status: StockTakeStatus,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Finalize stock take
    async fn finalize(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        completed_at: chrono::DateTime<chrono::Utc>,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Finalize stock take within transaction
    async fn finalize_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        completed_at: chrono::DateTime<chrono::Utc>,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Delete stock take (soft delete)
    async fn delete(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError>;

    /// List stock takes with filtering
    async fn list(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<StockTakeStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<StockTake>, AppError>;

    /// Count stock takes with filtering
    async fn count(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<StockTakeStatus>,
    ) -> Result<i64, AppError>;
}

/// Repository trait for stock take line operations
#[async_trait]
pub trait StockTakeLineRepository: Send + Sync {
    /// Create stock take lines from current inventory levels
    async fn create_from_inventory(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<StockTakeLine>, AppError>;

    /// Create stock take lines from current inventory levels within transaction
    async fn create_from_inventory_with_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<StockTakeLine>, AppError>;

    /// Find lines by stock take ID
    async fn find_by_stock_take_id(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
    ) -> Result<Vec<StockTakeLine>, AppError>;

    /// Find line by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
    ) -> Result<Option<StockTakeLine>, AppError>;

    /// Update actual quantity for a line
    async fn update_count(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        actual_quantity: i64,
        counted_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), AppError>;

    /// Batch update counts for multiple lines
    async fn batch_update_counts(
        &self,
        tenant_id: Uuid,
        counts: &[(Uuid, i64, Uuid, Option<String>)],
    ) -> Result<(), AppError>;

    /// Delete line (soft delete)
    async fn delete(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError>;
}
