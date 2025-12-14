//! Advanced Business Logic Tests for Valuation
//!
//! Integration tests for FIFO, AVCO, and Standard costing methods.
//! Covers cost layer management, average cost recalculation, and edge cases.

mod business_logic_test_helpers;

use business_logic_test_helpers::{
    cleanup_valuation_test_data, create_valuation_service, setup_test_pool,
    setup_test_tenant_and_product,
};
use inventory_service_core::domains::inventory::valuation::ValuationMethod;
use inventory_service_core::services::valuation::ValuationService;
use uuid::Uuid;

// ============================================================================
// FIFO Valuation Tests
// ============================================================================

#[cfg(test)]
mod fifo_valuation_tests {
    use super::*;
    use inventory_service_core::domains::inventory::dto::valuation_dto::SetValuationMethodRequest;

    #[tokio::test]
    async fn test_fifo_receipt_creates_cost_layer() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        // Set valuation method to FIFO
        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Fifo,
        };
        let result = service.set_valuation_method(set_method_request).await;
        assert!(result.is_ok(), "Should set FIFO method: {:?}", result.err());

        // Process a receipt (positive quantity change)
        let receipt_result = service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await;

        assert!(receipt_result.is_ok(), "Receipt should succeed: {:?}", receipt_result.err());
        let valuation = receipt_result.unwrap();

        // Verify valuation updated correctly
        assert_eq!(valuation.total_quantity, 100);
        assert_eq!(valuation.total_value, 100_000);

        // Verify cost layer was created
        let layers_request =
            inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationLayersRequest {
                tenant_id,
                product_id,
            };
        let layers = service.get_valuation_layers(layers_request).await;
        assert!(layers.is_ok());
        let layer_response = layers.unwrap();
        assert!(!layer_response.layers.is_empty(), "Should have created a cost layer");
        assert_eq!(layer_response.layers[0].quantity, 100);
        assert_eq!(layer_response.layers[0].unit_cost, 1000);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_fifo_multiple_receipts_different_costs() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Fifo,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // First receipt: 50 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 50, Some(1000), None)
            .await
            .expect("First receipt should succeed");

        // Second receipt: 30 units at $12.00
        service
            .process_stock_movement(tenant_id, product_id, 30, Some(1200), None)
            .await
            .expect("Second receipt should succeed");

        // Third receipt: 20 units at $15.00
        let final_valuation = service
            .process_stock_movement(tenant_id, product_id, 20, Some(1500), None)
            .await
            .expect("Third receipt should succeed");

        // Total: 50 + 30 + 20 = 100 units
        // Total value: (50*1000) + (30*1200) + (20*1500) = 116000
        assert_eq!(final_valuation.total_quantity, 100);
        assert_eq!(final_valuation.total_value, 116_000);

        // Verify 3 cost layers exist
        let layers_request =
            inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationLayersRequest {
                tenant_id,
                product_id,
            };
        let layers = service.get_valuation_layers(layers_request).await.unwrap();
        assert_eq!(layers.layers.len(), 3, "Should have 3 cost layers");

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_fifo_delivery_consumes_oldest_layer_first() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Fifo,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt 1: 50 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 50, Some(1000), None)
            .await
            .expect("First receipt should succeed");

        // Receipt 2: 50 units at $20.00
        service
            .process_stock_movement(tenant_id, product_id, 50, Some(2000), None)
            .await
            .expect("Second receipt should succeed");

        // Delivery of 60 units (should consume all of layer 1 and 10 from layer 2)
        let after_delivery = service
            .process_stock_movement(tenant_id, product_id, -60, None, None)
            .await
            .expect("Delivery should succeed");

        // Remaining: 40 units at $20.00 each = 80000
        assert_eq!(after_delivery.total_quantity, 40);
        assert_eq!(after_delivery.total_value, 80_000);

        // Verify layers - service returns only active layers
        let layers_request =
            inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationLayersRequest {
                tenant_id,
                product_id,
            };
        let layers = service.get_valuation_layers(layers_request).await.unwrap();

        // get_valuation_layers returns only active layers (quantity > 0)
        let active_layers = &layers.layers;
        assert_eq!(active_layers.len(), 1, "Should have 1 active layer remaining");
        assert_eq!(active_layers[0].quantity, 40);
        assert_eq!(active_layers[0].unit_cost, 2000);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_fifo_delivery_exceeding_available_quantity_fails() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        // Set valuation method to FIFO
        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Fifo,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .expect("Failed to set FIFO method");

        // Receipt: 100 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt should succeed");

        // Attempt to deliver 120 units when only 100 are available
        let delivery_result = service
            .process_stock_movement(tenant_id, product_id, -120, None, None)
            .await;

        // Should fail with an error (insufficient stock)
        assert!(delivery_result.is_err(), "Delivery exceeding available quantity should fail");

        // Verify the original quantity is unchanged
        let valuation = service
            .get_valuation(
                inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationRequest {
                    tenant_id,
                    product_id,
                },
            )
            .await
            .expect("Should get valuation");

        assert_eq!(
            valuation.total_quantity, 100,
            "Quantity should remain unchanged after failed delivery"
        );

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_fifo_partial_layer_consumption() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Fifo,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt: 100 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt should succeed");

        // Delivery of 30 units (partial consumption)
        let after_delivery = service
            .process_stock_movement(tenant_id, product_id, -30, None, None)
            .await
            .expect("Delivery should succeed");

        // Remaining: 70 units at $10.00 = 70000
        assert_eq!(after_delivery.total_quantity, 70);
        assert_eq!(after_delivery.total_value, 70_000);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }
}

// ============================================================================
// AVCO (Average Cost) Valuation Tests
// ============================================================================

#[cfg(test)]
mod avco_valuation_tests {
    use super::*;
    use inventory_service_core::domains::inventory::dto::valuation_dto::SetValuationMethodRequest;

    #[tokio::test]
    async fn test_avco_receipt_recalculates_average() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // First receipt: 100 units at $10.00
        let first_receipt = service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("First receipt should succeed");

        assert_eq!(first_receipt.total_quantity, 100);
        assert_eq!(first_receipt.current_unit_cost, Some(1000));

        // Second receipt: 100 units at $20.00
        // New average = (100*1000 + 100*2000) / 200 = 1500
        let second_receipt = service
            .process_stock_movement(tenant_id, product_id, 100, Some(2000), None)
            .await
            .expect("Second receipt should succeed");

        assert_eq!(second_receipt.total_quantity, 200);
        assert_eq!(second_receipt.current_unit_cost, Some(1500));
        assert_eq!(second_receipt.total_value, 300_000);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_avco_multiple_receipts_weighted_average() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt 1: 50 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 50, Some(1000), None)
            .await
            .expect("First receipt");

        // Receipt 2: 100 units at $15.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1500), None)
            .await
            .expect("Second receipt");

        // Receipt 3: 50 units at $20.00
        let final_valuation = service
            .process_stock_movement(tenant_id, product_id, 50, Some(2000), None)
            .await
            .expect("Third receipt");

        // Total: 200 units, value: 300000, average: 1500
        assert_eq!(final_valuation.total_quantity, 200);
        assert_eq!(final_valuation.total_value, 300_000);
        assert_eq!(final_valuation.current_unit_cost, Some(1500));

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_avco_delivery_uses_average_cost() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt: 100 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt should succeed");

        // Delivery of 40 units
        let after_delivery = service
            .process_stock_movement(tenant_id, product_id, -40, None, None)
            .await
            .expect("Delivery should succeed");

        // Remaining: 60 units at $10.00 average = 60000
        assert_eq!(after_delivery.total_quantity, 60);
        assert_eq!(after_delivery.total_value, 60_000);
        assert_eq!(after_delivery.current_unit_cost, Some(1000));

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }
}

// ============================================================================
// Standard Cost Valuation Tests
// ============================================================================

#[cfg(test)]
mod standard_cost_tests {
    use super::*;
    use inventory_service_core::domains::inventory::dto::valuation_dto::{
        SetStandardCostRequest, SetValuationMethodRequest,
    };

    #[tokio::test]
    async fn test_standard_cost_update_recalculates_value() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Standard,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Set standard cost to $10.00
        let set_cost_request = SetStandardCostRequest {
            tenant_id,
            product_id,
            standard_cost: 1000,
        };
        service.set_standard_cost(set_cost_request).await.unwrap();

        // Receipt: 100 units (uses standard cost)
        let after_receipt = service
            .process_stock_movement(tenant_id, product_id, 100, None, None)
            .await
            .expect("Receipt should succeed");

        assert_eq!(after_receipt.total_quantity, 100);
        assert_eq!(after_receipt.total_value, 100_000);

        // Update standard cost to $15.00
        let update_cost_request = SetStandardCostRequest {
            tenant_id,
            product_id,
            standard_cost: 1500,
        };
        let updated = service
            .set_standard_cost(update_cost_request)
            .await
            .unwrap();

        // Total value should be recalculated: 100 * 1500 = 150000
        assert_eq!(updated.total_value, 150_000);
        assert_eq!(updated.standard_cost, Some(1500));

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

#[cfg(test)]
mod valuation_edge_cases {
    use super::*;
    use inventory_service_core::domains::inventory::dto::valuation_dto::{
        CostAdjustmentRequest, GetValuationRequest, RevaluationRequest, SetValuationMethodRequest,
    };

    #[tokio::test]
    async fn test_valuation_zero_quantity_handling() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt then delivery to zero
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt");

        let zero_qty = service
            .process_stock_movement(tenant_id, product_id, -100, None, None)
            .await
            .expect("Complete delivery");

        assert_eq!(zero_qty.total_quantity, 0);
        assert_eq!(zero_qty.total_value, 0);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_cost_adjustment_updates_total_value() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt: 100 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt");

        // Cost adjustment: add $5000 (variance correction)
        let adjust_request = CostAdjustmentRequest {
            tenant_id,
            product_id,
            adjustment_amount: 5000,
            reason: "Variance correction".to_string(),
        };
        let adjusted = service
            .adjust_cost(adjust_request)
            .await
            .expect("Adjustment should succeed");

        // Total value should be 100000 + 5000 = 105000
        assert_eq!(adjusted.total_value, 105_000);
        assert_eq!(adjusted.total_quantity, 100);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_revaluation_applies_new_cost() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt: 100 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt");

        // Revalue to $12.00 per unit
        let reval_request = RevaluationRequest {
            tenant_id,
            product_id,
            new_unit_cost: 1200,
            reason: "Market price adjustment".to_string(),
        };
        let revalued = service
            .revalue_inventory(reval_request)
            .await
            .expect("Revaluation should succeed");

        // Total value should be 100 * 1200 = 120000
        assert_eq!(revalued.total_value, 120_000);
        assert_eq!(revalued.current_unit_cost, Some(1200));

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }

    #[tokio::test]
    async fn test_valuation_not_found_error() {
        let pool = setup_test_pool().await;
        let service = create_valuation_service(&pool);

        // Try to get valuation for non-existent product
        let request = GetValuationRequest {
            tenant_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
        };

        let result = service.get_valuation(request).await;
        assert!(result.is_err(), "Should return error for non-existent valuation");
    }

    #[tokio::test]
    async fn test_calculate_inventory_value() {
        let pool = setup_test_pool().await;
        let (tenant_id, product_id) = setup_test_tenant_and_product(&pool).await;
        let service = create_valuation_service(&pool);

        let set_method_request = SetValuationMethodRequest {
            tenant_id,
            product_id,
            valuation_method: ValuationMethod::Avco,
        };
        service
            .set_valuation_method(set_method_request)
            .await
            .unwrap();

        // Receipt: 100 units at $10.00
        service
            .process_stock_movement(tenant_id, product_id, 100, Some(1000), None)
            .await
            .expect("Receipt");

        // Calculate inventory value
        let value = service
            .calculate_inventory_value(tenant_id, product_id)
            .await
            .expect("Should calculate value");

        assert_eq!(value, 100_000);

        cleanup_valuation_test_data(&pool, tenant_id).await;
    }
}
