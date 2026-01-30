//! Stock Adjustment Service Trait
//!
//! This module defines the service trait for stock adjustment operations.
//! No implementations here - pure interfaces following the 3-crate pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::adjustment::{
    AddAdjustmentLinesRequest, AdjustmentDocumentResponse, AdjustmentDocumentWithLinesResponse,
    AdjustmentListQuery, AdjustmentListResponse, AdjustmentSummary, CreateAdjustmentRequest,
    PostAdjustmentRequest,
};
use shared_error::AppError;

/// Service trait for stock adjustment operations
///
/// Stock adjustment management provides a workflow for recording inventory
/// changes outside normal transactions (e.g., damaged goods, count corrections).
///
/// Key features:
/// - Document-based workflow (draft → posted)
/// - Multi-line adjustment documents
/// - Support for both increase and decrease adjustments
/// - Multiple reason codes for different scenarios
/// - Integration with stock moves
/// - Tenant isolation
/// - Idempotent posting (retry-safe)
#[async_trait]
pub trait AdjustmentService: Send + Sync {
    /// Create a new adjustment document (draft)
    ///
    /// Creates an adjustment document in Draft status that can have lines added.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `user_id` - User creating the document
    /// * `request` - Document creation parameters
    ///
    /// # Returns
    /// The created adjustment document
    async fn create_adjustment(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateAdjustmentRequest,
    ) -> Result<AdjustmentDocumentWithLinesResponse, AppError>;

    /// Get an adjustment document by ID with its lines
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `adjustment_id` - Adjustment document ID
    ///
    /// # Returns
    /// The adjustment document with all its lines
    async fn get_adjustment(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
    ) -> Result<AdjustmentDocumentWithLinesResponse, AppError>;

    /// List adjustment documents with filtering
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `query` - Filter and pagination parameters
    ///
    /// # Returns
    /// Paginated list of adjustment documents
    async fn list_adjustments(
        &self,
        tenant_id: Uuid,
        query: AdjustmentListQuery,
    ) -> Result<AdjustmentListResponse, AppError>;

    /// Get summary statistics for adjustments
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `warehouse_id` - Optional warehouse filter
    ///
    /// # Returns
    /// Summary of adjustments (totals, increases, decreases, net change)
    async fn get_adjustment_summary(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<AdjustmentSummary, AppError>;

    /// Add lines to an adjustment document
    ///
    /// Lines can only be added to Draft documents.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `adjustment_id` - Adjustment document ID
    /// * `user_id` - User adding the lines
    /// * `request` - Lines to add
    ///
    /// # Returns
    /// The updated adjustment document with lines
    ///
    /// # Errors
    /// - Document not found
    /// - Document not in Draft status
    /// - Invalid line data (qty <= 0, etc.)
    /// - Product not found or not belonging to tenant
    async fn add_lines(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
        user_id: Uuid,
        request: AddAdjustmentLinesRequest,
    ) -> Result<AdjustmentDocumentWithLinesResponse, AppError>;

    /// Post an adjustment document
    ///
    /// Atomically executes the adjustment:
    /// - Validates all lines
    /// - Creates stock moves for each line (increase or decrease)
    /// - Updates inventory levels
    /// - Marks document as Posted
    ///
    /// This operation is idempotent - if already posted, returns existing result.
    /// Uses idempotency_key if provided for additional retry safety.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `adjustment_id` - Adjustment document ID
    /// * `user_id` - User posting the document
    /// * `request` - Post options (idempotency_key)
    ///
    /// # Returns
    /// The posted adjustment document
    ///
    /// # Errors
    /// - Document not found
    /// - Document not in Draft status
    /// - No lines on document
    /// - Insufficient inventory for decrease adjustments
    async fn post_adjustment(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
        user_id: Uuid,
        request: PostAdjustmentRequest,
    ) -> Result<AdjustmentDocumentResponse, AppError>;

    /// Cancel a draft adjustment document
    ///
    /// Only Draft documents can be cancelled. Posted documents cannot be cancelled
    /// (would require reverse movements, not supported in MVP).
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `adjustment_id` - Adjustment document ID
    /// * `user_id` - User cancelling the document
    ///
    /// # Returns
    /// The cancelled adjustment document
    ///
    /// # Errors
    /// - Document not found
    /// - Document not in Draft status (already posted or cancelled)
    async fn cancel_adjustment(
        &self,
        tenant_id: Uuid,
        adjustment_id: Uuid,
        user_id: Uuid,
    ) -> Result<AdjustmentDocumentResponse, AppError>;
}
