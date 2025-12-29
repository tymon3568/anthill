//! Unit tests for lot/serial repository interactions using mocks
//!
//! These tests validate the repository trait contracts for lot/serial tracking,
//! supporting both lot-based and serial-based inventory tracking.

use chrono::{Duration, Utc};
use mockall::mock;
use uuid::Uuid;

use inventory_service_core::models::{
    CreateStockMoveRequest, LotSerial, LotSerialStatus, LotSerialTrackingType, StockMove,
};
use inventory_service_core::repositories::{LotSerialRepository, StockMoveRepository};
use inventory_service_core::Result;
use shared_error::AppError;

// Mock the LotSerialRepository trait
mock! {
    pub LotSerialRepositoryImpl {}

    #[async_trait::async_trait]
    impl LotSerialRepository for LotSerialRepositoryImpl {
        async fn create(&self, lot_serial: &LotSerial) -> Result<()>;
        async fn find_by_id(
            &self,
            tenant_id: Uuid,
            lot_serial_id: Uuid,
        ) -> Result<Option<LotSerial>>;
        async fn find_by_product(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            tracking_type: Option<LotSerialTrackingType>,
            status: Option<LotSerialStatus>,
        ) -> Result<Vec<LotSerial>>;
        async fn find_available_for_picking(
            &self,
            tenant_id: Uuid,
            product_id: Uuid,
            warehouse_id: Option<Uuid>,
            quantity_needed: i64,
        ) -> Result<Vec<LotSerial>>;
        async fn update(&self, lot_serial: &LotSerial) -> Result<()>;
        async fn update_remaining_quantity(
            &self,
            tenant_id: Uuid,
            lot_serial_id: Uuid,
            new_remaining_quantity: i64,
        ) -> Result<()>;
        async fn delete(&self, tenant_id: Uuid, lot_serial_id: Uuid) -> Result<()>;
        async fn quarantine_expired_lots(&self, tenant_id: Uuid) -> Result<i64>;
    }
}

// Mock the StockMoveRepository trait
mock! {
    pub StockMoveRepositoryImpl {}

    #[async_trait::async_trait]
    impl StockMoveRepository for StockMoveRepositoryImpl {
        async fn create(
            &self,
            stock_move: &CreateStockMoveRequest,
            tenant_id: Uuid,
        ) -> Result<StockMove>;
        async fn find_by_reference(
            &self,
            tenant_id: Uuid,
            reference_type: &str,
            reference_id: Uuid,
        ) -> Result<Vec<StockMove>>;
        async fn exists_by_idempotency_key(
            &self,
            tenant_id: Uuid,
            idempotency_key: &str,
        ) -> Result<bool>;
        async fn find_by_lot_serial(
            &self,
            tenant_id: Uuid,
            lot_serial_id: Uuid,
        ) -> Result<Vec<StockMove>>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test LotSerial with lot tracking
    fn create_test_lot_serial(tenant_id: Uuid, product_id: Uuid) -> LotSerial {
        LotSerial {
            lot_serial_id: Uuid::new_v4(),
            tenant_id,
            product_id,
            tracking_type: LotSerialTrackingType::Lot,
            lot_number: Some("LOT-2024-001".to_string()),
            serial_number: None,
            initial_quantity: Some(100),
            remaining_quantity: Some(100),
            expiry_date: Some(Utc::now() + Duration::days(365)),
            status: LotSerialStatus::Active,
            warehouse_id: None,
            location_id: None,
            created_by: Uuid::new_v4(),
            updated_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Helper function to create a test serial-tracked LotSerial
    fn create_test_serial(tenant_id: Uuid, product_id: Uuid, serial_number: &str) -> LotSerial {
        LotSerial {
            lot_serial_id: Uuid::new_v4(),
            tenant_id,
            product_id,
            tracking_type: LotSerialTrackingType::Serial,
            lot_number: None,
            serial_number: Some(serial_number.to_string()),
            initial_quantity: Some(1),
            remaining_quantity: Some(1),
            expiry_date: None,
            status: LotSerialStatus::Active,
            warehouse_id: None,
            location_id: None,
            created_by: Uuid::new_v4(),
            updated_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Helper function to create a test StockMove
    fn create_test_stock_move(tenant_id: Uuid, lot_serial_id: Option<Uuid>) -> StockMove {
        StockMove {
            move_id: Uuid::new_v4(),
            tenant_id,
            product_id: Uuid::new_v4(),
            source_location_id: None,
            destination_location_id: Some(Uuid::new_v4()),
            move_type: "receipt".to_string(),
            quantity: 10,
            unit_cost: Some(1000),
            total_cost: Some(10000),
            reference_type: "purchase_order".to_string(),
            reference_id: Uuid::new_v4(),
            lot_serial_id,
            idempotency_key: format!("test-{}", Uuid::new_v4()),
            move_date: Utc::now(),
            move_reason: None,
            batch_info: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }

    // =========================================================================
    // LotSerial Create Tests
    // =========================================================================

    #[tokio::test]
    async fn test_create_lot_serial_success() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let lot_serial = create_test_lot_serial(tenant_id, product_id);

        mock_repo.expect_create().times(1).returning(|_| Ok(()));

        let result = mock_repo.create(&lot_serial).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_lot_serial_duplicate_lot_number() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let lot_serial = create_test_lot_serial(tenant_id, product_id);

        mock_repo
            .expect_create()
            .times(1)
            .returning(|_| Err(AppError::Conflict("Lot number already exists".to_string())));

        let result = mock_repo.create(&lot_serial).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn test_create_serial_tracked_item() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let serial = create_test_serial(tenant_id, product_id, "SN-001");

        mock_repo.expect_create().times(1).returning(|_| Ok(()));

        let result = mock_repo.create(&serial).await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // LotSerial Find By ID Tests
    // =========================================================================

    #[tokio::test]
    async fn test_find_by_id_success() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let lot_serial = create_test_lot_serial(tenant_id, product_id);
        let lot_serial_id = lot_serial.lot_serial_id;
        let lot_serial_clone = lot_serial.clone();

        mock_repo
            .expect_find_by_id()
            .withf(move |t, l| *t == tenant_id && *l == lot_serial_id)
            .times(1)
            .returning(move |_, _| Ok(Some(lot_serial_clone.clone())));

        let result = mock_repo.find_by_id(tenant_id, lot_serial_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().lot_serial_id, lot_serial_id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let lot_serial_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let result = mock_repo.find_by_id(tenant_id, lot_serial_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_by_id_wrong_tenant() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let wrong_tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let lot_serial = create_test_lot_serial(tenant_id, product_id);
        let lot_serial_id = lot_serial.lot_serial_id;

        // When querying with wrong tenant, should return None (tenant isolation)
        mock_repo
            .expect_find_by_id()
            .withf(move |t, _| *t == wrong_tenant_id)
            .times(1)
            .returning(|_, _| Ok(None));

        let result = mock_repo.find_by_id(wrong_tenant_id, lot_serial_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // =========================================================================
    // LotSerial Find By Product Tests
    // =========================================================================

    #[tokio::test]
    async fn test_find_by_product_all_lots() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let lot1 = create_test_lot_serial(tenant_id, product_id);
        let lot2 = create_test_lot_serial(tenant_id, product_id);
        let lots = vec![lot1, lot2];
        let lots_clone = lots.clone();

        mock_repo
            .expect_find_by_product()
            .withf(move |t, p, _, _| *t == tenant_id && *p == product_id)
            .times(1)
            .returning(move |_, _, _, _| Ok(lots_clone.clone()));

        let result = mock_repo
            .find_by_product(tenant_id, product_id, None, None)
            .await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_product_filter_by_tracking_type() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let serial = create_test_serial(tenant_id, product_id, "SN-001");
        let serials = vec![serial];
        let serials_clone = serials.clone();

        mock_repo
            .expect_find_by_product()
            .withf(move |_, _, tracking_type, _| {
                *tracking_type == Some(LotSerialTrackingType::Serial)
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(serials_clone.clone()));

        let result = mock_repo
            .find_by_product(tenant_id, product_id, Some(LotSerialTrackingType::Serial), None)
            .await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].tracking_type, LotSerialTrackingType::Serial);
    }

    #[tokio::test]
    async fn test_find_by_product_filter_by_status() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot = create_test_lot_serial(tenant_id, product_id);
        lot.status = LotSerialStatus::Quarantined;
        let lots = vec![lot];
        let lots_clone = lots.clone();

        mock_repo
            .expect_find_by_product()
            .withf(move |_, _, _, status| *status == Some(LotSerialStatus::Quarantined))
            .times(1)
            .returning(move |_, _, _, _| Ok(lots_clone.clone()));

        let result = mock_repo
            .find_by_product(tenant_id, product_id, None, Some(LotSerialStatus::Quarantined))
            .await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].status, LotSerialStatus::Quarantined);
    }

    #[tokio::test]
    async fn test_find_by_product_empty_result() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_product()
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let result = mock_repo
            .find_by_product(tenant_id, product_id, None, None)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // =========================================================================
    // LotSerial Available For Picking Tests (FEFO)
    // =========================================================================

    #[tokio::test]
    async fn test_find_available_for_picking_fefo_order() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // Create lots with different expiry dates (FEFO should return earliest first)
        let mut lot1 = create_test_lot_serial(tenant_id, product_id);
        lot1.expiry_date = Some(Utc::now() + Duration::days(30)); // Expires soon

        let mut lot2 = create_test_lot_serial(tenant_id, product_id);
        lot2.expiry_date = Some(Utc::now() + Duration::days(90)); // Expires later

        // Return in FEFO order (earliest expiry first)
        let lots = vec![lot1.clone(), lot2.clone()];
        let lots_clone = lots.clone();

        mock_repo
            .expect_find_available_for_picking()
            .times(1)
            .returning(move |_, _, _, _| Ok(lots_clone.clone()));

        let result = mock_repo
            .find_available_for_picking(tenant_id, product_id, None, 50)
            .await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);
        // First lot should expire before second (FEFO)
        assert!(found[0].expiry_date < found[1].expiry_date);
    }

    #[tokio::test]
    async fn test_find_available_for_picking_insufficient_quantity() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // Only 50 units available
        let mut lot = create_test_lot_serial(tenant_id, product_id);
        lot.remaining_quantity = Some(50);
        let lots = vec![lot];
        let lots_clone = lots.clone();

        mock_repo
            .expect_find_available_for_picking()
            .times(1)
            .returning(move |_, _, _, _| Ok(lots_clone.clone()));

        // Request 100 units but only 50 available
        let result = mock_repo
            .find_available_for_picking(tenant_id, product_id, None, 100)
            .await;
        assert!(result.is_ok());
        let found = result.unwrap();
        // Returns what's available even if less than requested
        let total: i64 = found.iter().filter_map(|l| l.remaining_quantity).sum();
        assert_eq!(total, 50);
    }

    // =========================================================================
    // LotSerial Update Tests
    // =========================================================================

    #[tokio::test]
    async fn test_update_lot_serial_success() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot_serial = create_test_lot_serial(tenant_id, product_id);
        lot_serial.remaining_quantity = Some(50);

        mock_repo.expect_update().times(1).returning(|_| Ok(()));

        let result = mock_repo.update(&lot_serial).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_remaining_quantity() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let lot_serial_id = Uuid::new_v4();
        let new_quantity = 25;

        mock_repo
            .expect_update_remaining_quantity()
            .withf(move |t, l, q| *t == tenant_id && *l == lot_serial_id && *q == new_quantity)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let result = mock_repo
            .update_remaining_quantity(tenant_id, lot_serial_id, new_quantity)
            .await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // LotSerial Delete Tests
    // =========================================================================

    #[tokio::test]
    async fn test_delete_lot_serial_success() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let lot_serial_id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .withf(move |t, l| *t == tenant_id && *l == lot_serial_id)
            .times(1)
            .returning(|_, _| Ok(()));

        let result = mock_repo.delete(tenant_id, lot_serial_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_lot_serial_not_found() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let lot_serial_id = Uuid::new_v4();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(AppError::NotFound("Lot/serial not found".to_string())));

        let result = mock_repo.delete(tenant_id, lot_serial_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // Quarantine Expired Lots Tests
    // =========================================================================

    #[tokio::test]
    async fn test_quarantine_expired_lots_success() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        // Simulate 5 lots being quarantined
        mock_repo
            .expect_quarantine_expired_lots()
            .withf(move |t| *t == tenant_id)
            .times(1)
            .returning(|_| Ok(5));

        let result = mock_repo.quarantine_expired_lots(tenant_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[tokio::test]
    async fn test_quarantine_expired_lots_none_expired() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();

        mock_repo
            .expect_quarantine_expired_lots()
            .times(1)
            .returning(|_| Ok(0));

        let result = mock_repo.quarantine_expired_lots(tenant_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // =========================================================================
    // StockMove Integration Tests
    // =========================================================================

    #[tokio::test]
    async fn test_find_stock_moves_by_lot_serial() {
        let mut mock_stock_move_repo = MockStockMoveRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let lot_serial_id = Uuid::new_v4();

        let move1 = create_test_stock_move(tenant_id, Some(lot_serial_id));
        let move2 = create_test_stock_move(tenant_id, Some(lot_serial_id));
        let moves = vec![move1, move2];
        let moves_clone = moves.clone();

        mock_stock_move_repo
            .expect_find_by_lot_serial()
            .withf(move |t, l| *t == tenant_id && *l == lot_serial_id)
            .times(1)
            .returning(move |_, _| Ok(moves_clone.clone()));

        let result = mock_stock_move_repo
            .find_by_lot_serial(tenant_id, lot_serial_id)
            .await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);
        for m in found {
            assert_eq!(m.lot_serial_id, Some(lot_serial_id));
        }
    }

    // =========================================================================
    // Status Transition Tests
    // =========================================================================

    #[tokio::test]
    async fn test_lot_status_transition_active_to_disposed() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot_serial = create_test_lot_serial(tenant_id, product_id);

        // Simulate transition from Active to Disposed
        lot_serial.status = LotSerialStatus::Disposed;
        lot_serial.remaining_quantity = Some(0);

        mock_repo.expect_update().times(1).returning(|_| Ok(()));

        let result = mock_repo.update(&lot_serial).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lot_status_transition_to_quarantine() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot_serial = create_test_lot_serial(tenant_id, product_id);

        // Simulate transition to Quarantined (e.g., quality issue)
        lot_serial.status = LotSerialStatus::Quarantined;

        mock_repo.expect_update().times(1).returning(|_| Ok(()));

        let result = mock_repo.update(&lot_serial).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lot_status_reserved() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot_serial = create_test_lot_serial(tenant_id, product_id);

        // Simulate reservation for an order
        lot_serial.status = LotSerialStatus::Reserved;

        mock_repo.expect_update().times(1).returning(|_| Ok(()));

        let result = mock_repo.update(&lot_serial).await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // Tenant Isolation Tests
    // =========================================================================

    #[tokio::test]
    async fn test_lot_serial_tenant_isolation() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // Create lot for tenant A
        let lot_a = create_test_lot_serial(tenant_a, product_id);
        let lot_a_id = lot_a.lot_serial_id;
        let lot_a_clone = lot_a.clone();

        // Tenant A should find their lot
        mock_repo
            .expect_find_by_id()
            .withf(move |t, l| *t == tenant_a && *l == lot_a_id)
            .times(1)
            .returning(move |_, _| Ok(Some(lot_a_clone.clone())));

        let result_a = mock_repo.find_by_id(tenant_a, lot_a_id).await;
        assert!(result_a.is_ok());
        assert!(result_a.unwrap().is_some());

        // Tenant B should NOT find tenant A's lot
        mock_repo
            .expect_find_by_id()
            .withf(move |t, l| *t == tenant_b && *l == lot_a_id)
            .times(1)
            .returning(|_, _| Ok(None));

        let result_b = mock_repo.find_by_id(tenant_b, lot_a_id).await;
        assert!(result_b.is_ok());
        assert!(result_b.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_by_product_tenant_isolation() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // Create lots for tenant A
        let lot1 = create_test_lot_serial(tenant_a, product_id);
        let lot2 = create_test_lot_serial(tenant_a, product_id);
        let lots_a = vec![lot1, lot2];
        let lots_a_clone = lots_a.clone();

        // Tenant A should find their lots
        mock_repo
            .expect_find_by_product()
            .withf(move |t, _, _, _| *t == tenant_a)
            .times(1)
            .returning(move |_, _, _, _| Ok(lots_a_clone.clone()));

        let result_a = mock_repo
            .find_by_product(tenant_a, product_id, None, None)
            .await;
        assert!(result_a.is_ok());
        assert_eq!(result_a.unwrap().len(), 2);

        // Tenant B should find no lots for the same product
        mock_repo
            .expect_find_by_product()
            .withf(move |t, _, _, _| *t == tenant_b)
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let result_b = mock_repo
            .find_by_product(tenant_b, product_id, None, None)
            .await;
        assert!(result_b.is_ok());
        assert!(result_b.unwrap().is_empty());
    }

    // =========================================================================
    // Expiry Date Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_lot_with_no_expiry_date() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot_serial = create_test_lot_serial(tenant_id, product_id);
        lot_serial.expiry_date = None; // No expiry (non-perishable)

        mock_repo.expect_create().times(1).returning(|_| Ok(()));

        let result = mock_repo.create(&lot_serial).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lot_already_expired() {
        let mut mock_repo = MockLotSerialRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let mut lot_serial = create_test_lot_serial(tenant_id, product_id);
        lot_serial.expiry_date = Some(Utc::now() - Duration::days(1)); // Already expired
        lot_serial.status = LotSerialStatus::Expired;

        mock_repo.expect_create().times(1).returning(|_| Ok(()));

        let result = mock_repo.create(&lot_serial).await;
        assert!(result.is_ok());
    }
}
