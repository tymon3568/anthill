//! Landed Cost Repository traits.
//!
//! Defines the repository interfaces for landed cost operations.

use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

use crate::domains::inventory::landed_cost::{
    AllocationMethod, LandedCostAllocation, LandedCostDocument, LandedCostLine, LandedCostStatus,
};

/// Repository for landed cost documents.
#[async_trait]
pub trait LandedCostDocumentRepository: Send + Sync {
    /// Create a new landed cost document.
    async fn create(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
        document_number: String,
        reference_number: Option<String>,
        allocation_method: AllocationMethod,
        currency_code: String,
        notes: Option<String>,
        created_by: Uuid,
    ) -> Result<LandedCostDocument, AppError>;

    /// Find a document by ID.
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Option<LandedCostDocument>, AppError>;

    /// Find documents by receipt ID.
    async fn find_by_receipt_id(
        &self,
        tenant_id: Uuid,
        receipt_id: Uuid,
    ) -> Result<Vec<LandedCostDocument>, AppError>;

    /// List documents with optional filters.
    async fn list(
        &self,
        tenant_id: Uuid,
        status: Option<LandedCostStatus>,
        receipt_id: Option<Uuid>,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<LandedCostDocument>, i64), AppError>;

    /// Update a document.
    async fn update(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        reference_number: Option<String>,
        allocation_method: Option<AllocationMethod>,
        notes: Option<String>,
    ) -> Result<LandedCostDocument, AppError>;

    /// Update document status.
    async fn update_status(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        status: LandedCostStatus,
    ) -> Result<LandedCostDocument, AppError>;

    /// Update document total cost amount.
    async fn update_total_cost(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        total_cost_amount: i64,
    ) -> Result<(), AppError>;

    /// Soft delete a document.
    async fn delete(&self, tenant_id: Uuid, document_id: Uuid) -> Result<(), AppError>;

    /// Generate next document number.
    async fn generate_document_number(&self, tenant_id: Uuid) -> Result<String, AppError>;
}

/// Repository for landed cost lines.
#[async_trait]
pub trait LandedCostLineRepository: Send + Sync {
    /// Create a new cost line.
    async fn create(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        cost_type: String,
        description: Option<String>,
        amount: i64,
        vendor_reference: Option<String>,
    ) -> Result<LandedCostLine, AppError>;

    /// Find lines by document ID.
    async fn find_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<LandedCostLine>, AppError>;

    /// Find a line by ID.
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
    ) -> Result<Option<LandedCostLine>, AppError>;

    /// Update a line.
    async fn update(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        cost_type: Option<String>,
        description: Option<String>,
        amount: Option<i64>,
        vendor_reference: Option<String>,
    ) -> Result<LandedCostLine, AppError>;

    /// Delete a line.
    async fn delete(&self, tenant_id: Uuid, line_id: Uuid) -> Result<(), AppError>;

    /// Delete all lines for a document.
    async fn delete_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<i64, AppError>;
}

/// Repository for landed cost allocations.
#[async_trait]
pub trait LandedCostAllocationRepository: Send + Sync {
    /// Create allocations in batch.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for multi-tenant isolation
    /// * `allocations` - Allocations to create (must all belong to the same tenant)
    async fn create_batch(
        &self,
        tenant_id: Uuid,
        allocations: Vec<LandedCostAllocation>,
    ) -> Result<Vec<LandedCostAllocation>, AppError>;

    /// Find allocations by document ID.
    async fn find_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<LandedCostAllocation>, AppError>;

    /// Find allocations by receipt item ID.
    async fn find_by_receipt_item_id(
        &self,
        tenant_id: Uuid,
        receipt_item_id: Uuid,
    ) -> Result<Vec<LandedCostAllocation>, AppError>;

    /// Delete allocations by document ID (for rollback/cancel).
    async fn delete_by_document_id(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<i64, AppError>;
}
