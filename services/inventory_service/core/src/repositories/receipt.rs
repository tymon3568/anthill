//! Repository traits for receipt operations
//!
//! This module contains trait definitions for Goods Receipt Note (GRN) data access operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::receipt::{
    ReceiptCreateRequest, ReceiptListQuery, ReceiptListResponse, ReceiptResponse,
};
use shared_error::AppError;

/// Repository trait for receipt operations
#[async_trait]
pub trait ReceiptRepository: Send + Sync {
    /// Create a new goods receipt with items in a single transaction
    async fn create_receipt(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: &ReceiptCreateRequest,
        idempotency_key: &str,
    ) -> Result<ReceiptResponse, AppError>;

    /// Get a receipt by ID
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

    /// Check if a receipt exists by ID
    async fn receipt_exists(&self, tenant_id: Uuid, receipt_id: Uuid) -> Result<bool, AppError>;
}
