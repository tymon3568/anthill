use crate::domains::replenishment::{CreateReorderRule, ReorderRule, UpdateReorderRule};
use crate::error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for reorder rules
#[async_trait]
pub trait ReorderRuleRepository: Send + Sync {
    /// Find reorder rule by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
    ) -> Result<Option<ReorderRule>, AppError>;

    /// Find reorder rules for a product (optionally filtered by warehouse)
    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<ReorderRule>, AppError>;

    /// Find all active reorder rules for a tenant
    async fn find_all_active(&self, tenant_id: Uuid) -> Result<Vec<ReorderRule>, AppError>;

    /// Create a new reorder rule
    async fn create(
        &self,
        tenant_id: Uuid,
        rule: CreateReorderRule,
    ) -> Result<ReorderRule, AppError>;

    /// Update an existing reorder rule
    async fn update(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
        updates: UpdateReorderRule,
    ) -> Result<ReorderRule, AppError>;

    /// Soft delete a reorder rule
    async fn delete(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<(), AppError>;
}
