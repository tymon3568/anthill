//! Scrap Management Service Trait
//!
//! This module defines the service trait for scrap management operations.
//! No implementations here - pure interfaces following the 3-crate pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::scrap::{
    AddScrapLinesRequest, CreateScrapRequest, PostScrapRequest, ScrapDocumentResponse,
    ScrapDocumentWithLinesResponse, ScrapListQuery, ScrapListResponse,
};
use shared_error::AppError;

/// Service trait for scrap management operations
///
/// Scrap management provides a workflow for discarding/damaging/expired goods
/// with proper audit trail and inventory impact.
///
/// Key features:
/// - Document-based workflow (draft → posted)
/// - Multi-line scrap documents
/// - Integration with stock moves and valuation
/// - Tenant isolation
/// - Idempotent posting (retry-safe)
#[async_trait]
pub trait ScrapService: Send + Sync {
    /// Create a new scrap document (draft)
    ///
    /// Creates a scrap document in Draft status that can have lines added.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `user_id` - User creating the document
    /// * `request` - Document creation parameters
    ///
    /// # Returns
    /// The created scrap document
    async fn create_scrap(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateScrapRequest,
    ) -> Result<ScrapDocumentResponse, AppError>;

    /// Get a scrap document by ID with its lines
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `scrap_id` - Scrap document ID
    ///
    /// # Returns
    /// The scrap document with all its lines
    async fn get_scrap(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
    ) -> Result<ScrapDocumentWithLinesResponse, AppError>;

    /// List scrap documents with filtering
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `query` - Filter and pagination parameters
    ///
    /// # Returns
    /// Paginated list of scrap documents
    async fn list_scraps(
        &self,
        tenant_id: Uuid,
        query: ScrapListQuery,
    ) -> Result<ScrapListResponse, AppError>;

    /// Add or replace lines on a scrap document
    ///
    /// Lines can only be added to Draft documents.
    /// If lines already exist, they are replaced.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `scrap_id` - Scrap document ID
    /// * `user_id` - User adding the lines
    /// * `request` - Lines to add
    ///
    /// # Returns
    /// The updated scrap document with lines
    ///
    /// # Errors
    /// - Document not found
    /// - Document not in Draft status
    /// - Invalid line data (qty <= 0, etc.)
    /// - Product/location/lot not found or not belonging to tenant
    async fn add_lines(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
        user_id: Uuid,
        request: AddScrapLinesRequest,
    ) -> Result<ScrapDocumentWithLinesResponse, AppError>;

    /// Post a scrap document
    ///
    /// Atomically executes the scrap:
    /// - Validates all lines and inventory availability
    /// - Creates stock moves for each line (source → scrap location)
    /// - Updates inventory levels
    /// - Applies valuation adjustments
    /// - Marks document as Posted
    ///
    /// This operation is idempotent - if already posted, returns existing result.
    /// Uses idempotency_key if provided for additional retry safety.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `scrap_id` - Scrap document ID
    /// * `user_id` - User posting the document
    /// * `request` - Post options (idempotency_key)
    ///
    /// # Returns
    /// The posted scrap document
    ///
    /// # Errors
    /// - Document not found
    /// - Document not in Draft status
    /// - No lines on document
    /// - Insufficient inventory at source locations
    async fn post_scrap(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
        user_id: Uuid,
        request: PostScrapRequest,
    ) -> Result<ScrapDocumentResponse, AppError>;

    /// Cancel a draft scrap document
    ///
    /// Only Draft documents can be cancelled. Posted documents cannot be cancelled
    /// (would require reverse movements, not supported in MVP).
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `scrap_id` - Scrap document ID
    /// * `user_id` - User cancelling the document
    ///
    /// # Returns
    /// The cancelled scrap document
    ///
    /// # Errors
    /// - Document not found
    /// - Document not in Draft status (already posted or cancelled)
    async fn cancel_scrap(
        &self,
        tenant_id: Uuid,
        scrap_id: Uuid,
        user_id: Uuid,
    ) -> Result<ScrapDocumentResponse, AppError>;
}
