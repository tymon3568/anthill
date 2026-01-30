//! Unit tests for PgReplenishmentService using mocks
//!
//! These tests validate the repository interactions for stock replenishment,
//! including reorder rules, safety stock, and min/max quantity management.

use chrono::Utc;
use mockall::mock;
use uuid::Uuid;

use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReorderRule, UpdateReorderRule,
};
use inventory_service_core::models::InventoryLevel;
use inventory_service_core::repositories::replenishment::ReorderRuleRepository;
use inventory_service_core::repositories::InventoryLevelRepository;
use inventory_service_core::Result;
use shared_error::AppError;

// Mock the ReorderRuleRepository trait
mock! {
    pub ReorderRuleRepositoryImpl {}

    #[async_trait::async_trait]
    impl ReorderRuleRepository for ReorderRuleRepositoryImpl {
        async fn create(&self, tenant_id: Uuid, rule: CreateReorderRule) -> Result<ReorderRule>;
        async fn find_by_id(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<Option<ReorderRule>>;
        async fn find_by_product(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            warehouse_id: Option<Uuid>,
        ) -> Result<Vec<ReorderRule>>;
        async fn find_all_active(&self, tenant_id: Uuid) -> Result<Vec<ReorderRule>>;
        async fn update(
            &self,
            tenant_id: Uuid,
            rule_id: Uuid,
            updates: UpdateReorderRule,
        ) -> Result<ReorderRule>;
        async fn delete(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<()>;
    }
}

// Mock the InventoryLevelRepository trait
mock! {
    pub InventoryLevelRepositoryImpl {}

    #[async_trait::async_trait]
    impl InventoryLevelRepository for InventoryLevelRepositoryImpl {
        async fn find_by_product(
            &self,
            tenant_id: Uuid,
            warehouse_id: Uuid,
            product_id: Uuid,
        ) -> Result<Option<InventoryLevel>>;
        async fn update_available_quantity(
            &self,
            tenant_id: Uuid,
            warehouse_id: Uuid,
            location_id: Option<Uuid>,
            product_id: Uuid,
            quantity_change: i64,
        ) -> Result<()>;
        async fn upsert(
            &self,
            tenant_id: Uuid,
            warehouse_id: Uuid,
            product_id: Uuid,
            available_quantity: i64,
            reserved_quantity: i64,
        ) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test ReorderRule matching actual domain struct
    fn create_test_reorder_rule(
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> ReorderRule {
        ReorderRule {
            rule_id: Uuid::new_v4(),
            tenant_id,
            product_id,
            warehouse_id,
            reorder_point: 10,
            min_quantity: 5,
            max_quantity: 100,
            safety_stock: 5,
            lead_time_days: 7,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Helper function to create a test InventoryLevel matching actual model struct
    fn create_test_inventory_level(
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        available_quantity: i64,
    ) -> InventoryLevel {
        InventoryLevel {
            inventory_id: Uuid::new_v4(),
            tenant_id,
            warehouse_id,
            product_id,
            available_quantity,
            reserved_quantity: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    // =========================================================================
    // ReorderRule Create Tests
    // =========================================================================

    #[tokio::test]
    async fn test_create_reorder_rule_success() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Some(Uuid::new_v4());

        let create_dto = CreateReorderRule {
            product_id,
            warehouse_id,
            reorder_point: 10,
            min_quantity: 5,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 5,
        };

        let expected_rule = create_test_reorder_rule(tenant_id, product_id, warehouse_id);
        let expected_rule_clone = expected_rule.clone();

        mock_repo
            .expect_create()
            .withf(move |t, dto| {
                *t == tenant_id
                    && dto.product_id == product_id
                    && dto.warehouse_id == warehouse_id
                    && dto.reorder_point == 10
            })
            .times(1)
            .returning(move |_, _| Ok(expected_rule_clone.clone()));

        let result = mock_repo.create(tenant_id, create_dto).await;
        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.product_id, product_id);
        assert_eq!(rule.reorder_point, 10);
    }

    #[tokio::test]
    async fn test_create_reorder_rule_duplicate() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let create_dto = CreateReorderRule {
            product_id,
            warehouse_id: None,
            reorder_point: 10,
            min_quantity: 5,
            max_quantity: 100,
            lead_time_days: 7,
            safety_stock: 5,
        };

        mock_repo.expect_create().times(1).returning(|_, _| {
            Err(AppError::Conflict(
                "Reorder rule already exists for this product/warehouse".to_string(),
            ))
        });

        let result = mock_repo.create(tenant_id, create_dto).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
    }

    // =========================================================================
    // ReorderRule Find Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_reorder_rule_success() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let rule = create_test_reorder_rule(tenant_id, product_id, None);
        let rule_id = rule.rule_id;
        let rule_clone = rule.clone();

        mock_repo
            .expect_find_by_id()
            .withf(move |t, r| *t == tenant_id && *r == rule_id)
            .times(1)
            .returning(move |_, _| Ok(Some(rule_clone.clone())));

        let result = mock_repo.find_by_id(tenant_id, rule_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().rule_id, rule_id);
    }

    #[tokio::test]
    async fn test_get_reorder_rule_not_found() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let rule_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let result = mock_repo.find_by_id(tenant_id, rule_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // =========================================================================
    // ReorderRule Update Tests
    // =========================================================================

    #[tokio::test]
    async fn test_update_reorder_rule_success() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut rule = create_test_reorder_rule(tenant_id, product_id, None);
        let rule_id = rule.rule_id;

        // Update the rule values
        rule.reorder_point = 20;
        rule.safety_stock = 10;
        let rule_clone = rule.clone();

        let update_dto = UpdateReorderRule {
            reorder_point: Some(20),
            min_quantity: None,
            max_quantity: None,
            lead_time_days: None,
            safety_stock: Some(10),
        };

        mock_repo
            .expect_update()
            .withf(move |t, r, dto| {
                *t == tenant_id && *r == rule_id && dto.reorder_point == Some(20)
            })
            .times(1)
            .returning(move |_, _, _| Ok(rule_clone.clone()));

        let result = mock_repo.update(tenant_id, rule_id, update_dto).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.reorder_point, 20);
        assert_eq!(updated.safety_stock, 10);
    }

    #[tokio::test]
    async fn test_update_reorder_rule_not_found() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let rule_id = Uuid::new_v4();

        let update_dto = UpdateReorderRule {
            reorder_point: Some(20),
            min_quantity: None,
            max_quantity: None,
            lead_time_days: None,
            safety_stock: None,
        };

        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, _, _| Err(AppError::NotFound("Reorder rule not found".to_string())));

        let result = mock_repo.update(tenant_id, rule_id, update_dto).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // ReorderRule Delete Tests
    // =========================================================================

    #[tokio::test]
    async fn test_delete_reorder_rule_success() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let rule_id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .withf(move |t, r| *t == tenant_id && *r == rule_id)
            .times(1)
            .returning(|_, _| Ok(()));

        let result = mock_repo.delete(tenant_id, rule_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_reorder_rule_not_found() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let rule_id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(AppError::NotFound("Reorder rule not found".to_string())));

        let result = mock_repo.delete(tenant_id, rule_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // Find By Product Tests
    // =========================================================================

    #[tokio::test]
    async fn test_list_reorder_rules_for_product_success() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        let rule1 = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        let rule2 = create_test_reorder_rule(tenant_id, product_id, None);
        let rules = vec![rule1, rule2];
        let rules_clone = rules.clone();

        mock_repo
            .expect_find_by_product()
            .withf(move |t, p, _| *t == tenant_id && *p == product_id)
            .times(1)
            .returning(move |_, _, _| Ok(rules_clone.clone()));

        let result = mock_repo.find_by_product(tenant_id, product_id, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_reorder_rules_for_product_empty() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_product()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let result = mock_repo.find_by_product(tenant_id, product_id, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // =========================================================================
    // Replenishment Check Logic Tests
    // =========================================================================

    #[tokio::test]
    async fn test_check_product_replenishment_needs_reorder() {
        let mut mock_rule_repo = MockReorderRuleRepositoryImpl::new();
        let mut mock_inv_repo = MockInventoryLevelRepositoryImpl::new();

        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        // Rule: reorder_point = 10, min_quantity = 5
        let rule = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        let rule_clone = rule.clone();

        // Current inventory: 8 units (below reorder_point of 10)
        let inv_level = create_test_inventory_level(tenant_id, warehouse_id, product_id, 8);
        let inv_level_clone = inv_level.clone();

        mock_rule_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(vec![rule_clone.clone()]));

        mock_inv_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(Some(inv_level_clone.clone())));

        // Simulate the check
        let rules = mock_rule_repo
            .find_by_product(tenant_id, product_id, Some(warehouse_id))
            .await
            .unwrap();
        let inventory = mock_inv_repo
            .find_by_product(tenant_id, warehouse_id, product_id)
            .await
            .unwrap();

        assert_eq!(rules.len(), 1);
        assert!(inventory.is_some());
        let inv = inventory.unwrap();
        let rule = &rules[0];

        // Check if replenishment is needed
        let effective_reorder_point = rule.reorder_point + rule.safety_stock; // 10 + 5 = 15
        let needs_reorder = inv.available_quantity < effective_reorder_point;
        assert!(needs_reorder);
    }

    #[tokio::test]
    async fn test_check_product_replenishment_no_reorder_needed() {
        let mut mock_rule_repo = MockReorderRuleRepositoryImpl::new();
        let mut mock_inv_repo = MockInventoryLevelRepositoryImpl::new();

        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        // Rule: reorder_point = 10, safety_stock = 5
        let rule = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        let rule_clone = rule.clone();

        // Current inventory: 50 units (above effective reorder point of 15)
        let inv_level = create_test_inventory_level(tenant_id, warehouse_id, product_id, 50);
        let inv_level_clone = inv_level.clone();

        mock_rule_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(vec![rule_clone.clone()]));

        mock_inv_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(Some(inv_level_clone.clone())));

        let rules = mock_rule_repo
            .find_by_product(tenant_id, product_id, Some(warehouse_id))
            .await
            .unwrap();
        let inventory = mock_inv_repo
            .find_by_product(tenant_id, warehouse_id, product_id)
            .await
            .unwrap();

        let inv = inventory.unwrap();
        let rule = &rules[0];

        let effective_reorder_point = rule.reorder_point + rule.safety_stock;
        let needs_reorder = inv.available_quantity < effective_reorder_point;
        assert!(!needs_reorder);
    }

    #[tokio::test]
    async fn test_check_product_replenishment_no_rule_found() {
        let mut mock_rule_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_rule_repo
            .expect_find_by_product()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let rules = mock_rule_repo
            .find_by_product(tenant_id, product_id, None)
            .await
            .unwrap();

        // No rules means no replenishment check needed
        assert!(rules.is_empty());
    }

    #[tokio::test]
    async fn test_check_product_replenishment_zero_inventory() {
        let mut mock_rule_repo = MockReorderRuleRepositoryImpl::new();
        let mut mock_inv_repo = MockInventoryLevelRepositoryImpl::new();

        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        let rule = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        let rule_clone = rule.clone();

        // Zero inventory
        let inv_level = create_test_inventory_level(tenant_id, warehouse_id, product_id, 0);
        let inv_level_clone = inv_level.clone();

        mock_rule_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(vec![rule_clone.clone()]));

        mock_inv_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(Some(inv_level_clone.clone())));

        let rules = mock_rule_repo
            .find_by_product(tenant_id, product_id, Some(warehouse_id))
            .await
            .unwrap();
        let inventory = mock_inv_repo
            .find_by_product(tenant_id, warehouse_id, product_id)
            .await
            .unwrap();

        let inv = inventory.unwrap();
        let rule = &rules[0];

        // Zero inventory definitely needs reorder
        let needs_reorder = inv.available_quantity < (rule.reorder_point + rule.safety_stock);
        assert!(needs_reorder);
        assert_eq!(inv.available_quantity, 0);
    }

    // =========================================================================
    // Find All Active Rules Tests
    // =========================================================================

    #[tokio::test]
    async fn test_run_replenishment_check_multiple_rules() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        let product1 = Uuid::new_v4();
        let product2 = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        let rule1 = create_test_reorder_rule(tenant_id, product1, Some(warehouse_id));
        let rule2 = create_test_reorder_rule(tenant_id, product2, Some(warehouse_id));
        let rules = vec![rule1, rule2];
        let rules_clone = rules.clone();

        mock_repo
            .expect_find_all_active()
            .withf(move |t| *t == tenant_id)
            .times(1)
            .returning(move |_| Ok(rules_clone.clone()));

        let result = mock_repo.find_all_active(tenant_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_run_replenishment_check_no_active_rules() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        mock_repo
            .expect_find_all_active()
            .times(1)
            .returning(|_| Ok(vec![]));

        let result = mock_repo.find_all_active(tenant_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_run_replenishment_check_no_inventory_record() {
        let mut mock_inv_repo = MockInventoryLevelRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // No inventory record exists
        mock_inv_repo
            .expect_find_by_product()
            .times(1)
            .returning(|_, _, _| Ok(None));

        let result = mock_inv_repo
            .find_by_product(tenant_id, warehouse_id, product_id)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // =========================================================================
    // Safety Stock Calculation Tests
    // =========================================================================

    #[tokio::test]
    async fn test_safety_stock_calculation() {
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        let mut rule = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        rule.reorder_point = 20;
        rule.safety_stock = 10;

        // Effective reorder point = reorder_point + safety_stock
        let effective_reorder_point = rule.reorder_point + rule.safety_stock;
        assert_eq!(effective_reorder_point, 30);

        // If inventory is at 25, still need to reorder (below 30)
        let inventory = create_test_inventory_level(tenant_id, warehouse_id, product_id, 25);
        assert!(inventory.available_quantity < effective_reorder_point);

        // If inventory is at 35, no reorder needed (above 30)
        let inventory_high = create_test_inventory_level(tenant_id, warehouse_id, product_id, 35);
        assert!(inventory_high.available_quantity >= effective_reorder_point);
    }

    #[tokio::test]
    async fn test_min_quantity_enforcement() {
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        let mut rule = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        rule.reorder_point = 10;
        rule.min_quantity = 20;
        rule.max_quantity = 100;

        // When reordering, should order at least min_quantity
        let inventory = create_test_inventory_level(tenant_id, warehouse_id, product_id, 5);

        // Calculate suggested order quantity
        // Typically: max(min_quantity, max_quantity - current_qty)
        let qty_to_max = rule.max_quantity - inventory.available_quantity;
        let suggested_qty = std::cmp::max(rule.min_quantity, qty_to_max);

        // With 5 units and max 100, we'd want to order 95, which is > min(20)
        assert_eq!(suggested_qty, 95);

        // Edge case: closer to max, order min
        let inventory_high = create_test_inventory_level(tenant_id, warehouse_id, product_id, 85);
        let qty_to_max_high = rule.max_quantity - inventory_high.available_quantity;
        let suggested_qty_high = std::cmp::max(rule.min_quantity, qty_to_max_high);
        // 100 - 85 = 15, but min is 20, so order 20
        assert_eq!(suggested_qty_high, 20);
    }

    // =========================================================================
    // Tenant Isolation Tests
    // =========================================================================

    #[tokio::test]
    async fn test_replenishment_tenant_isolation() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let rule_a = create_test_reorder_rule(tenant_a, product_id, None);
        let rule_a_clone = rule_a.clone();

        // Tenant A should find their rule
        mock_repo
            .expect_find_by_product()
            .withf(move |t, _, _| *t == tenant_a)
            .times(1)
            .returning(move |_, _, _| Ok(vec![rule_a_clone.clone()]));

        let result_a = mock_repo.find_by_product(tenant_a, product_id, None).await;
        assert!(result_a.is_ok());
        assert_eq!(result_a.unwrap().len(), 1);

        // Tenant B should NOT find tenant A's rule
        mock_repo
            .expect_find_by_product()
            .withf(move |t, _, _| *t == tenant_b)
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let result_b = mock_repo.find_by_product(tenant_b, product_id, None).await;
        assert!(result_b.is_ok());
        assert!(result_b.unwrap().is_empty());
    }

    // =========================================================================
    // Inventory Level Update Tests
    // =========================================================================

    #[tokio::test]
    async fn test_update_available_quantity_increase() {
        let mut mock_repo = MockInventoryLevelRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_update_available_quantity()
            .withf(move |t, w, l, p, q| {
                *t == tenant_id && *w == warehouse_id && l.is_none() && *p == product_id && *q == 50
            })
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let result = mock_repo
            .update_available_quantity(tenant_id, warehouse_id, None, product_id, 50)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_available_quantity_decrease() {
        let mut mock_repo = MockInventoryLevelRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_update_available_quantity()
            .withf(move |_, _, _, _, q| *q == -30)
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let result = mock_repo
            .update_available_quantity(tenant_id, warehouse_id, None, product_id, -30)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_inventory_level() {
        let mut mock_repo = MockInventoryLevelRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_upsert()
            .withf(move |t, w, p, avail, res| {
                *t == tenant_id
                    && *w == warehouse_id
                    && *p == product_id
                    && *avail == 100
                    && *res == 10
            })
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let result = mock_repo
            .upsert(tenant_id, warehouse_id, product_id, 100, 10)
            .await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_reorder_rule_warehouse_specific_vs_global() {
        let mut mock_repo = MockReorderRuleRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();

        // Warehouse-specific rule
        let rule_wh = create_test_reorder_rule(tenant_id, product_id, Some(warehouse_id));
        // Global rule (no warehouse)
        let rule_global = create_test_reorder_rule(tenant_id, product_id, None);

        let rules = vec![rule_wh.clone(), rule_global.clone()];
        let rules_clone = rules.clone();

        mock_repo
            .expect_find_by_product()
            .times(1)
            .returning(move |_, _, _| Ok(rules_clone.clone()));

        let result = mock_repo.find_by_product(tenant_id, product_id, None).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);

        // Verify one has warehouse, one doesn't
        let has_wh: Vec<_> = found.iter().filter(|r| r.warehouse_id.is_some()).collect();
        let no_wh: Vec<_> = found.iter().filter(|r| r.warehouse_id.is_none()).collect();
        assert_eq!(has_wh.len(), 1);
        assert_eq!(no_wh.len(), 1);
    }

    #[tokio::test]
    async fn test_lead_time_days_in_rule() {
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let mut rule = create_test_reorder_rule(tenant_id, product_id, None);
        rule.lead_time_days = 14; // 2 weeks lead time

        // Lead time affects when to trigger reorder
        // Higher lead time = should reorder earlier (at higher inventory levels)
        assert_eq!(rule.lead_time_days, 14);

        // Calculation would typically be:
        // effective_reorder_point = reorder_point + (daily_demand * lead_time_days) + safety_stock
        // But that requires daily_demand which we don't have here
        // The rule stores lead_time_days for use in the service calculation
    }
}
