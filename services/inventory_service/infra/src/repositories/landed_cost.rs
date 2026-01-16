//! Landed Cost Repository Implementation
//!
//! PostgreSQL implementation for landed cost data access.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::landed_cost::{
    AllocationMethod, AllocationTarget, CostType, LandedCost, LandedCostAllocation, LandedCostLine,
    LandedCostStatus, TargetType,
};
use shared_error::AppError;

/// Repository trait for landed cost data access
#[async_trait]
pub trait LandedCostRepository: Send + Sync {
    /// Insert a new landed cost document
    async fn insert(&self, landed_cost: &LandedCost) -> Result<(), AppError>;

    /// Find landed cost by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<Option<LandedCost>, AppError>;

    /// Update landed cost status
    async fn update_status(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        status: LandedCostStatus,
        posted_by: Option<Uuid>,
    ) -> Result<(), AppError>;

    /// List landed costs with filtering
    async fn list(
        &self,
        tenant_id: Uuid,
        status: Option<LandedCostStatus>,
        grn_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LandedCost>, i64), AppError>;

    /// Count landed costs matching criteria
    async fn count(
        &self,
        tenant_id: Uuid,
        status: Option<LandedCostStatus>,
        grn_id: Option<Uuid>,
    ) -> Result<i64, AppError>;
}

/// Repository trait for landed cost line data access
#[async_trait]
pub trait LandedCostLineRepository: Send + Sync {
    /// Insert a new cost line
    async fn insert(&self, line: &LandedCostLine) -> Result<(), AppError>;

    /// Find lines by landed cost ID
    async fn find_by_landed_cost_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<Vec<LandedCostLine>, AppError>;

    /// Get total amount for a landed cost
    async fn get_total_amount(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<i64, AppError>;
}

/// Repository trait for landed cost allocation data access
#[async_trait]
pub trait LandedCostAllocationRepository: Send + Sync {
    /// Insert allocations (batch)
    async fn insert_batch(&self, allocations: &[LandedCostAllocation]) -> Result<(), AppError>;

    /// Delete allocations for a landed cost (for recompute)
    async fn delete_by_landed_cost_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<i64, AppError>;

    /// Find allocations by landed cost ID
    async fn find_by_landed_cost_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<Vec<LandedCostAllocation>, AppError>;

    /// Check if allocations exist for a landed cost
    async fn has_allocations(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<bool, AppError>;
}

/// Repository trait for fetching allocation targets (GRN items)
#[async_trait]
pub trait AllocationTargetRepository: Send + Sync {
    /// Get allocation targets for a GRN
    async fn get_grn_item_targets(
        &self,
        tenant_id: Uuid,
        grn_id: Uuid,
    ) -> Result<Vec<AllocationTarget>, AppError>;
}

/// PostgreSQL implementation of landed cost repositories
pub struct PgLandedCostRepository {
    pool: Arc<PgPool>,
}

impl PgLandedCostRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn parse_status(s: &str) -> LandedCostStatus {
        match s {
            "draft" => LandedCostStatus::Draft,
            "posted" => LandedCostStatus::Posted,
            "cancelled" => LandedCostStatus::Cancelled,
            _ => LandedCostStatus::Draft,
        }
    }

    fn parse_cost_type(s: &str) -> CostType {
        match s {
            "freight" => CostType::Freight,
            "customs" => CostType::Customs,
            "handling" => CostType::Handling,
            "insurance" => CostType::Insurance,
            _ => CostType::Other,
        }
    }

    fn parse_allocation_method(s: &str) -> AllocationMethod {
        match s {
            "by_value" => AllocationMethod::ByValue,
            "by_quantity" => AllocationMethod::ByQuantity,
            "by_weight" => AllocationMethod::ByWeight,
            "by_volume" => AllocationMethod::ByVolume,
            _ => AllocationMethod::ByValue,
        }
    }

    fn parse_target_type(s: &str) -> TargetType {
        match s {
            "grn_item" => TargetType::GrnItem,
            "stock_move" => TargetType::StockMove,
            _ => TargetType::GrnItem,
        }
    }
}

#[async_trait]
impl LandedCostRepository for PgLandedCostRepository {
    async fn insert(&self, lc: &LandedCost) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO landed_costs (
                tenant_id, landed_cost_id, reference, status, grn_id,
                notes, posted_at, posted_by, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            lc.tenant_id,
            lc.landed_cost_id,
            lc.reference,
            lc.status.to_string(),
            lc.grn_id,
            lc.notes,
            lc.posted_at,
            lc.posted_by,
            lc.created_by,
            lc.created_at,
            lc.updated_at,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<Option<LandedCost>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT
                tenant_id, landed_cost_id, reference, status, grn_id,
                notes, posted_at, posted_by, created_by, created_at, updated_at, deleted_at
            FROM landed_costs
            WHERE tenant_id = $1 AND landed_cost_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            landed_cost_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(|r| LandedCost {
            tenant_id: r.tenant_id,
            landed_cost_id: r.landed_cost_id,
            reference: r.reference,
            status: Self::parse_status(&r.status),
            grn_id: r.grn_id,
            notes: r.notes,
            posted_at: r.posted_at,
            posted_by: r.posted_by,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
        }))
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        status: LandedCostStatus,
        posted_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let posted_at = if status == LandedCostStatus::Posted {
            Some(now)
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE landed_costs
            SET status = $3, posted_at = COALESCE($4, posted_at), posted_by = COALESCE($5, posted_by), updated_at = $6
            WHERE tenant_id = $1 AND landed_cost_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            landed_cost_id,
            status.to_string(),
            posted_at,
            posted_by,
            now,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        status: Option<LandedCostStatus>,
        grn_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LandedCost>, i64), AppError> {
        let status_str = status.map(|s| s.to_string());

        let rows = sqlx::query!(
            r#"
            SELECT
                tenant_id, landed_cost_id, reference, status, grn_id,
                notes, posted_at, posted_by, created_by, created_at, updated_at, deleted_at
            FROM landed_costs
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR grn_id = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id,
            status_str,
            grn_id,
            limit,
            offset,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        let items: Vec<LandedCost> = rows
            .into_iter()
            .map(|r| LandedCost {
                tenant_id: r.tenant_id,
                landed_cost_id: r.landed_cost_id,
                reference: r.reference,
                status: Self::parse_status(&r.status),
                grn_id: r.grn_id,
                notes: r.notes,
                posted_at: r.posted_at,
                posted_by: r.posted_by,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            })
            .collect();

        let total = self.count(tenant_id, status, grn_id).await?;

        Ok((items, total))
    }

    async fn count(
        &self,
        tenant_id: Uuid,
        status: Option<LandedCostStatus>,
        grn_id: Option<Uuid>,
    ) -> Result<i64, AppError> {
        let status_str = status.map(|s| s.to_string());

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM landed_costs
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::UUID IS NULL OR grn_id = $3)
            "#,
            tenant_id,
            status_str,
            grn_id,
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row.count)
    }
}

#[async_trait]
impl LandedCostLineRepository for PgLandedCostRepository {
    async fn insert(&self, line: &LandedCostLine) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO landed_cost_lines (
                tenant_id, landed_cost_line_id, landed_cost_id, cost_type,
                description, amount_cents, allocation_method, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            line.tenant_id,
            line.landed_cost_line_id,
            line.landed_cost_id,
            line.cost_type.to_string(),
            line.description,
            line.amount_cents,
            line.allocation_method.to_string(),
            line.created_at,
            line.updated_at,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn find_by_landed_cost_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<Vec<LandedCostLine>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                tenant_id, landed_cost_line_id, landed_cost_id, cost_type,
                description, amount_cents, allocation_method, created_at, updated_at, deleted_at
            FROM landed_cost_lines
            WHERE tenant_id = $1 AND landed_cost_id = $2 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
            tenant_id,
            landed_cost_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| LandedCostLine {
                tenant_id: r.tenant_id,
                landed_cost_line_id: r.landed_cost_line_id,
                landed_cost_id: r.landed_cost_id,
                cost_type: Self::parse_cost_type(&r.cost_type),
                description: r.description,
                amount_cents: r.amount_cents,
                allocation_method: Self::parse_allocation_method(&r.allocation_method),
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            })
            .collect())
    }

    async fn get_total_amount(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_cents), 0) as "total!"
            FROM landed_cost_lines
            WHERE tenant_id = $1 AND landed_cost_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            landed_cost_id,
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        use num_traits::ToPrimitive;
        Ok(row.total.to_i64().unwrap_or(0))
    }
}

#[async_trait]
impl LandedCostAllocationRepository for PgLandedCostRepository {
    async fn insert_batch(&self, allocations: &[LandedCostAllocation]) -> Result<(), AppError> {
        if allocations.is_empty() {
            return Ok(());
        }

        for alloc in allocations {
            sqlx::query!(
                r#"
                INSERT INTO landed_cost_allocations (
                    tenant_id, landed_cost_allocation_id, landed_cost_id, landed_cost_line_id,
                    target_type, target_id, allocated_amount_cents, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                alloc.tenant_id,
                alloc.landed_cost_allocation_id,
                alloc.landed_cost_id,
                alloc.landed_cost_line_id,
                alloc.target_type.to_string(),
                alloc.target_id,
                alloc.allocated_amount_cents,
                alloc.created_at,
                alloc.updated_at,
            )
            .execute(self.pool.as_ref())
            .await?;
        }

        Ok(())
    }

    async fn delete_by_landed_cost_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM landed_cost_allocations
            WHERE tenant_id = $1 AND landed_cost_id = $2
            "#,
            tenant_id,
            landed_cost_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn find_by_landed_cost_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<Vec<LandedCostAllocation>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                tenant_id, landed_cost_allocation_id, landed_cost_id, landed_cost_line_id,
                target_type, target_id, allocated_amount_cents, created_at, updated_at, deleted_at
            FROM landed_cost_allocations
            WHERE tenant_id = $1 AND landed_cost_id = $2 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
            tenant_id,
            landed_cost_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| LandedCostAllocation {
                tenant_id: r.tenant_id,
                landed_cost_allocation_id: r.landed_cost_allocation_id,
                landed_cost_id: r.landed_cost_id,
                landed_cost_line_id: r.landed_cost_line_id,
                target_type: Self::parse_target_type(&r.target_type),
                target_id: r.target_id,
                allocated_amount_cents: r.allocated_amount_cents,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            })
            .collect())
    }

    async fn has_allocations(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<bool, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM landed_cost_allocations
                WHERE tenant_id = $1 AND landed_cost_id = $2 AND deleted_at IS NULL
            ) as "exists!"
            "#,
            tenant_id,
            landed_cost_id,
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row.exists)
    }
}

#[async_trait]
impl AllocationTargetRepository for PgLandedCostRepository {
    async fn get_grn_item_targets(
        &self,
        tenant_id: Uuid,
        grn_id: Uuid,
    ) -> Result<Vec<AllocationTarget>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                receipt_item_id as target_id,
                COALESCE(line_total, 0) as "value_cents!"
            FROM goods_receipt_items
            WHERE tenant_id = $1 AND receipt_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            grn_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| AllocationTarget {
                target_type: TargetType::GrnItem,
                target_id: r.target_id,
                value_cents: r.value_cents,
            })
            .collect())
    }
}
