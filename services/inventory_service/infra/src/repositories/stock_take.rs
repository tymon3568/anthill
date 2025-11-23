use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::stock_take::{
    StockTake, StockTakeLine, StockTakeStatus,
};
use inventory_service_core::repositories::stock_take::{
    StockTakeLineRepository, StockTakeRepository,
};
use shared_error::AppError;

/// PostgreSQL implementation of StockTakeRepository
pub struct PgStockTakeRepository {
    pool: Arc<PgPool>,
}

impl PgStockTakeRepository {
    /// Create a new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockTakeRepository for PgStockTakeRepository {
    async fn create(&self, tenant_id: Uuid, stock_take: &StockTake) -> Result<StockTake, AppError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO stock_takes (
                stock_take_id, tenant_id, stock_take_number, warehouse_id, status,
                started_at, created_by, updated_by, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING stock_take_id, tenant_id, stock_take_number, warehouse_id, status,
                      started_at, completed_at, created_by, updated_by, notes,
                      created_at, updated_at, deleted_at, deleted_by
            "#,
            stock_take.stock_take_id,
            tenant_id,
            stock_take.stock_take_number,
            stock_take.warehouse_id,
            stock_take.status as StockTakeStatus,
            stock_take.started_at,
            stock_take.created_by,
            stock_take.updated_by,
            stock_take.notes
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create stock take: {}", e)))?;

        Ok(StockTake {
            stock_take_id: row.stock_take_id,
            tenant_id: row.tenant_id,
            stock_take_number: row.stock_take_number,
            warehouse_id: row.warehouse_id,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            deleted_by: row.deleted_by,
        })
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
    ) -> Result<Option<StockTake>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT stock_take_id, tenant_id, stock_take_number, warehouse_id, status,
                   started_at, completed_at, created_by, updated_by, notes,
                   created_at, updated_at, deleted_at, deleted_by
            FROM stock_takes
            WHERE tenant_id = $1 AND stock_take_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            stock_take_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find stock take: {}", e)))?;

        Ok(row.map(|r| StockTake {
            stock_take_id: r.stock_take_id,
            tenant_id: r.tenant_id,
            stock_take_number: r.stock_take_number,
            warehouse_id: r.warehouse_id,
            status: r.status,
            started_at: r.started_at,
            completed_at: r.completed_at,
            created_by: r.created_by,
            updated_by: r.updated_by,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
            deleted_by: r.deleted_by,
        }))
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        status: StockTakeStatus,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_takes
            SET status = $1, updated_by = $2, updated_at = NOW()
            WHERE tenant_id = $3 AND stock_take_id = $4 AND deleted_at IS NULL
            "#,
            status as StockTakeStatus,
            updated_by,
            tenant_id,
            stock_take_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update stock take status: {}", e))
        })?;

        Ok(())
    }

    async fn finalize(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        completed_at: chrono::DateTime<chrono::Utc>,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_takes
            SET status = $1, completed_at = $2, updated_by = $3, updated_at = NOW()
            WHERE tenant_id = $4 AND stock_take_id = $5 AND deleted_at IS NULL
            "#,
            StockTakeStatus::Completed as StockTakeStatus,
            completed_at,
            updated_by,
            tenant_id,
            stock_take_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to finalize stock take: {}", e)))?;

        Ok(())
    }

    async fn delete(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_takes
            SET deleted_at = NOW(), deleted_by = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND stock_take_id = $3 AND deleted_at IS NULL
            "#,
            deleted_by,
            tenant_id,
            stock_take_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete stock take: {}", e)))?;

        Ok(())
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<StockTakeStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<StockTake>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT stock_take_id, tenant_id, stock_take_number, warehouse_id, status,
                   started_at, completed_at, created_by, updated_by, notes,
                   created_at, updated_at, deleted_at, deleted_by
            FROM stock_takes
            WHERE tenant_id = $1 AND deleted_at IS NULL
            AND ($2::uuid IS NULL OR warehouse_id = $2)
            AND ($3::stock_take_status IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id,
            warehouse_id,
            status as Option<StockTakeStatus>,
            limit.unwrap_or(50),
            offset.unwrap_or(0)
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list stock takes: {}", e)))?;

        let stock_takes = rows
            .into_iter()
            .map(|r| StockTake {
                stock_take_id: r.stock_take_id,
                tenant_id: r.tenant_id,
                stock_take_number: r.stock_take_number,
                warehouse_id: r.warehouse_id,
                status: r.status,
                started_at: r.started_at,
                completed_at: r.completed_at,
                created_by: r.created_by,
                updated_by: r.updated_by,
                notes: r.notes,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
                deleted_by: r.deleted_by,
            })
            .collect();

        Ok(stock_takes)
    }

    async fn count(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<StockTakeStatus>,
    ) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM stock_takes
            WHERE tenant_id = $1 AND deleted_at IS NULL
            AND ($2::uuid IS NULL OR warehouse_id = $2)
            AND ($3::stock_take_status IS NULL OR status = $3)
            "#,
            tenant_id,
            warehouse_id,
            status as Option<StockTakeStatus>
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count stock takes: {}", e)))?;

        Ok(row.count.unwrap_or(0))
    }
}

/// PostgreSQL implementation of StockTakeLineRepository
pub struct PgStockTakeLineRepository {
    pool: Arc<PgPool>,
}

impl PgStockTakeLineRepository {
    /// Create a new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockTakeLineRepository for PgStockTakeLineRepository {
    async fn create_from_inventory(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<Vec<StockTakeLine>, AppError> {
        let rows = sqlx::query!(
            r#"
            INSERT INTO stock_take_lines (line_id, tenant_id, stock_take_id, product_id, expected_quantity)
            SELECT gen_random_uuid(), $1, $2, il.product_id, il.quantity
            FROM inventory_levels il
            WHERE il.tenant_id = $1 AND il.warehouse_id = $3
            RETURNING line_id, tenant_id, stock_take_id, product_id, expected_quantity,
                      actual_quantity, difference_quantity, counted_by, counted_at, notes,
                      created_at, updated_at, deleted_at, deleted_by
            "#,
            tenant_id,
            stock_take_id,
            warehouse_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create stock take lines from inventory: {}", e)))?;

        let lines = rows
            .into_iter()
            .map(|r| StockTakeLine {
                line_id: r.line_id,
                tenant_id: r.tenant_id,
                stock_take_id: r.stock_take_id,
                product_id: r.product_id,
                expected_quantity: r.expected_quantity,
                actual_quantity: r.actual_quantity,
                difference_quantity: r.difference_quantity,
                counted_by: r.counted_by,
                counted_at: r.counted_at,
                notes: r.notes,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
                deleted_by: r.deleted_by,
            })
            .collect();

        Ok(lines)
    }

    async fn find_by_stock_take_id(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
    ) -> Result<Vec<StockTakeLine>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT line_id, tenant_id, stock_take_id, product_id, expected_quantity,
                   actual_quantity, difference_quantity, counted_by, counted_at, notes,
                   created_at, updated_at, deleted_at, deleted_by
            FROM stock_take_lines
            WHERE tenant_id = $1 AND stock_take_id = $2 AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            tenant_id,
            stock_take_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find stock take lines: {}", e)))?;

        let lines = rows
            .into_iter()
            .map(|r| StockTakeLine {
                line_id: r.line_id,
                tenant_id: r.tenant_id,
                stock_take_id: r.stock_take_id,
                product_id: r.product_id,
                expected_quantity: r.expected_quantity,
                actual_quantity: r.actual_quantity,
                difference_quantity: r.difference_quantity,
                counted_by: r.counted_by,
                counted_at: r.counted_at,
                notes: r.notes,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
                deleted_by: r.deleted_by,
            })
            .collect();

        Ok(lines)
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
    ) -> Result<Option<StockTakeLine>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT line_id, tenant_id, stock_take_id, product_id, expected_quantity,
                   actual_quantity, difference_quantity, counted_by, counted_at, notes,
                   created_at, updated_at, deleted_at, deleted_by
            FROM stock_take_lines
            WHERE tenant_id = $1 AND line_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            line_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to find stock take line: {}", e)))?;

        Ok(row.map(|r| StockTakeLine {
            line_id: r.line_id,
            tenant_id: r.tenant_id,
            stock_take_id: r.stock_take_id,
            product_id: r.product_id,
            expected_quantity: r.expected_quantity,
            actual_quantity: r.actual_quantity,
            difference_quantity: r.difference_quantity,
            counted_by: r.counted_by,
            counted_at: r.counted_at,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
            deleted_by: r.deleted_by,
        }))
    }

    async fn update_count(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        actual_quantity: i64,
        counted_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_take_lines
            SET actual_quantity = $1, counted_by = $2, counted_at = NOW(), notes = $3, updated_at = NOW()
            WHERE tenant_id = $4 AND line_id = $5 AND deleted_at IS NULL
            "#,
            actual_quantity,
            counted_by,
            notes,
            tenant_id,
            line_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update stock take line count: {}", e)))?;

        Ok(())
    }

    async fn batch_update_counts(
        &self,
        tenant_id: Uuid,
        counts: &[(Uuid, i64, Uuid, Option<String>)],
    ) -> Result<(), AppError> {
        if counts.is_empty() {
            return Ok(());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            UPDATE stock_take_lines
            SET actual_quantity = data.actual_quantity,
                counted_by = data.counted_by,
                counted_at = NOW(),
                notes = data.notes,
                updated_at = NOW()
            FROM (VALUES
            "#,
        );

        let mut separated = query_builder.separated(", ");
        for (line_id, actual_quantity, counted_by, notes) in counts {
            separated.push("(");
            separated.push_bind_unseparated(line_id);
            separated.push_bind_unseparated(actual_quantity);
            separated.push_bind_unseparated(counted_by);
            separated.push_bind_unseparated(notes);
            separated.push_unseparated(")");
        }

        query_builder.push(
            r#"
            ) AS data(line_id, actual_quantity, counted_by, notes)
            WHERE stock_take_lines.line_id = data.line_id
            AND stock_take_lines.tenant_id = $1
            AND stock_take_lines.deleted_at IS NULL
            "#,
        );

        let query = query_builder.build();
        query
            .bind(tenant_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!(
                    "Failed to batch update stock take line counts: {}",
                    e
                ))
            })?;

        Ok(())
    }

    async fn delete(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE stock_take_lines
            SET deleted_at = NOW(), deleted_by = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND line_id = $3 AND deleted_at IS NULL
            "#,
            deleted_by,
            tenant_id,
            line_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete stock take line: {}", e)))?;

        Ok(())
    }
}
