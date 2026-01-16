//! Landed Cost service trait
//!
//! Defines the business logic interface for landed cost operations.
//! Supports creating, posting, and managing landed cost allocations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::landed_cost_dto::{
    AllocationPreviewResponse, CreateLandedCostDocumentRequest, CreateLandedCostLineRequest,
    LandedCostDocumentListResponse, LandedCostDocumentWithLinesDto, LandedCostLineDto,
    PostLandedCostResponse, UpdateLandedCostDocumentRequest, UpdateLandedCostLineRequest,
};
use crate::domains::inventory::landed_cost::LandedCostDocument;
use crate::Result;

/// Service trait for landed cost business logic
///
/// This trait defines all business operations for landed cost management.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait LandedCostService: Send + Sync {
    /// Create a new landed cost document.
    ///
    /// # Business Rules
    /// - Validates that the receipt exists and is in a valid state
    /// - Generates a unique document number
    /// - Creates the document in draft status
    /// - Optionally creates initial cost lines if provided
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `user_id` - User creating the document
    /// * `request` - Document creation request
    ///
    /// # Returns
    /// The created document with lines
    async fn create_document(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateLandedCostDocumentRequest,
    ) -> Result<LandedCostDocumentWithLinesDto>;

    /// Get a landed cost document by ID.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    ///
    /// # Returns
    /// The document with lines and allocations (if posted)
    async fn get_document(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<LandedCostDocumentWithLinesDto>;

    /// List landed cost documents with filters.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `status` - Optional status filter
    /// * `receipt_id` - Optional receipt filter
    /// * `page` - Page number (1-based)
    /// * `page_size` - Items per page
    ///
    /// # Returns
    /// Paginated list of documents
    async fn list_documents(
        &self,
        tenant_id: Uuid,
        status: Option<String>,
        receipt_id: Option<Uuid>,
        page: i32,
        page_size: i32,
    ) -> Result<LandedCostDocumentListResponse>;

    /// Update a landed cost document.
    ///
    /// # Business Rules
    /// - Only draft documents can be updated
    /// - Cannot change the associated receipt
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    /// * `request` - Update request
    ///
    /// # Returns
    /// The updated document
    async fn update_document(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        request: UpdateLandedCostDocumentRequest,
    ) -> Result<LandedCostDocument>;

    /// Delete a landed cost document.
    ///
    /// # Business Rules
    /// - Only draft documents can be deleted
    /// - Posted documents must be cancelled first
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    async fn delete_document(&self, tenant_id: Uuid, document_id: Uuid) -> Result<()>;

    /// Add a cost line to a document.
    ///
    /// # Business Rules
    /// - Only draft documents can have lines added
    /// - Updates document total cost
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    /// * `request` - Line creation request
    ///
    /// # Returns
    /// The created line
    async fn add_line(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        request: CreateLandedCostLineRequest,
    ) -> Result<LandedCostLineDto>;

    /// Update a cost line.
    ///
    /// # Business Rules
    /// - Only lines on draft documents can be updated
    /// - Updates document total cost
    /// - Validates that line belongs to the specified document
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier (for ownership validation)
    /// * `line_id` - Line identifier
    /// * `request` - Update request
    ///
    /// # Returns
    /// The updated line
    async fn update_line(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        line_id: Uuid,
        request: UpdateLandedCostLineRequest,
    ) -> Result<LandedCostLineDto>;

    /// Delete a cost line.
    ///
    /// # Business Rules
    /// - Only lines on draft documents can be deleted
    /// - Updates document total cost
    /// - Validates that line belongs to the specified document
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier (for ownership validation)
    /// * `line_id` - Line identifier
    async fn delete_line(&self, tenant_id: Uuid, document_id: Uuid, line_id: Uuid) -> Result<()>;

    /// Get allocation preview before posting.
    ///
    /// # Business Rules
    /// - Shows how costs would be allocated without actually posting
    /// - Uses the document's allocation method
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    ///
    /// # Returns
    /// Preview of allocations
    async fn get_allocation_preview(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<AllocationPreviewResponse>;

    /// Post a landed cost document.
    ///
    /// # Business Rules
    /// - Only draft documents with at least one line can be posted
    /// - Calculates allocations based on allocation method
    /// - Creates allocation records
    /// - Updates receipt item unit costs
    /// - Updates document status to posted
    /// - Records valuation history entries
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    ///
    /// # Returns
    /// Post result with allocations
    async fn post_document(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<PostLandedCostResponse>;

    /// Cancel a posted landed cost document.
    ///
    /// # Business Rules
    /// - Only posted documents can be cancelled
    /// - Reverses the allocation (restores original unit costs)
    /// - Creates reversal allocation records
    /// - Updates document status to cancelled
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `document_id` - Document identifier
    async fn cancel_document(&self, tenant_id: Uuid, document_id: Uuid) -> Result<()>;
}
