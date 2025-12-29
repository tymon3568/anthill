//! Unit tests for InventoryServiceImpl using mocks

use mockall::mock;
use mockall::predicate::*;
use uuid::Uuid;

use inventory_service_core::repositories::InventoryRepository;
use inventory_service_core::services::InventoryService;
use inventory_service_core::Result;
use shared_error::AppError;

use super::InventoryServiceImpl;
use std::sync::Arc;

// Mock the InventoryRepository trait
mock! {
    pub InventoryRepositoryImpl {}

    #[async_trait::async_trait]
    impl InventoryRepository for InventoryRepositoryImpl {
        async fn reserve_stock(
            &self,
            tenant_id: Uuid,
            warehouse_id: Uuid,
            product_id: Uuid,
            quantity: i64,
        ) -> Result<()>;

        async fn release_stock(
            &self,
            tenant_id: Uuid,
            warehouse_id: Uuid,
            product_id: Uuid,
            quantity: i64,
        ) -> Result<()>;

        async fn get_available_stock(
            &self,
            tenant_id: Uuid,
            warehouse_id: Uuid,
            product_id: Uuid,
        ) -> Result<i64>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // reserve_stock Tests
    // =========================================================================

    #[tokio::test]
    async fn test_reserve_stock_success() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 10i64;

        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| Ok(()));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .reserve_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reserve_stock_insufficient_stock() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 100i64;

        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| {
                Err(AppError::ValidationError("Insufficient stock available".to_string()))
            });

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .reserve_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_reserve_stock_zero_quantity() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 0i64;

        // Zero quantity reservation should still work (no-op)
        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| Ok(()));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .reserve_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reserve_stock_database_error() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 10i64;

        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| {
                Err(AppError::InternalError("Database connection failed".to_string()))
            });

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .reserve_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InternalError(_)));
    }

    // =========================================================================
    // release_stock Tests
    // =========================================================================

    #[tokio::test]
    async fn test_release_stock_success() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 10i64;

        mock_repo
            .expect_release_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| Ok(()));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .release_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_release_stock_more_than_reserved() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 100i64;

        mock_repo
            .expect_release_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| {
                Err(AppError::ValidationError(
                    "Cannot release more than reserved quantity".to_string(),
                ))
            });

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .release_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_release_stock_zero_quantity() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 0i64;

        mock_repo
            .expect_release_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| Ok(()));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .release_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_release_stock_product_not_found() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 10i64;

        mock_repo
            .expect_release_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .returning(|_, _, _, _| {
                Err(AppError::NotFound("Product not found in warehouse".to_string()))
            });

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .release_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // =========================================================================
    // get_available_stock Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_available_stock_success() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let available_stock = 50i64;

        mock_repo
            .expect_get_available_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id))
            .returning(move |_, _, _| Ok(available_stock));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .get_available_stock(tenant_id, warehouse_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }

    #[tokio::test]
    async fn test_get_available_stock_zero() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_get_available_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id))
            .returning(|_, _, _| Ok(0));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .get_available_stock(tenant_id, warehouse_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_available_stock_negative() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // Negative stock could indicate overselling or data issue
        mock_repo
            .expect_get_available_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id))
            .returning(|_, _, _| Ok(-5));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .get_available_stock(tenant_id, warehouse_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -5);
    }

    #[tokio::test]
    async fn test_get_available_stock_warehouse_not_found() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        mock_repo
            .expect_get_available_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id))
            .returning(|_, _, _| Err(AppError::NotFound("Warehouse not found".to_string())));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .get_available_stock(tenant_id, warehouse_id, product_id)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_available_stock_large_quantity() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let large_stock = 1_000_000_000i64;

        mock_repo
            .expect_get_available_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id))
            .returning(move |_, _, _| Ok(large_stock));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        let result = service
            .get_available_stock(tenant_id, warehouse_id, product_id)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_000_000_000);
    }

    // =========================================================================
    // Multi-tenant Isolation Tests
    // =========================================================================

    #[tokio::test]
    async fn test_reserve_stock_different_tenants() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 10i64;

        // First call for tenant A
        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_a), eq(warehouse_id), eq(product_id), eq(quantity))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        // Second call for tenant B
        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_b), eq(warehouse_id), eq(product_id), eq(quantity))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        // Both tenants should be able to reserve independently
        let result_a = service
            .reserve_stock(tenant_a, warehouse_id, product_id, quantity)
            .await;
        let result_b = service
            .reserve_stock(tenant_b, warehouse_id, product_id, quantity)
            .await;

        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
    }

    // =========================================================================
    // Integration Scenario Tests
    // =========================================================================

    #[tokio::test]
    async fn test_reserve_and_release_flow() {
        let mut mock_repo = MockInventoryRepositoryImpl::new();
        let tenant_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let quantity = 5i64;

        // Setup: expect reserve then release
        mock_repo
            .expect_reserve_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        mock_repo
            .expect_release_stock()
            .with(eq(tenant_id), eq(warehouse_id), eq(product_id), eq(quantity))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = InventoryServiceImpl::new(Arc::new(mock_repo));

        // Reserve stock
        let reserve_result = service
            .reserve_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(reserve_result.is_ok());

        // Release stock
        let release_result = service
            .release_stock(tenant_id, warehouse_id, product_id, quantity)
            .await;
        assert!(release_result.is_ok());
    }
}
