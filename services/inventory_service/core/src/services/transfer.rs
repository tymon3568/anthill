//! Transfer service trait
//!
//! This module defines the service trait for stock transfer operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::transfer_dto::{
    ConfirmTransferRequest, ConfirmTransferResponse, CreateTransferRequest, CreateTransferResponse,
    ReceiveTransferRequest, ReceiveTransferResponse,
};
use shared_error::AppError;

/// Service trait for stock transfer operations
#[async_trait]
pub trait TransferService: Send + Sync {
    /// Create a new stock transfer in draft status
    ///
    /// Creates a transfer with items, validates business rules, and initializes
    /// total quantities and values.
    async fn create_transfer(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateTransferRequest,
    ) -> Result<CreateTransferResponse, AppError>;

    /// Confirm a draft transfer and move stock to In-Transit location
    ///
    /// Updates transfer status to confirmed, creates stock moves to move inventory
    /// from source warehouse to transit location, and updates inventory levels.
    async fn confirm_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        user_id: Uuid,
        request: ConfirmTransferRequest,
    ) -> Result<ConfirmTransferResponse, AppError>;

    /// Receive a shipped transfer at destination warehouse
    ///
    /// Updates transfer status to received, creates stock moves to move inventory
    /// from transit to destination warehouse, updates inventory levels, and
    /// publishes inventory.transfer.completed event.
    async fn receive_transfer(
        &self,
        tenant_id: Uuid,
        transfer_id: Uuid,
        user_id: Uuid,
        request: ReceiveTransferRequest,
    ) -> Result<ReceiveTransferResponse, AppError>;
}
