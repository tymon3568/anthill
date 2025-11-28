use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::models::{RmaItem, RmaRequest, RmaStatus};
use inventory_service_core::repositories::{RmaItemRepository, RmaRepository};
use shared_error::AppError;

pub struct PgRmaRepository {
    pool: Arc<PgPool>,
}

impl PgRmaRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RmaRepository for PgRmaRepository {
    async fn create(&self, rma: &RmaRequest) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO rma_requests (
                rma_id, rma_number, tenant_id, customer_id, original_delivery_id,
                status, return_reason, notes, total_items, total_value, currency_code,
                created_by, updated_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            "#,
            rma.rma_id,
            rma.rma_number,
            rma.tenant_id,
            rma.customer_id,
            rma.original_delivery_id,
            rma.status.to_string(),
            rma.return_reason,
            rma.notes,
            rma.total_items,
            rma.total_value,
            rma.currency_code,
            rma.created_by,
            rma.updated_by,
            rma.created_at,
            rma.updated_at,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        rma_id: Uuid,
    ) -> Result<Option<RmaRequest>, AppError> {
        let result = sqlx::query_as!(
            RmaRequest,
            r#"
            SELECT
                rma_id, rma_number, tenant_id, customer_id, original_delivery_id,
                status as "status: _",
                return_reason, notes, total_items, total_value, currency_code,
                created_by, updated_by, created_at, updated_at, deleted_at
            FROM rma_requests
            WHERE tenant_id = $1 AND rma_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rma_id,
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn find_by_number(
        &self,
        tenant_id: Uuid,
        rma_number: &str,
    ) -> Result<Option<RmaRequest>, AppError> {
        let result = sqlx::query_as!(
            RmaRequest,
            r#"
            SELECT
                rma_id, rma_number, tenant_id, customer_id, original_delivery_id,
                status as "status: _",
                return_reason, notes, total_items, total_value, currency_code,
                created_by, updated_by, created_at, updated_at, deleted_at
            FROM rma_requests
            WHERE tenant_id = $1 AND rma_number = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rma_number
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, rma: &RmaRequest) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE rma_requests SET
                rma_number = $3, customer_id = $4, original_delivery_id = $5,
                status = $6, return_reason = $7, notes = $8, total_items = $9,
                total_value = $10, currency_code = $11, updated_by = $12, updated_at = $13
            WHERE tenant_id = $1 AND rma_id = $2 AND deleted_at IS NULL
            "#,
            rma.tenant_id,
            rma.rma_id,
            rma.rma_number,
            rma.customer_id,
            rma.original_delivery_id,
            rma.status.to_string(),
            rma.return_reason,
            rma.notes,
            rma.total_items,
            rma.total_value,
            rma.currency_code,
            rma.updated_by,
            rma.updated_at
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn update_status(
        &self,
        tenant_id: Uuid,
        rma_id: Uuid,
        status: RmaStatus,
        updated_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE rma_requests SET
                status = $3, updated_by = $4, updated_at = NOW()
            WHERE tenant_id = $1 AND rma_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rma_id,
            status.to_string(),
            updated_by
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<RmaRequest>, AppError> {
        let rows = sqlx::query_as!(
            RmaRequest,
            r#"
            SELECT
                rma_id, rma_number, tenant_id, customer_id, original_delivery_id,
                status as "status: _",
                return_reason, notes, total_items, total_value, currency_code,
                created_by, updated_by, created_at, updated_at, deleted_at
            FROM rma_requests
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit.unwrap_or(50),
            offset.unwrap_or(0)
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }
}

pub struct PgRmaItemRepository {
    pool: Arc<PgPool>,
}

impl PgRmaItemRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RmaItemRepository for PgRmaItemRepository {
    async fn create(&self, item: &RmaItem) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO rma_items (
                rma_item_id, tenant_id, rma_id, product_id, variant_id,
                quantity_returned, condition, action, unit_cost, line_total,
                notes, created_by, updated_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            "#,
            item.rma_item_id,
            item.tenant_id,
            item.rma_id,
            item.product_id,
            item.variant_id,
            item.quantity_returned,
            item.condition.to_string(),
            item.action.to_string(),
            item.unit_cost,
            item.line_total,
            item.notes,
            item.created_by,
            item.updated_by,
            item.created_at,
            item.updated_at,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        rma_item_id: Uuid,
    ) -> Result<Option<RmaItem>, AppError> {
        let result = sqlx::query_as!(
            RmaItem,
            r#"
            SELECT
                rma_item_id, tenant_id, rma_id, product_id, variant_id,
                quantity_returned, condition as "condition: _", action as "action: _",
                unit_cost, line_total, notes, created_by, updated_by,
                created_at, updated_at, deleted_at
            FROM rma_items
            WHERE tenant_id = $1 AND rma_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rma_item_id,
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn find_by_rma_id(
        &self,
        tenant_id: Uuid,
        rma_id: Uuid,
    ) -> Result<Vec<RmaItem>, AppError> {
        let result = sqlx::query_as!(
            RmaItem,
            r#"
            SELECT
                rma_item_id, tenant_id, rma_id, product_id, variant_id,
                quantity_returned, condition as "condition: _", action as "action: _",
                unit_cost, line_total, notes, created_by, updated_by,
                created_at, updated_at, deleted_at
            FROM rma_items
            WHERE tenant_id = $1 AND rma_id = $2 AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            tenant_id,
            rma_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, item: &RmaItem) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE rma_items SET
                product_id = $4, variant_id = $5, quantity_returned = $6,
                condition = $7, action = $8, unit_cost = $9, line_total = $10,
                notes = $11, updated_by = $12, updated_at = $13
            WHERE tenant_id = $1 AND rma_item_id = $2 AND rma_id = $3 AND deleted_at IS NULL
            "#,
            item.tenant_id,
            item.rma_item_id,
            item.rma_id,
            item.product_id,
            item.variant_id,
            item.quantity_returned,
            item.condition.to_string(),
            item.action.to_string(),
            item.unit_cost,
            item.line_total,
            item.notes,
            item.updated_by,
            item.updated_at,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: Uuid, rma_item_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE rma_items SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND rma_item_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rma_item_id,
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
