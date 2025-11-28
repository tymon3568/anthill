//! Service traits for RMA operations
//!
//! This module contains trait definitions for RMA business logic operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::rma::{
    ApproveRmaRequest, ApproveRmaResponse, CreateRmaRequest, CreateRmaResponse, ReceiveRmaRequest,
    ReceiveRmaResponse,
};
use shared_error::AppError;

/// Service trait for RMA operations
#[async_trait]
pub trait RmaService: Send + Sync {
    /// Create a new RMA request
    async fn create_rma(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateRmaRequest,
    ) -> Result<CreateRmaResponse, AppError>;

    /// Approve or reject an RMA request
    async fn approve_rma(
        &self,
        tenant_id: Uuid,
        rma_id: Uuid,
        user_id: Uuid,
        request: ApproveRmaRequest,
    ) -> Result<ApproveRmaResponse, AppError>;

    /// Receive returned goods for an RMA
    async fn receive_rma(
        &self,
        tenant_id: Uuid,
        rma_id: Uuid,
        user_id: Uuid,
        request: ReceiveRmaRequest,
    ) -> Result<ReceiveRmaResponse, AppError>;
}
