use async_trait::async_trait;
use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
use inventory_service_core::repositories::replenishment::ReorderRuleRepository;

use inventory_service_core::repositories::InventoryLevelRepository;
use inventory_service_core::services::replenishment::ReplenishmentService;
use inventory_service_core::AppError;
use shared_events::{EventEnvelope, NatsClient, ReorderTriggeredEvent};
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL implementation of ReplenishmentService
pub struct PgReplenishmentService {
    reorder_repo: Arc<dyn ReorderRuleRepository>,
    inventory_repo: Arc<dyn InventoryLevelRepository>,
    nats_client: Option<Arc<NatsClient>>,
}

impl PgReplenishmentService {
    /// Create a new replenishment service
    pub fn new(
        reorder_repo: Arc<dyn ReorderRuleRepository>,
        inventory_repo: Arc<dyn InventoryLevelRepository>,
        nats_client: Option<Arc<NatsClient>>,
    ) -> Self {
        Self {
            reorder_repo,
            inventory_repo,
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
            if let Some(level) = self
                .inventory_repo
                .find_by_product(tenant_id, wh_id, product_id)
                .await?
            {
                level.available_quantity
            } else {
                0
            }
        } else {
            // TODO: Implement aggregation across warehouses once supported at the repository level.
            // For now we assume reorder rules are warehouse-specific; warn loudly if this is hit.
            tracing::warn!(
                "calculate_projected_quantity called without warehouse_id; \
cross-warehouse aggregation not implemented – treating available as 0"
            );
            0
        };

        // Projected quantity = available + incoming - reserved
        // Currently simplified: projected = available (incoming = 0, reserved = 0)
        // TODO: Implement full calculation with incoming stock moves and reserved quantities
        Ok(available)
    }

    fn compute_replenishment_decision(
        &self,
        rule: &ReorderRule,
        projected_quantity: i64,
    ) -> (i64, i64, bool, i64) {
        let effective_reorder_point = rule.reorder_point.saturating_add(rule.safety_stock);
        let needs_replenishment = projected_quantity < effective_reorder_point;
        let target_quantity = rule.max_quantity.saturating_add(rule.safety_stock);
        let suggested_order_quantity = if needs_replenishment {
            (target_quantity - projected_quantity).max(rule.min_quantity)
        } else {
            0
        };

        (
            effective_reorder_point,
            target_quantity,
            needs_replenishment,
            suggested_order_quantity,
        )
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
        let mut results = Vec::with_capacity(rules.len());

        for rule in rules {
            let projected_quantity = self
                .calculate_projected_quantity(tenant_id, rule.product_id, rule.warehouse_id)
                .await?;
            let current_quantity = projected_quantity;
            let (
                effective_reorder_point,
                _target_quantity,
                needs_replenishment,
                suggested_order_quantity,
            ) = self.compute_replenishment_decision(&rule, projected_quantity);

            let action_taken = if needs_replenishment {
                if let Some(nats) = &self.nats_client {
                    let event = ReorderTriggeredEvent {
                        event_id: uuid::Uuid::now_v7(),
                        tenant_id,
                        product_id: rule.product_id,
                        warehouse_id: rule.warehouse_id,
                        current_quantity,
                        projected_quantity,
                        reorder_point: effective_reorder_point,
                        suggested_order_quantity,
                        rule_id: rule.rule_id,
                        triggered_at: chrono::Utc::now(),
                    };
                    let envelope = EventEnvelope::new("inventory.reorder.triggered", event);
                    match nats
                        .publish_event("inventory.reorder.triggered".to_string(), &envelope)
                        .await
                    {
                        Ok(_) => Some("Reorder triggered event published".to_string()),
                        Err(e) => {
                            tracing::warn!("Failed to publish reorder event: {}", e);
                            Some("Reorder needed but event publishing failed".to_string())
                        },
                    }
                } else {
                    Some("Reorder needed but event publishing disabled".to_string())
                }
            } else {
                None
            };

            results.push(ReplenishmentCheckResult {
                product_id: rule.product_id,
                warehouse_id: rule.warehouse_id,
                current_quantity,
                projected_quantity,
                reorder_point: effective_reorder_point,
                suggested_order_quantity,
                needs_replenishment,
                action_taken,
            });
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

        if warehouse_id.is_none() && rules.len() > 1 {
            return Err(AppError::ValidationError(
                "Multiple reorder rules found; specify warehouse_id".to_string(),
            ));
        }

        let rule = if let Some(wh) = warehouse_id {
            rules
                .iter()
                .find(|r| r.warehouse_id == Some(wh))
                .unwrap_or(&rules[0])
        } else {
            &rules[0]
        };

        // Calculate projected quantity
        let projected_quantity = self
            .calculate_projected_quantity(tenant_id, product_id, warehouse_id)
            .await?;

        // Get current quantity (same as projected for now)
        let current_quantity = projected_quantity;

        let (
            effective_reorder_point,
            _target_quantity,
            needs_replenishment,
            suggested_order_quantity,
        ) = self.compute_replenishment_decision(rule, projected_quantity);

        let mut action_taken = None;

        // Publish reorder triggered event if needed
        if needs_replenishment {
            if let Some(nats) = &self.nats_client {
                let event = ReorderTriggeredEvent {
                    event_id: uuid::Uuid::now_v7(),
                    tenant_id,
                    product_id,
                    warehouse_id,
                    current_quantity,
                    projected_quantity,
                    reorder_point: effective_reorder_point,
                    suggested_order_quantity,
                    rule_id: rule.rule_id,
                    triggered_at: chrono::Utc::now(),
                };
                let envelope = EventEnvelope::new("inventory.reorder.triggered", event);
                match nats
                    .publish_event("inventory.reorder.triggered".to_string(), &envelope)
                    .await
                {
                    Ok(_) => {
                        action_taken = Some("Reorder triggered event published".to_string());
                    },
                    Err(e) => {
                        tracing::warn!("Failed to publish reorder event: {}", e);
                        action_taken =
                            Some("Reorder needed but event publishing failed".to_string());
                    },
                }
            } else {
                action_taken = Some("Reorder needed but event publishing disabled".to_string());
            }
        }

        Ok(ReplenishmentCheckResult {
            product_id,
            warehouse_id,
            current_quantity,
            projected_quantity,
            reorder_point: effective_reorder_point,
            suggested_order_quantity,
            needs_replenishment,
            action_taken,
        })
    }
}
