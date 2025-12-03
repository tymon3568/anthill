use async_trait::async_trait;
use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReorderRule, UpdateReorderRule,
};
use inventory_service_core::repositories::replenishment::ReorderRuleRepository;
use inventory_service_core::AppError;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of ReorderRuleRepository
pub struct PgReorderRuleRepository {
    pool: PgPool,
}

impl PgReorderRuleRepository {
    /// Create a new PostgreSQL reorder rule repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReorderRuleRepository for PgReorderRuleRepository {
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
    ) -> Result<Option<ReorderRule>, AppError> {
        let rule = sqlx::query_as!(
            ReorderRule,
            r#"
            SELECT
                rule_id, tenant_id, product_id, warehouse_id,
                reorder_point, min_quantity, max_quantity,
                lead_time_days, safety_stock,
                created_at, updated_at, deleted_at
            FROM reorder_rules
            WHERE tenant_id = $1 AND rule_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rule_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule)
    }

    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<ReorderRule>, AppError> {
        let rules = if let Some(wh_id) = warehouse_id {
            sqlx::query_as!(
                ReorderRule,
                r#"
                SELECT
                    rule_id, tenant_id, product_id, warehouse_id,
                    reorder_point, min_quantity, max_quantity,
                    lead_time_days, safety_stock,
                    created_at, updated_at, deleted_at
                FROM reorder_rules
                WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $3 AND deleted_at IS NULL
                ORDER BY created_at
                "#,
                tenant_id,
                product_id,
                wh_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                ReorderRule,
                r#"
                SELECT
                    rule_id, tenant_id, product_id, warehouse_id,
                    reorder_point, min_quantity, max_quantity,
                    lead_time_days, safety_stock,
                    created_at, updated_at, deleted_at
                FROM reorder_rules
                WHERE tenant_id = $1 AND product_id = $2 AND deleted_at IS NULL
                ORDER BY created_at
                "#,
                tenant_id,
                product_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rules)
    }

    async fn find_all_active(&self, tenant_id: Uuid) -> Result<Vec<ReorderRule>, AppError> {
        let rules = sqlx::query_as!(
            ReorderRule,
            r#"
            SELECT
                rule_id, tenant_id, product_id, warehouse_id,
                reorder_point, min_quantity, max_quantity,
                lead_time_days, safety_stock,
                created_at, updated_at, deleted_at
            FROM reorder_rules
            WHERE tenant_id = $1 AND deleted_at IS NULL
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }

    async fn create(
        &self,
        tenant_id: Uuid,
        rule: CreateReorderRule,
    ) -> Result<ReorderRule, AppError> {
        let new_rule = sqlx::query_as!(
            ReorderRule,
            r#"
            INSERT INTO reorder_rules (
                tenant_id, product_id, warehouse_id,
                reorder_point, min_quantity, max_quantity,
                lead_time_days, safety_stock
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                rule_id, tenant_id, product_id, warehouse_id,
                reorder_point, min_quantity, max_quantity,
                lead_time_days, safety_stock,
                created_at, updated_at, deleted_at
            "#,
            tenant_id,
            rule.product_id,
            rule.warehouse_id,
            rule.reorder_point,
            rule.min_quantity,
            rule.max_quantity,
            rule.lead_time_days,
            rule.safety_stock
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Reorder rule for product {} not found", rule.product_id))
        })?;

        Ok(new_rule)
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
        updates: UpdateReorderRule,
    ) -> Result<ReorderRule, AppError> {
        let updated_rule = sqlx::query_as!(
            ReorderRule,
            r#"
            UPDATE reorder_rules
            SET
                reorder_point = COALESCE($3, reorder_point),
                min_quantity = COALESCE($4, min_quantity),
                max_quantity = COALESCE($5, max_quantity),
                lead_time_days = COALESCE($6, lead_time_days),
                safety_stock = COALESCE($7, safety_stock),
                updated_at = NOW()
            WHERE tenant_id = $1 AND rule_id = $2 AND deleted_at IS NULL
            RETURNING
                rule_id, tenant_id, product_id, warehouse_id,
                reorder_point, min_quantity, max_quantity,
                lead_time_days, safety_stock,
                created_at, updated_at, deleted_at
            "#,
            tenant_id,
            rule_id,
            updates.reorder_point,
            updates.min_quantity,
            updates.max_quantity,
            updates.lead_time_days,
            updates.safety_stock
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Reorder rule {} not found", rule_id)))?;

        Ok(updated_rule)
    }

    async fn delete(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE reorder_rules
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND rule_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            rule_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
