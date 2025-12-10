//! PostgreSQL implementations for stock repositories
//!
//! This module contains PostgreSQL implementations of StockMoveRepository and InventoryLevelRepository.

use async_trait::async_trait;
use sqlx::PgPool;
use std::ops::DerefMut;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::models::{CreateStockMoveRequest, InventoryLevel, StockMove};
use inventory_service_core::repositories::{InventoryLevelRepository, StockMoveRepository};
use shared_error::AppError;

/// Helper type for infra-internal transaction operations
pub type InfraTx<'a> = &'a mut sqlx::Transaction<'a, sqlx::Postgres>;

/// PostgreSQL implementation of StockMoveRepository
pub struct PgStockMoveRepository {
    pool: Arc<PgPool>,
}

impl PgStockMoveRepository {
    /// Create a new PostgreSQL stock move repository
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Internal helper: Create a stock move within a transaction
    /// This is used by services for transactional orchestration
    /// Returns the created move_id and the transaction
    pub async fn create_with_tx<'a>(
        &self,
        mut tx: sqlx::Transaction<'a, sqlx::Postgres>,
        stock_move: CreateStockMoveRequest,
        tenant_id: Uuid,
    ) -> Result<(Uuid, sqlx::Transaction<'a, sqlx::Postgres>), AppError> {
        let move_id = sqlx::query!(
            r#"
            INSERT INTO stock_moves (
                tenant_id, product_id, source_location_id, destination_location_id,
                move_type, quantity, unit_cost, reference_type, reference_id,
                lot_serial_id, idempotency_key, move_reason, batch_info, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING move_id
            "#,
            tenant_id,
            stock_move.product_id,
            stock_move.source_location_id,
            stock_move.destination_location_id,
            stock_move.move_type,
            stock_move.quantity,
            stock_move.unit_cost,
            stock_move.reference_type,
            stock_move.reference_id,
            stock_move.lot_serial_id,
            stock_move.idempotency_key,
            stock_move.move_reason,
            stock_move.batch_info,
            stock_move.metadata,
        )
        .fetch_one(tx.deref_mut())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .move_id;

        Ok((move_id, tx))
    }

    /// Internal helper: Create a stock move idempotently within a transaction
    /// Returns true if the row was created, false if it already existed (no-op)
    pub async fn create_idempotent_with_tx<'a>(
        &self,
        mut tx: sqlx::Transaction<'a, sqlx::Postgres>,
        stock_move: &CreateStockMoveRequest,
        tenant_id: Uuid,
    ) -> Result<(bool, sqlx::Transaction<'a, sqlx::Postgres>), AppError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO stock_moves (
                tenant_id, product_id, source_location_id, destination_location_id,
                move_type, quantity, unit_cost, reference_type, reference_id,
                lot_serial_id, idempotency_key, move_reason, batch_info, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
            "#,
            tenant_id,
            stock_move.product_id,
            stock_move.source_location_id,
            stock_move.destination_location_id,
            stock_move.move_type,
            stock_move.quantity,
            stock_move.unit_cost,
            stock_move.reference_type,
            stock_move.reference_id,
            stock_move.lot_serial_id,
            stock_move.idempotency_key,
            stock_move.move_reason,
            stock_move.batch_info,
            stock_move.metadata,
        )
        .execute(tx.deref_mut())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Return true if a row was inserted, false if it was a no-op due to conflict
        Ok((result.rows_affected() > 0, tx))
    }
}

#[async_trait]
impl StockMoveRepository for PgStockMoveRepository {
    async fn create(
        &self,
        stock_move: &CreateStockMoveRequest,
        tenant_id: Uuid,
    ) -> Result<StockMove, AppError> {
        let created_move = sqlx::query_as!(
            StockMove,
            r#"
            INSERT INTO stock_moves (
                tenant_id, product_id, source_location_id, destination_location_id,
                move_type, quantity, unit_cost, reference_type, reference_id,
                lot_serial_id, idempotency_key, move_reason, batch_info, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING
                move_id, tenant_id, product_id, source_location_id, destination_location_id,
                move_type, quantity, unit_cost, total_cost, reference_type, reference_id,
                lot_serial_id, idempotency_key, move_date, move_reason, batch_info, metadata, created_at
            "#,
            tenant_id,
            stock_move.product_id,
            stock_move.source_location_id,
            stock_move.destination_location_id,
            stock_move.move_type,
            stock_move.quantity,
            stock_move.unit_cost,
            stock_move.reference_type,
            stock_move.reference_id,
            stock_move.lot_serial_id,
            stock_move.idempotency_key,
            stock_move.move_reason,
            stock_move.batch_info,
            stock_move.metadata,
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(created_move)
    }

    async fn find_by_reference(
        &self,
        tenant_id: Uuid,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<StockMove>, AppError> {
        let stock_moves = sqlx::query_as!(
            StockMove,
            r#"
            SELECT
                move_id, tenant_id, product_id, source_location_id, destination_location_id,
                move_type, quantity, unit_cost, total_cost, reference_type, reference_id,
                lot_serial_id, idempotency_key, move_date, move_reason, batch_info, metadata, created_at
            FROM stock_moves
            WHERE tenant_id = $1 AND reference_type = $2 AND reference_id = $3
            ORDER BY created_at ASC
            "#,
            tenant_id,
            reference_type,
            reference_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(stock_moves)
    }

    async fn exists_by_idempotency_key(
        &self,
        tenant_id: Uuid,
        idempotency_key: &str,
    ) -> Result<bool, AppError> {
        let exists = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM stock_moves
                WHERE tenant_id = $1 AND idempotency_key = $2
            ) as exists
            "#,
            tenant_id,
            idempotency_key
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .exists
        .unwrap_or(false);

        Ok(exists)
    }

    async fn find_by_lot_serial(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<Vec<StockMove>, AppError> {
        let stock_moves = sqlx::query_as!(
            StockMove,
            r#"
            SELECT
                move_id, tenant_id, product_id, source_location_id, destination_location_id,
                move_type, quantity, unit_cost, total_cost, reference_type, reference_id,
                lot_serial_id, idempotency_key, move_date, move_reason, batch_info, metadata, created_at
            FROM stock_moves
            WHERE tenant_id = $1 AND lot_serial_id = $2
            ORDER BY created_at ASC
            "#,
            tenant_id,
            lot_serial_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(stock_moves)
    }
}

/// PostgreSQL implementation of InventoryLevelRepository
pub struct PgInventoryLevelRepository {
    pool: Arc<PgPool>,
}

impl PgInventoryLevelRepository {
    /// Create a new PostgreSQL inventory level repository
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Internal helper: Update available quantity within a transaction
    /// This is used by services for transactional orchestration
    /// Note: Assumes the transaction already holds the necessary locks
    pub async fn update_available_quantity_with_tx<'a>(
        &self,
        mut tx: sqlx::Transaction<'a, sqlx::Postgres>,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
    ) -> Result<sqlx::Transaction<'a, sqlx::Postgres>, AppError> {
        // Note: In transactional context, the caller should have already acquired
        // the necessary locks (e.g., SELECT ... FOR UPDATE) before calling this method

        let result = sqlx::query!(
            r#"
            UPDATE inventory_levels
            SET available_quantity = available_quantity + $4,
                updated_at = NOW()
            WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id,
            product_id,
            quantity_change
        )
        .execute(tx.deref_mut())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Inventory level not found for product {} in warehouse {}",
                product_id, warehouse_id
            )));
        }

        Ok(tx)
    }
}

#[async_trait]
impl InventoryLevelRepository for PgInventoryLevelRepository {
    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<InventoryLevel>, AppError> {
        let inventory_level = sqlx::query_as!(
            InventoryLevel,
            r#"
            SELECT
                inventory_id, tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity,
                created_at, updated_at, deleted_at
            FROM inventory_levels
            WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3 AND deleted_at IS NULL
            "#,
            tenant_id,
            warehouse_id,
            product_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(inventory_level)
    }

    async fn update_available_quantity(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx = self
            .update_available_quantity_with_tx(
                tx,
                tenant_id,
                warehouse_id,
                product_id,
                quantity_change,
            )
            .await?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn upsert(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        available_quantity: i64,
        reserved_quantity: i64,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query!(
            r#"
            INSERT INTO inventory_levels (tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tenant_id, warehouse_id, product_id) WHERE deleted_at IS NULL
            DO UPDATE SET
                available_quantity = EXCLUDED.available_quantity,
                reserved_quantity = EXCLUDED.reserved_quantity,
                updated_at = NOW()
            "#,
            tenant_id,
            warehouse_id,
            product_id,
            available_quantity,
            reserved_quantity
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
