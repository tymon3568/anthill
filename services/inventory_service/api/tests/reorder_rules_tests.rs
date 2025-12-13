//! Advanced Business Logic Tests for Reorder Rules
//!
//! Integration tests for reorder point calculations, quantity suggestions,
//! lead time considerations, and safety stock calculations.

mod business_logic_test_helpers;

use business_logic_test_helpers::{
    cleanup_reorder_test_data, create_inventory_level, create_replenishment_service,
    setup_test_pool, setup_test_tenant_product_warehouse,
};
use inventory_service_core::domains::replenishment::CreateReorderRule;
use inventory_service_core::services::ReplenishmentService;

// ============================================================================
// Reorder Quantity Calculation Tests
// ============================================================================

#[cfg(test)]
mod reorder_quantity_tests {
    use super::*;

    #[tokio::test]
    async fn test_reorder_quantity_respects_max() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Set inventory level very low (10 units)
        create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 10).await;

        // Create reorder rule: max_quantity = 100
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 20,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 5,
        };

        service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Failed to create reorder rule");

        // Check replenishment
        let result = service
            .check_product_replenishment(tenant_id, product_id, Some(warehouse_id))
            .await
            .expect("Replenishment check should succeed");

        // Suggested order quantity should be max - current = 100 - 10 = 90
        assert!(result.needs_replenishment);
        assert_eq!(result.suggested_order_quantity, 90);
        assert!(
            result.suggested_order_quantity <= 100,
            "Should not exceed max_quantity"
        );

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_reorder_quantity_at_least_min() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Set inventory level slightly below reorder point
        create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 45).await;

        // Create reorder rule with min_quantity = 30
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 30,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 5,
        };

        service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Failed to create reorder rule");

        // Check replenishment
        let result = service
            .check_product_replenishment(tenant_id, product_id, Some(warehouse_id))
            .await
            .expect("Replenishment check should succeed");

        assert!(result.needs_replenishment);
        // max - current = 100 - 45 = 55, which is >= min(30)
        assert!(
            result.suggested_order_quantity >= 30,
            "Should order at least min_quantity"
        );

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_no_replenishment_when_above_reorder_point() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Set inventory level above reorder point
        create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 100).await;

        // Create reorder rule
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 20,
            max_quantity: 150,
            lead_time_days: 7,
            safety_stock: 5,
        };

        service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Failed to create reorder rule");

        // Check replenishment
        let result = service
            .check_product_replenishment(tenant_id, product_id, Some(warehouse_id))
            .await
            .expect("Replenishment check should succeed");

        // Current quantity (100) > reorder_point (50), no replenishment needed
        assert!(!result.needs_replenishment);
        assert_eq!(result.suggested_order_quantity, 0);

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_reorder_with_zero_current_stock() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Set inventory level to zero
        create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 0).await;

        // Create reorder rule
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 20,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 10,
        };

        service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Failed to create reorder rule");

        // Check replenishment
        let result = service
            .check_product_replenishment(tenant_id, product_id, Some(warehouse_id))
            .await
            .expect("Replenishment check should succeed");

        assert!(result.needs_replenishment);
        assert_eq!(result.current_quantity, 0);
        // Suggested = max - current = 100 - 0 = 100
        assert_eq!(result.suggested_order_quantity, 100);

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }
}

// ============================================================================
// Safety Stock and Lead Time Tests
// ============================================================================

#[cfg(test)]
mod safety_stock_tests {
    use super::*;

    #[tokio::test]
    async fn test_safety_stock_in_reorder_calculation() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Set inventory level at safety stock + 1
        create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 11).await;

        // Create reorder rule with safety_stock = 10
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 20,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 10,
        };

        service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Failed to create reorder rule");

        // Check replenishment
        let result = service
            .check_product_replenishment(tenant_id, product_id, Some(warehouse_id))
            .await
            .expect("Replenishment check should succeed");

        // Current (11) < reorder_point (50), needs replenishment
        assert!(result.needs_replenishment);
        assert_eq!(result.reorder_point, 50);

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_lead_time_in_rule_creation() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Create reorder rule with lead_time_days = 14
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 100,
            min_quantity: 50,
            max_quantity: 200,
            lead_time_days: 14,
            safety_stock: 20,
        };

        let created = service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Failed to create reorder rule");

        // Verify rule was created with correct lead time
        assert_eq!(created.lead_time_days, 14);
        assert_eq!(created.reorder_point, 100);
        assert_eq!(created.safety_stock, 20);

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[cfg(test)]
mod reorder_edge_cases {
    use super::*;
    use shared_error::AppError;

    #[tokio::test]
    async fn test_replenishment_check_no_rule_returns_not_found() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Don't create any reorder rule, just check replenishment
        let result = service
            .check_product_replenishment(tenant_id, product_id, Some(warehouse_id))
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => {
                assert!(msg.contains("No reorder rule found") || msg.contains("not found"));
            }
            other => panic!("Expected NotFound error, got {:?}", other),
        }

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_reorder_rule_crud_operations() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Create rule
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 20,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 5,
        };

        let created = service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Create should succeed");

        assert_eq!(created.product_id, product_id);
        assert_eq!(created.reorder_point, 50);

        // Get rule
        let fetched = service
            .get_reorder_rule(tenant_id, created.rule_id)
            .await
            .expect("Get should succeed");

        assert!(fetched.is_some());
        let fetched_rule = fetched.unwrap();
        assert_eq!(fetched_rule.rule_id, created.rule_id);

        // Delete rule
        service
            .delete_reorder_rule(tenant_id, created.rule_id)
            .await
            .expect("Delete should succeed");

        // Verify deleted
        let after_delete = service
            .get_reorder_rule(tenant_id, created.rule_id)
            .await
            .expect("Get after delete should succeed");

        assert!(after_delete.is_none(), "Rule should be deleted");

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_list_reorder_rules_for_product() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id, warehouse_id) =
            setup_test_tenant_product_warehouse(&pool).await;
        let service = create_replenishment_service(&pool);

        // Create rule
        let rule = CreateReorderRule {
            product_id,
            warehouse_id: Some(warehouse_id),
            reorder_point: 50,
            min_quantity: 20,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 5,
        };

        service
            .create_reorder_rule(tenant_id, rule)
            .await
            .expect("Create should succeed");

        // List rules for product
        let rules = service
            .list_reorder_rules_for_product(tenant_id, product_id, Some(warehouse_id))
            .await
            .expect("List should succeed");

        assert!(!rules.is_empty());
        assert_eq!(rules[0].product_id, product_id);

        cleanup_reorder_test_data(&pool, tenant_id).await;
    }
}
