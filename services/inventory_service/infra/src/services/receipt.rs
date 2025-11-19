//! Implementation of ReceiptService business logic
//!
//! This module provides the concrete implementation of the ReceiptService trait,
//! orchestrating receipt creation with validation, stock movements, and event publishing.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::dto::receipt::{
    ReceiptCreateRequest, ReceiptListQuery, ReceiptListResponse, ReceiptResponse,
};
use inventory_service_core::repositories::receipt::{
    OutboxRepository, ReceiptRepository, StockMoveRepository,
};
use inventory_service_core::services::receipt::ReceiptService;
use shared_error::AppError;

/// Implementation of ReceiptService
///
/// Orchestrates the creation and management of Goods Receipt Notes (GRN)
/// with proper validation, transaction management, and side effects.
pub struct ReceiptServiceImpl<R> {
    receipt_repository: Arc<R>,
}

impl<R> ReceiptServiceImpl<R>
where
    R: ReceiptRepository + Send + Sync,
{
    /// Create a new ReceiptServiceImpl
    ///
    /// # Arguments
    /// * `receipt_repository` - Repository for receipt operations
    ///
    /// # Returns
    /// New ReceiptServiceImpl instance
    pub fn new(receipt_repository: Arc<R>) -> Self {
        Self { receipt_repository }
    }
}

#[async_trait]
impl<R> ReceiptService for ReceiptServiceImpl<R>
where
    R: ReceiptRepository + Send + Sync,
{
    /// Create a new goods receipt note with validation and side effects
    async fn create_receipt(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: ReceiptCreateRequest,
    ) -> Result<ReceiptResponse, AppError> {
        // Validate request
        self.validate_receipt_request(tenant_id, &request).await?;

        // Generate idempotency key from request data
        let idempotency_key = generate_idempotency_key(&request);

        // Create receipt, items, stock moves, and outbox event in a single transaction
        let receipt = self
            .receipt_repository
            .create_receipt(tenant_id, user_id, &request, &idempotency_key)
            .await?;

        Ok(receipt)
    }

    /// Get a receipt by ID with full details
    async fn get_receipt(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
    ) -> Result<ReceiptResponse, AppError> {
        self.receipt_repository
            .get_receipt(tenant_id, receipt_id)
            .await
    }

    /// List receipts with pagination and filtering
    async fn list_receipts(
        &self,
        tenant_id: Uuid,
        query: ReceiptListQuery,
    ) -> Result<ReceiptListResponse, AppError> {
        self.receipt_repository
            .list_receipts(tenant_id, query)
            .await
    }

    /// Validate and complete a goods receipt note
    async fn validate_receipt(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
        user_id: Uuid,
    ) -> Result<ReceiptResponse, AppError> {
        self.receipt_repository
            .validate_receipt(tenant_id, receipt_id, user_id)
            .await
    }

    /// Validate receipt data before creation
    async fn validate_receipt_request(
        &self,
        tenant_id: Uuid,
        request: &ReceiptCreateRequest,
    ) -> Result<(), AppError> {
        // Validate that warehouse exists (basic check)
        // In a real implementation, you'd check against warehouse repository
        if request.warehouse_id.is_nil() {
            return Err(AppError::ValidationError("Warehouse ID is required".to_string()));
        }

        // Validate items
        if request.items.is_empty() {
            return Err(AppError::ValidationError(
                "At least one receipt item is required".to_string(),
            ));
        }

        for (index, item) in request.items.iter().enumerate() {
            if item.product_id.is_nil() {
                return Err(AppError::ValidationError(format!(
                    "Item {}: Product ID is required",
                    index + 1
                )));
            }
            if item.received_quantity <= 0 {
                return Err(AppError::ValidationError(format!(
                    "Item {}: Received quantity must be positive",
                    index + 1
                )));
            }
            if item.expected_quantity < 0 {
                return Err(AppError::ValidationError(format!(
                    "Item {}: Expected quantity cannot be negative",
                    index + 1
                )));
            }
            if let Some(cost) = item.unit_cost {
                if cost < 0 {
                    return Err(AppError::ValidationError(format!(
                        "Item {}: Unit cost cannot be negative",
                        index + 1
                    )));
                }
            }
        }

        // Additional validations could include:
        // - Check if products exist
        // - Check if warehouse belongs to tenant
        // - Check if supplier exists (if provided)
        // - Business rule validations

        Ok(())
    }
}

/// Generate idempotency key from request data
///
/// Creates a deterministic key based on key request fields to prevent duplicates.
/// In production, this should be more sophisticated and include user context.
fn generate_idempotency_key(request: &ReceiptCreateRequest) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(request.warehouse_id.as_bytes());
    if let Some(supplier_id) = request.supplier_id {
        hasher.update(supplier_id.as_bytes());
    }
    if let Some(ref_num) = &request.reference_number {
        hasher.update(ref_num.as_bytes());
    }

    // Sort items for consistent hashing
    let mut sorted_items = request.items.clone();
    sorted_items.sort_by_key(|item| item.product_id);

    // Hash the items
    for item in &sorted_items {
        hasher.update(item.product_id.as_bytes());
        hasher.update(&item.received_quantity.to_le_bytes());
    }

    format!("receipt-{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use validator::Validate;

    #[test]
    fn test_generate_idempotency_key() {
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            supplier_id: Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap()),
            reference_number: Some("PO-123".to_string()),
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
                    expected_quantity: 10,
                    received_quantity: 8,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let key1 = generate_idempotency_key(&request);
        let key2 = generate_idempotency_key(&request);

        // Same request should generate same key
        assert_eq!(key1, key2);

        // Different request should generate different key
        let mut different_request = request.clone();
        different_request.items[0].received_quantity = 9;
        let key3 = generate_idempotency_key(&different_request);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_dto_validation_valid() {
        // This would require mocking repositories
        // For now, just test basic validation logic
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: 10,
                    received_quantity: 8,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        // Basic validation should pass (detailed validation requires repositories)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_dto_validation_invalid() {
        // Note: These tests only cover DTO-level validation.
        // Service-level validation (e.g., warehouse/product existence) requires mocking repositories.
        // Test empty items
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![],
        };

        assert!(request.validate().is_err());

        // Test negative quantity
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: 10,
                    received_quantity: -1,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        assert!(request.validate().is_err());
    }

    // Dummy repository for testing service-level validation
    struct DummyReceiptRepository;

    #[async_trait]
    impl ReceiptRepository for DummyReceiptRepository {
        async fn create_receipt(
            &self,
            _tenant_id: Uuid,
            _user_id: Uuid,
            _request: &ReceiptCreateRequest,
            _idempotency_key: &str,
        ) -> Result<ReceiptResponse, AppError> {
            unimplemented!("Not needed for validation tests")
        }

        async fn get_receipt(
            &self,
            _tenant_id: Uuid,
            _receipt_id: Uuid,
        ) -> Result<ReceiptResponse, AppError> {
            unimplemented!("Not needed for validation tests")
        }

        async fn list_receipts(
            &self,
            _tenant_id: Uuid,
            _query: ReceiptListQuery,
        ) -> Result<ReceiptListResponse, AppError> {
            unimplemented!("Not needed for validation tests")
        }

        async fn receipt_exists(
            &self,
            _tenant_id: Uuid,
            _receipt_id: Uuid,
        ) -> Result<bool, AppError> {
            unimplemented!("Not needed for validation tests")
        }

        async fn validate_receipt(
            &self,
            _tenant_id: Uuid,
            _receipt_id: Uuid,
            _user_id: Uuid,
        ) -> Result<ReceiptResponse, AppError> {
            unimplemented!("Not needed for validation tests")
        }
    }

    #[tokio::test]
    async fn test_service_validation_valid_request() {
        let tenant_id = Uuid::new_v4();
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: 10,
                    received_quantity: 8,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_validation_invalid_empty_items() {
        let tenant_id = Uuid::new_v4();
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "At least one receipt item is required");
    }

    #[tokio::test]
    async fn test_service_validation_invalid_nil_warehouse() {
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);

        let tenant_id = Uuid::new_v4();
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::nil(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: 10,
                    received_quantity: 8,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Warehouse ID is required");
    }

    #[tokio::test]
    async fn test_service_validation_invalid_nil_product() {
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);

        let tenant_id = Uuid::new_v4();
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::nil(),
                    expected_quantity: 10,
                    received_quantity: 8,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Item 1: Product ID is required");
    }

    #[tokio::test]
    async fn test_service_validation_invalid_zero_received_quantity() {
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);

        let tenant_id = Uuid::new_v4();
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: 10,
                    received_quantity: 0,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Item 1: Received quantity must be positive");
    }

    #[tokio::test]
    async fn test_service_validation_invalid_negative_expected_quantity() {
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);

        let tenant_id = Uuid::new_v4();
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: -1,
                    received_quantity: 8,
                    unit_cost: Some(1000),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Item 1: Expected quantity cannot be negative");
    }

    #[tokio::test]
    async fn test_service_validation_invalid_negative_unit_cost() {
        let repo = Arc::new(DummyReceiptRepository);
        let service = ReceiptServiceImpl::new(repo);

        let tenant_id = Uuid::new_v4();
        let request = ReceiptCreateRequest {
            warehouse_id: Uuid::new_v4(),
            supplier_id: None,
            reference_number: None,
            expected_delivery_date: None,
            notes: None,
            currency_code: "USD".to_string(),
            items: vec![
                inventory_service_core::dto::receipt::ReceiptItemCreateRequest {
                    product_id: Uuid::new_v4(),
                    expected_quantity: 10,
                    received_quantity: 8,
                    unit_cost: Some(-100),
                    uom_id: None,
                    lot_number: None,
                    serial_numbers: None,
                    expiry_date: None,
                    notes: None,
                },
            ],
        };

        let result = service.validate_receipt_request(tenant_id, &request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Item 1: Unit cost cannot be negative");
    }
}
