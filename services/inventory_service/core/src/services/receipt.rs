//! Service traits for receipt operations
//!
//! This module contains trait definitions for Goods Receipt Note (GRN) business logic operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::receipt::{
    ReceiptCreateRequest, ReceiptListQuery, ReceiptListResponse, ReceiptResponse,
};
use shared_error::AppError;

/// Service trait for receipt operations
#[async_trait]
pub trait ReceiptService: Send + Sync {
    /// Create a new goods receipt note with validation and side effects
    async fn create_receipt(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: ReceiptCreateRequest,
    ) -> Result<ReceiptResponse, AppError>;

    /// Get a receipt by ID with full details
    async fn get_receipt(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
    ) -> Result<ReceiptResponse, AppError>;

    /// List receipts with pagination and filtering
    async fn list_receipts(
        &self,
        tenant_id: Uuid,
        query: ReceiptListQuery,
    ) -> Result<ReceiptListResponse, AppError>;

    /// Validate and complete a goods receipt note
    async fn validate_receipt(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
        user_id: Uuid,
    ) -> Result<ReceiptResponse, AppError>;

    /// Validate receipt data before creation
    async fn validate_receipt_request(
        &self,
        tenant_id: Uuid,
        request: &ReceiptCreateRequest,
    ) -> Result<(), AppError>;
}
