//! Transfer repository traits
//!
//! This module defines the repository traits for stock transfer operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::transfer::{Transfer, TransferItem, TransferStatus};
use shared_error::AppError;

/// Repository trait for stock transfer operations
#[async_trait]
pub trait TransferRepository: Send + Sync {
    /// Create a new transfer
    async fn create(&self, tenant_id: Uuid, transfer: &Transfer) -> Result<Transfer, AppError>;

    /// Find transfer by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
    ) -> Result<Option<Transfer>, AppError>;

    /// Update transfer status
    async fn update_status(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        status: TransferStatus,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Update transfer with confirmation details
    async fn confirm_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        approved_by: Uuid,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Update transfer with receipt details
    async fn receive_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Delete transfer (soft delete)
    async fn delete(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError>;
}

/// Repository trait for stock transfer item operations
#[async_trait]
pub trait TransferItemRepository: Send + Sync {
    /// Create multiple transfer items
    async fn create_batch(
        &self,
        tenant_id: Uuid,
        items: &[TransferItem],
    ) -> Result<Vec<TransferItem>, AppError>;

    /// Find items by transfer ID
    async fn find_by_transfer_id(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
    ) -> Result<Vec<TransferItem>, AppError>;

    /// Find item by ID
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<TransferItem>, AppError>;

    /// Update item quantity
    async fn update_quantity(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        quantity: i64,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Delete item (soft delete)
    async fn delete(
        &self,
        tenant_id: Uuid,
        item_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError>;
}
