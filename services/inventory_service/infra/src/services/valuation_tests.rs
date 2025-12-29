//! Unit tests for ValuationService repository layer using mocks
//!
//! These tests validate the repository interactions for inventory valuation,
//! supporting FIFO, AVCO, and Standard costing methods.

use chrono::Utc;
use mockall::mock;
use mockall::predicate::*;
use uuid::Uuid;

use inventory_service_core::domains::inventory::valuation::{
    Valuation, ValuationHistory, ValuationLayer, ValuationMethod,
};
use inventory_service_core::repositories::valuation::{
    ValuationHistoryRepository, ValuationLayerRepository, ValuationRepository,
};
use inventory_service_core::Result;
use shared_error::AppError;

// Mock the ValuationRepository trait
mock! {
    pub ValuationRepositoryImpl {}

    #[async_trait::async_trait]
    impl ValuationRepository for ValuationRepositoryImpl {
        async fn find_by_product_id(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
        ) -> Result<Option<Valuation>>;

        async fn create(&self, valuation: &Valuation) -> Result<Valuation>;

        async fn update(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            valuation: &Valuation,
        ) -> Result<Valuation>;

        async fn set_valuation_method(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            method: ValuationMethod,
            updated_by: Option<Uuid>,
        ) -> Result<Valuation>;

        async fn set_standard_cost(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            standard_cost: i64,
            updated_by: Option<Uuid>,
        ) -> Result<Valuation>;

        async fn update_from_stock_move(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            quantity_change: i64,
            unit_cost: Option<i64>,
            updated_by: Option<Uuid>,
        ) -> Result<Valuation>;

        async fn adjust_cost(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            adjustment_amount: i64,
            reason: &str,
            updated_by: Option<Uuid>,
        ) -> Result<Valuation>;

        async fn revalue_inventory(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            new_unit_cost: i64,
            reason: &str,
            updated_by: Option<Uuid>,
        ) -> Result<Valuation>;
    }
}

// Mock the ValuationLayerRepository trait
mock! {
    pub ValuationLayerRepositoryImpl {}

    #[async_trait::async_trait]
    impl ValuationLayerRepository for ValuationLayerRepositoryImpl {
        async fn find_active_by_product_id(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
        ) -> Result<Vec<ValuationLayer>>;

        async fn create(&self, layer: &ValuationLayer) -> Result<ValuationLayer>;

        async fn consume_layers(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            quantity_to_consume: i64,
        ) -> Result<i64>;

        async fn get_total_quantity(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;

        async fn cleanup_empty_layers(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;
    }
}

// Mock the ValuationHistoryRepository trait
mock! {
    pub ValuationHistoryRepositoryImpl {}

    #[async_trait::async_trait]
    impl ValuationHistoryRepository for ValuationHistoryRepositoryImpl {
        async fn find_by_product_id(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            limit: Option<i64>,
            offset: Option<i64>,
        ) -> Result<Vec<ValuationHistory>>;

        async fn create(&self, history: &ValuationHistory) -> Result<ValuationHistory>;

        async fn count_by_product_id(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test Valuation using the actual struct fields
    fn create_test_valuation(tenant_id: Uuid, product_id: Uuid) -> Valuation {
        Valuation {
            valuation_id: Uuid::new_v4(),
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
            current_unit_cost: Some(1000), // $10.00 in cents
            total_quantity: 100,
            total_value: 100000, // $1000.00 in cents
            standard_cost: Some(1000),
            last_updated: Utc::now(),
            updated_by: None,
        }
    }

    /// Helper function to create a test ValuationLayer using actual struct fields
    fn create_test_layer(
        tenant_id: Uuid,
        product_id: Uuid,
        quantity: i64,
        unit_cost: i64,
    ) -> ValuationLayer {
        ValuationLayer {
            layer_id: Uuid::new_v4(),
            tenant_id,
            product_id,
            quantity,
            unit_cost,
            total_value: quantity * unit_cost,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Helper function to create a test ValuationHistory using actual struct fields
    fn create_test_history(tenant_id: Uuid, product_id: Uuid) -> ValuationHistory {
        ValuationHistory {
            history_id: Uuid::new_v4(),
            valuation_id: Uuid::new_v4(),
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
            unit_cost: Some(1000),
            total_quantity: 100,
            total_value: 100000,
            standard_cost: Some(1000),
            changed_at: Utc::now(),
            changed_by: None,
            change_reason: Some("Receipt".to_string()),
        }
    }

    // =========================================================================
    // ValuationRepository Tests
    // =========================================================================

    #[tokio::test]
    async fn test_find_valuation_by_product_success() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let valuation = create_test_valuation(tenant_id, product_id);

        mock_repo
            .expect_find_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(move |_, _| Ok(Some(valuation.clone())));

        let result = mock_repo.find_by_product_id(tenant_id, product_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().current_unit_cost, Some(1000));
    }

    #[tokio::test]
    async fn test_find_valuation_not_found() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(|_, _| Ok(None));

        let result = mock_repo.find_by_product_id(tenant_id, product_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // =========================================================================
    // set_valuation_method Tests
    // =========================================================================

    #[tokio::test]
    async fn test_set_valuation_method_fifo() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let user_id = Some(Uuid::new_v4());

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.valuation_method = ValuationMethod::Fifo;

        mock_repo
            .expect_set_valuation_method()
            .with(eq(tenant_id), eq(product_id), eq(ValuationMethod::Fifo), eq(user_id))
            .returning(move |_, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .set_valuation_method(tenant_id, product_id, ValuationMethod::Fifo, user_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().valuation_method, ValuationMethod::Fifo);
    }

    #[tokio::test]
    async fn test_set_valuation_method_avco() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.valuation_method = ValuationMethod::Avco;

        mock_repo
            .expect_set_valuation_method()
            .with(eq(tenant_id), eq(product_id), eq(ValuationMethod::Avco), eq(None))
            .returning(move |_, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .set_valuation_method(tenant_id, product_id, ValuationMethod::Avco, None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().valuation_method, ValuationMethod::Avco);
    }

    #[tokio::test]
    async fn test_set_valuation_method_standard() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.valuation_method = ValuationMethod::Standard;

        mock_repo
            .expect_set_valuation_method()
            .with(eq(tenant_id), eq(product_id), eq(ValuationMethod::Standard), eq(None))
            .returning(move |_, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .set_valuation_method(tenant_id, product_id, ValuationMethod::Standard, None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().valuation_method, ValuationMethod::Standard);
    }

    // =========================================================================
    // set_standard_cost Tests
    // =========================================================================

    #[tokio::test]
    async fn test_set_standard_cost_success() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let new_standard_cost = 1500i64; // $15.00

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.standard_cost = Some(new_standard_cost);

        mock_repo
            .expect_set_standard_cost()
            .with(eq(tenant_id), eq(product_id), eq(new_standard_cost), eq(None))
            .returning(move |_, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .set_standard_cost(tenant_id, product_id, new_standard_cost, None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().standard_cost, Some(1500));
    }

    #[tokio::test]
    async fn test_set_standard_cost_negative_value() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let negative_cost = -100i64;

        mock_repo
            .expect_set_standard_cost()
            .with(eq(tenant_id), eq(product_id), eq(negative_cost), eq(None))
            .returning(|_, _, _, _| {
                Err(AppError::ValidationError("Standard cost cannot be negative".to_string()))
            });

        let result = mock_repo
            .set_standard_cost(tenant_id, product_id, negative_cost, None)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationError(_)));
    }

    // =========================================================================
    // ValuationLayerRepository Tests (FIFO)
    // =========================================================================

    #[tokio::test]
    async fn test_get_valuation_layers_success() {
        let mut mock_layer_repo = MockValuationLayerRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let layer1 = create_test_layer(tenant_id, product_id, 50, 900);
        let layer2 = create_test_layer(tenant_id, product_id, 30, 1000);
        let layer3 = create_test_layer(tenant_id, product_id, 20, 1100);
        let layers = vec![layer1, layer2, layer3];

        mock_layer_repo
            .expect_find_active_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(move |_, _| Ok(layers.clone()));

        let result = mock_layer_repo
            .find_active_by_product_id(tenant_id, product_id)
            .await;
        assert!(result.is_ok());
        let found_layers = result.unwrap();
        assert_eq!(found_layers.len(), 3);
        // Verify layer costs
        assert_eq!(found_layers[0].unit_cost, 900);
        assert_eq!(found_layers[1].unit_cost, 1000);
        assert_eq!(found_layers[2].unit_cost, 1100);
    }

    #[tokio::test]
    async fn test_get_valuation_layers_empty() {
        let mut mock_layer_repo = MockValuationLayerRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_layer_repo
            .expect_find_active_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(|_, _| Ok(vec![]));

        let result = mock_layer_repo
            .find_active_by_product_id(tenant_id, product_id)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_fifo_consume_layers_success() {
        let mut mock_layer_repo = MockValuationLayerRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity_to_consume = 30i64;

        // Consume 30 units, expecting cost of $280 (28000 cents)
        let expected_cost = 28000i64;

        mock_layer_repo
            .expect_consume_layers()
            .with(eq(tenant_id), eq(product_id), eq(quantity_to_consume))
            .returning(move |_, _, _| Ok(expected_cost));

        let result = mock_layer_repo
            .consume_layers(tenant_id, product_id, quantity_to_consume)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 28000);
    }

    #[tokio::test]
    async fn test_fifo_get_total_quantity() {
        let mut mock_layer_repo = MockValuationLayerRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_layer_repo
            .expect_get_total_quantity()
            .with(eq(tenant_id), eq(product_id))
            .returning(|_, _| Ok(100));

        let result = mock_layer_repo
            .get_total_quantity(tenant_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_cleanup_empty_layers() {
        let mut mock_layer_repo = MockValuationLayerRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_layer_repo
            .expect_cleanup_empty_layers()
            .with(eq(tenant_id), eq(product_id))
            .returning(|_, _| Ok(3));

        let result = mock_layer_repo
            .cleanup_empty_layers(tenant_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    // =========================================================================
    // ValuationHistoryRepository Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_valuation_history_success() {
        let mut mock_history_repo = MockValuationHistoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let history1 = create_test_history(tenant_id, product_id);
        let history2 = create_test_history(tenant_id, product_id);
        let history = vec![history1, history2];

        mock_history_repo
            .expect_find_by_product_id()
            .with(eq(tenant_id), eq(product_id), eq(Some(10i64)), eq(Some(0i64)))
            .returning(move |_, _, _, _| Ok(history.clone()));

        let result = mock_history_repo
            .find_by_product_id(tenant_id, product_id, Some(10), Some(0))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_valuation_history_pagination() {
        let mut mock_history_repo = MockValuationHistoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let history = vec![create_test_history(tenant_id, product_id)];

        mock_history_repo
            .expect_find_by_product_id()
            .with(eq(tenant_id), eq(product_id), eq(Some(5i64)), eq(Some(10i64)))
            .returning(move |_, _, _, _| Ok(history.clone()));

        let result = mock_history_repo
            .find_by_product_id(tenant_id, product_id, Some(5), Some(10))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_valuation_history_count() {
        let mut mock_history_repo = MockValuationHistoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_history_repo
            .expect_count_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(|_, _| Ok(150));

        let result = mock_history_repo
            .count_by_product_id(tenant_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 150);
    }

    // =========================================================================
    // adjust_cost Tests
    // =========================================================================

    #[tokio::test]
    async fn test_adjust_cost_increase() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let adjustment = 500i64;

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.current_unit_cost = Some(1500);

        mock_repo
            .expect_adjust_cost()
            .with(
                eq(tenant_id),
                eq(product_id),
                eq(adjustment),
                eq("Market price adjustment"),
                eq(None),
            )
            .returning(move |_, _, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .adjust_cost(tenant_id, product_id, adjustment, "Market price adjustment", None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().current_unit_cost, Some(1500));
    }

    #[tokio::test]
    async fn test_adjust_cost_decrease() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let adjustment = -200i64;

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.current_unit_cost = Some(800);

        mock_repo
            .expect_adjust_cost()
            .with(
                eq(tenant_id),
                eq(product_id),
                eq(adjustment),
                eq("Write-down for damaged goods"),
                eq(None),
            )
            .returning(move |_, _, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .adjust_cost(tenant_id, product_id, adjustment, "Write-down for damaged goods", None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().current_unit_cost, Some(800));
    }

    // =========================================================================
    // revalue_inventory Tests
    // =========================================================================

    #[tokio::test]
    async fn test_revalue_inventory_success() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let new_cost = 1200i64;

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.current_unit_cost = Some(new_cost);
        valuation.total_value = new_cost * 100;

        mock_repo
            .expect_revalue_inventory()
            .with(eq(tenant_id), eq(product_id), eq(new_cost), eq("Annual revaluation"), eq(None))
            .returning(move |_, _, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .revalue_inventory(tenant_id, product_id, new_cost, "Annual revaluation", None)
            .await;
        assert!(result.is_ok());
        let result_val = result.unwrap();
        assert_eq!(result_val.current_unit_cost, Some(1200));
        assert_eq!(result_val.total_value, 120000);
    }

    #[tokio::test]
    async fn test_revalue_inventory_product_not_found() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let new_cost = 1200i64;

        mock_repo
            .expect_revalue_inventory()
            .with(eq(tenant_id), eq(product_id), eq(new_cost), eq("Revaluation"), eq(None))
            .returning(|_, _, _, _, _| {
                Err(AppError::NotFound("Product valuation not found".to_string()))
            });

        let result = mock_repo
            .revalue_inventory(tenant_id, product_id, new_cost, "Revaluation", None)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // update_from_stock_move Tests
    // =========================================================================

    #[tokio::test]
    async fn test_update_from_stock_move_receipt() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity_change = 50i64;
        let unit_cost = Some(1100i64);

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.total_quantity = 150;

        mock_repo
            .expect_update_from_stock_move()
            .with(eq(tenant_id), eq(product_id), eq(quantity_change), eq(unit_cost), eq(None))
            .returning(move |_, _, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .update_from_stock_move(tenant_id, product_id, quantity_change, unit_cost, None)
            .await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.total_quantity, 150);
    }

    #[tokio::test]
    async fn test_update_from_stock_move_delivery() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity_change = -20i64;

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.total_quantity = 80;
        valuation.total_value = 80000;

        mock_repo
            .expect_update_from_stock_move()
            .with(eq(tenant_id), eq(product_id), eq(quantity_change), eq(None), eq(None))
            .returning(move |_, _, _, _, _| Ok(valuation.clone()));

        let result = mock_repo
            .update_from_stock_move(tenant_id, product_id, quantity_change, None, None)
            .await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.total_quantity, 80);
    }

    // =========================================================================
    // Multi-tenant Isolation Tests
    // =========================================================================

    #[tokio::test]
    async fn test_valuation_tenant_isolation() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let valuation_a = create_test_valuation(tenant_a, product_id);
        let mut valuation_b = create_test_valuation(tenant_b, product_id);
        valuation_b.current_unit_cost = Some(2000);

        mock_repo
            .expect_find_by_product_id()
            .with(eq(tenant_a), eq(product_id))
            .returning(move |_, _| Ok(Some(valuation_a.clone())));

        mock_repo
            .expect_find_by_product_id()
            .with(eq(tenant_b), eq(product_id))
            .returning(move |_, _| Ok(Some(valuation_b.clone())));

        let result_a = mock_repo.find_by_product_id(tenant_a, product_id).await;
        let result_b = mock_repo.find_by_product_id(tenant_b, product_id).await;

        assert!(result_a.is_ok());
        assert!(result_b.is_ok());

        let val_a = result_a.unwrap().unwrap();
        let val_b = result_b.unwrap().unwrap();

        assert_eq!(val_a.current_unit_cost, Some(1000));
        assert_eq!(val_b.current_unit_cost, Some(2000));
        assert_eq!(val_a.tenant_id, tenant_a);
        assert_eq!(val_b.tenant_id, tenant_b);
    }

    // =========================================================================
    // Edge Cases Tests
    // =========================================================================

    #[tokio::test]
    async fn test_valuation_zero_quantity() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.total_quantity = 0;
        valuation.total_value = 0;

        mock_repo
            .expect_find_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(move |_, _| Ok(Some(valuation.clone())));

        let result = mock_repo.find_by_product_id(tenant_id, product_id).await;
        assert!(result.is_ok());
        let val = result.unwrap().unwrap();
        assert_eq!(val.total_quantity, 0);
        assert_eq!(val.total_value, 0);
    }

    #[tokio::test]
    async fn test_valuation_large_values() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.total_quantity = 1_000_000;
        valuation.current_unit_cost = Some(99_999_999);
        valuation.total_value = 99_999_999_000_000;

        mock_repo
            .expect_find_by_product_id()
            .with(eq(tenant_id), eq(product_id))
            .returning(move |_, _| Ok(Some(valuation.clone())));

        let result = mock_repo.find_by_product_id(tenant_id, product_id).await;
        assert!(result.is_ok());
        let val = result.unwrap().unwrap();
        assert_eq!(val.total_quantity, 1_000_000);
    }

    #[tokio::test]
    async fn test_create_valuation_layer() {
        let mut mock_layer_repo = MockValuationLayerRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let layer = create_test_layer(tenant_id, product_id, 100, 1000);

        mock_layer_repo
            .expect_create()
            .returning(move |l| Ok(l.clone()));

        let result = mock_layer_repo.create(&layer).await;
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.quantity, 100);
        assert_eq!(created.unit_cost, 1000);
    }

    #[tokio::test]
    async fn test_create_valuation_history() {
        let mut mock_history_repo = MockValuationHistoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let history = create_test_history(tenant_id, product_id);

        mock_history_repo
            .expect_create()
            .returning(move |h| Ok(h.clone()));

        let result = mock_history_repo.create(&history).await;
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.valuation_method, ValuationMethod::Avco);
        assert_eq!(created.total_quantity, 100);
    }

    #[tokio::test]
    async fn test_create_valuation() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let valuation = create_test_valuation(tenant_id, product_id);

        mock_repo.expect_create().returning(move |v| Ok(v.clone()));

        let result = mock_repo.create(&valuation).await;
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.valuation_method, ValuationMethod::Avco);
        assert_eq!(created.current_unit_cost, Some(1000));
    }

    #[tokio::test]
    async fn test_update_valuation() {
        let mut mock_repo = MockValuationRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut valuation = create_test_valuation(tenant_id, product_id);
        valuation.current_unit_cost = Some(1500);
        let valuation_clone = valuation.clone();

        mock_repo
            .expect_update()
            .with(eq(tenant_id), eq(product_id), always())
            .returning(move |_, _, _| Ok(valuation_clone.clone()));

        let result = mock_repo.update(tenant_id, product_id, &valuation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().current_unit_cost, Some(1500));
    }
}
