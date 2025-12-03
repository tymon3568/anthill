use crate::domains::replenishment::{
    CreateReorderRule, ReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
use crate::error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

/// Service trait for automated stock replenishment
#[async_trait]
pub trait ReplenishmentService: Send + Sync {
    /// Create a new reorder rule
    async fn create_reorder_rule(
        &self,
        tenant_id: Uuid,
        rule: CreateReorderRule,
    ) -> Result<ReorderRule, AppError>;

    /// Get reorder rule by ID
    async fn get_reorder_rule(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
    ) -> Result<Option<ReorderRule>, AppError>;

    /// Update reorder rule
    async fn update_reorder_rule(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
        updates: UpdateReorderRule,
    ) -> Result<ReorderRule, AppError>;

    /// Delete reorder rule
    async fn delete_reorder_rule(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<(), AppError>;

    /// List reorder rules for a product
    async fn list_reorder_rules_for_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<ReorderRule>, AppError>;

    /// Run replenishment check for all active rules
    async fn run_replenishment_check(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<ReplenishmentCheckResult>, AppError>;

    /// Run replenishment check for a specific product
    async fn check_product_replenishment(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<ReplenishmentCheckResult, AppError>;
}
