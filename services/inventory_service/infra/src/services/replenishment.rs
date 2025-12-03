use async_trait::async_trait;
use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
use inventory_service_core::repositories::inventory_level::InventoryLevelRepository;
use inventory_service_core::repositories::replenishment::ReorderRuleRepository;
use inventory_service_core::repositories::stock::StockMoveRepository;
use inventory_service_core::services::replenishment::ReplenishmentService;
use inventory_service_core::AppError;
use shared_events::{EventEnvelope, NatsClient, ReorderTriggeredEvent};
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL implementation of ReplenishmentService
pub struct PgReplenishmentService {
    reorder_repo: Arc<dyn ReorderRuleRepository>,
    inventory_repo: Arc<dyn InventoryLevelRepository>,
    stock_move_repo: Arc<dyn StockMoveRepository>,
    nats_client: Option<Arc<NatsClient>>,
}

impl PgReplenishmentService {
    /// Create a new replenishment service
    pub fn new(
        reorder_repo: Arc<dyn ReorderRuleRepository>,
        inventory_repo: Arc<dyn InventoryLevelRepository>,
        stock_move_repo: Arc<dyn StockMoveRepository>,
        nats_client: Option<Arc<NatsClient>>,
    ) -> Self {
        Self {
            reorder_repo,
            inventory_repo,
            stock_move_repo,
            nats_client,
        }
    }

    /// Calculate projected quantity for a product at a warehouse
    async fn calculate_projected_quantity(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<i64, AppError> {
        // Get current available quantity
        let available = if let Some(wh_id) = warehouse_id {
            self.inventory_repo
                .get_available_quantity(tenant_id, wh_id, product_id)
                .await?
        } else {
            // Sum across all warehouses if no specific warehouse
            self.inventory_repo
                .get_total_available_quantity(tenant_id, product_id)
                .await?
        };

        // For simplicity, projected = available (incoming - reserved not implemented yet)
        // TODO: Add incoming stock moves and reserved quantities
        Ok(available)
    }
}

#[async_trait]
impl ReplenishmentService for PgReplenishmentService {
    async fn create_reorder_rule(
        &self,
        tenant_id: Uuid,
        rule: CreateReorderRule,
    ) -> Result<ReorderRule, AppError> {
        self.reorder_repo.create(tenant_id, rule).await
    }

    async fn get_reorder_rule(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
    ) -> Result<Option<ReorderRule>, AppError> {
        self.reorder_repo.find_by_id(tenant_id, rule_id).await
    }

    async fn update_reorder_rule(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
        updates: UpdateReorderRule,
    ) -> Result<ReorderRule, AppError> {
        self.reorder_repo.update(tenant_id, rule_id, updates).await
    }

    async fn delete_reorder_rule(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<(), AppError> {
        self.reorder_repo.delete(tenant_id, rule_id).await
    }

    async fn list_reorder_rules_for_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<ReorderRule>, AppError> {
        self.reorder_repo
            .find_by_product(tenant_id, product_id, warehouse_id)
            .await
    }

    async fn run_replenishment_check(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<ReplenishmentCheckResult>, AppError> {
        let rules = self.reorder_repo.find_all_active(tenant_id).await?;
        let mut results = Vec::new();

        for rule in rules {
            let result = self
                .check_product_replenishment(tenant_id, rule.product_id, rule.warehouse_id)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn check_product_replenishment(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<ReplenishmentCheckResult, AppError> {
        // Get reorder rule
        let rules = self
            .reorder_repo
            .find_by_product(tenant_id, product_id, warehouse_id)
            .await?;

        if rules.is_empty() {
            return Err(AppError::NotFound("No reorder rule found for product".to_string()));
        }

        let rule = &rules[0]; // Take first if multiple

        // Calculate projected quantity
        let projected_quantity = self
            .calculate_projected_quantity(tenant_id, product_id, warehouse_id)
            .await?;

        // Get current quantity (same as projected for now)
        let current_quantity = projected_quantity;

        // Check if needs replenishment
        let needs_replenishment = projected_quantity < rule.reorder_point;

        // Calculate suggested order quantity
        let suggested_order_quantity = if needs_replenishment {
            (rule.max_quantity - projected_quantity).max(rule.min_quantity)
        } else {
            0
        };

        // Publish reorder triggered event if needed
        if needs_replenishment {
            if let Some(nats) = &self.nats_client {
                let event = ReorderTriggeredEvent {
                    tenant_id,
                    product_id,
                    warehouse_id,
                    current_quantity,
                    projected_quantity,
                    reorder_point: rule.reorder_point,
                    suggested_order_quantity,
                    rule_id: rule.rule_id,
                };
                let envelope = EventEnvelope::new("inventory.reorder.triggered", event);
                if let Err(e) = nats
                    .publish_event("inventory.reorder.triggered".to_string(), &envelope)
                    .await
                {
                    tracing::warn!("Failed to publish reorder event: {}", e);
                }
            }
        }

        Ok(ReplenishmentCheckResult {
            product_id,
            warehouse_id,
            current_quantity,
            projected_quantity,
            reorder_point: rule.reorder_point,
            suggested_order_quantity,
            needs_replenishment,
            action_taken: if needs_replenishment && self.nats_client.is_some() {
                Some("Reorder triggered event published".to_string())
            } else if needs_replenishment {
                Some("Reorder needed but event publishing disabled".to_string())
            } else {
                None
            },
        })
    }
}
