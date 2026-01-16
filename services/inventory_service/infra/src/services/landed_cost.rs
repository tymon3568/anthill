//! Landed Cost Service implementation.
//!
//! Business logic for landed cost management including document creation,
//! cost allocation, and posting.

use async_trait::async_trait;
use chrono::Utc;
use inventory_service_core::domains::inventory::dto::landed_cost_dto::{
    AddLandedCostLineRequest, AllocationPreviewItem, AllocationPreviewResponse,
    CreateLandedCostDocumentRequest, LandedCostDocumentListResponse,
    LandedCostDocumentWithLinesDto, LandedCostLineDto, PostLandedCostResponse,
    UpdateLandedCostDocumentRequest, UpdateLandedCostLineRequest,
};
use inventory_service_core::domains::inventory::landed_cost::{
    LandedCostAllocation, LandedCostDocument, LandedCostDocumentWithLines, LandedCostStatus,
};
use inventory_service_core::repositories::landed_cost::{
    LandedCostAllocationRepository, LandedCostDocumentRepository, LandedCostLineRepository,
};
use inventory_service_core::services::landed_cost::LandedCostService;
use inventory_service_core::AppError;
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL implementation of `LandedCostService`.
pub struct LandedCostServiceImpl {
    document_repo: Arc<dyn LandedCostDocumentRepository>,
    line_repo: Arc<dyn LandedCostLineRepository>,
    allocation_repo: Arc<dyn LandedCostAllocationRepository>,
}

impl LandedCostServiceImpl {
    /// Create a new service instance.
    pub fn new(
        document_repo: Arc<dyn LandedCostDocumentRepository>,
        line_repo: Arc<dyn LandedCostLineRepository>,
        allocation_repo: Arc<dyn LandedCostAllocationRepository>,
    ) -> Self {
        Self {
            document_repo,
            line_repo,
            allocation_repo,
        }
    }

    /// Recalculate and update the document total cost.
    async fn recalculate_total_cost(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<(), AppError> {
        let lines = self
            .line_repo
            .find_by_document_id(tenant_id, document_id)
            .await?;
        let total: i64 = lines.iter().map(|l| l.amount).sum();
        self.document_repo
            .update_total_cost(tenant_id, document_id, total)
            .await
    }

    /// Validate document is in draft status.
    fn validate_draft_status(&self, doc: &LandedCostDocument) -> Result<(), AppError> {
        if doc.status != LandedCostStatus::Draft {
            return Err(AppError::ValidationError(
                "Document must be in draft status for this operation".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate document is in posted status.
    fn validate_posted_status(&self, doc: &LandedCostDocument) -> Result<(), AppError> {
        if doc.status != LandedCostStatus::Posted {
            return Err(AppError::ValidationError(
                "Document must be in posted status for cancellation".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl LandedCostService for LandedCostServiceImpl {
    async fn create_document(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateLandedCostDocumentRequest,
    ) -> Result<LandedCostDocumentWithLinesDto, AppError> {
        // Generate document number
        let document_number = self
            .document_repo
            .generate_document_number(tenant_id)
            .await?;

        // Create the document
        let document = self
            .document_repo
            .create(
                tenant_id,
                request.receipt_id,
                document_number,
                request.reference_number,
                request.allocation_method,
                request.currency_code,
                request.notes,
                user_id,
            )
            .await?;

        // Create initial lines if provided
        let mut lines = Vec::new();
        for line_req in request.lines {
            let line = self
                .line_repo
                .create(
                    tenant_id,
                    document.document_id,
                    line_req.cost_type,
                    line_req.description,
                    line_req.amount,
                    line_req.vendor_reference,
                )
                .await?;
            lines.push(line);
        }

        // Recalculate total if lines were added
        if !lines.is_empty() {
            self.recalculate_total_cost(tenant_id, document.document_id)
                .await?;
        }

        // Fetch updated document
        let updated_doc = self
            .document_repo
            .find_by_id(tenant_id, document.document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        Ok(LandedCostDocumentWithLinesDto {
            data: LandedCostDocumentWithLines {
                document: updated_doc,
                lines,
                allocations: None,
            },
        })
    }

    async fn get_document(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<LandedCostDocumentWithLinesDto, AppError> {
        let document = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        let lines = self
            .line_repo
            .find_by_document_id(tenant_id, document_id)
            .await?;

        // Fetch allocations if document is posted
        let allocations = if document.status == LandedCostStatus::Posted {
            Some(
                self.allocation_repo
                    .find_by_document_id(tenant_id, document_id)
                    .await?,
            )
        } else {
            None
        };

        Ok(LandedCostDocumentWithLinesDto {
            data: LandedCostDocumentWithLines {
                document,
                lines,
                allocations,
            },
        })
    }

    async fn list_documents(
        &self,
        tenant_id: Uuid,
        status: Option<String>,
        receipt_id: Option<Uuid>,
        page: i32,
        page_size: i32,
    ) -> Result<LandedCostDocumentListResponse, AppError> {
        let status_enum = match status {
            Some(s) => {
                let parsed = LandedCostStatus::parse(&s).ok_or_else(|| {
                    AppError::ValidationError(format!(
                        "Invalid status '{}'. Valid values: draft, posted, cancelled",
                        s
                    ))
                })?;
                Some(parsed)
            },
            None => None,
        };

        let (documents, total) = self
            .document_repo
            .list(tenant_id, status_enum, receipt_id, page, page_size)
            .await?;

        Ok(LandedCostDocumentListResponse {
            documents,
            total,
            page,
            page_size,
        })
    }

    async fn update_document(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        request: UpdateLandedCostDocumentRequest,
    ) -> Result<LandedCostDocument, AppError> {
        // Verify document exists and is in draft status
        let doc = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_draft_status(&doc)?;

        // Update the document
        self.document_repo
            .update(
                tenant_id,
                document_id,
                request.reference_number,
                request.allocation_method,
                request.notes,
            )
            .await
    }

    async fn delete_document(&self, tenant_id: Uuid, document_id: Uuid) -> Result<(), AppError> {
        // Verify document exists and is in draft status
        let doc = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_draft_status(&doc)?;

        // Delete lines first
        self.line_repo
            .delete_by_document_id(tenant_id, document_id)
            .await?;

        // Delete the document
        self.document_repo.delete(tenant_id, document_id).await
    }

    async fn add_line(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        request: AddLandedCostLineRequest,
    ) -> Result<LandedCostLineDto, AppError> {
        // Verify document exists and is in draft status
        let doc = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_draft_status(&doc)?;

        // Create the line
        let line = self
            .line_repo
            .create(
                tenant_id,
                document_id,
                request.cost_type,
                request.description,
                request.amount,
                request.vendor_reference,
            )
            .await?;

        // Recalculate total
        self.recalculate_total_cost(tenant_id, document_id).await?;

        Ok(LandedCostLineDto { line })
    }

    async fn update_line(
        &self,
        tenant_id: Uuid,
        line_id: Uuid,
        request: UpdateLandedCostLineRequest,
    ) -> Result<LandedCostLineDto, AppError> {
        // Get the line to find its document
        let existing_line = self
            .line_repo
            .find_by_id(tenant_id, line_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost line not found".to_string()))?;

        // Verify document is in draft status
        let doc = self
            .document_repo
            .find_by_id(tenant_id, existing_line.document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_draft_status(&doc)?;

        // Update the line
        let line = self
            .line_repo
            .update(
                tenant_id,
                line_id,
                request.cost_type,
                request.description,
                request.amount,
                request.vendor_reference,
            )
            .await?;

        // Recalculate total
        self.recalculate_total_cost(tenant_id, existing_line.document_id)
            .await?;

        Ok(LandedCostLineDto { line })
    }

    async fn delete_line(&self, tenant_id: Uuid, line_id: Uuid) -> Result<(), AppError> {
        // Get the line to find its document
        let existing_line = self
            .line_repo
            .find_by_id(tenant_id, line_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost line not found".to_string()))?;

        // Verify document is in draft status
        let doc = self
            .document_repo
            .find_by_id(tenant_id, existing_line.document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_draft_status(&doc)?;

        // Delete the line
        self.line_repo.delete(tenant_id, line_id).await?;

        // Recalculate total
        self.recalculate_total_cost(tenant_id, existing_line.document_id)
            .await
    }

    async fn get_allocation_preview(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<AllocationPreviewResponse, AppError> {
        // Get document
        let doc = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        // Get lines to calculate total
        let lines = self
            .line_repo
            .find_by_document_id(tenant_id, document_id)
            .await?;
        let total_cost: i64 = lines.iter().map(|l| l.amount).sum();

        if lines.is_empty() {
            return Err(AppError::ValidationError(
                "Document must have at least one cost line".to_string(),
            ));
        }

        // For now, create mock preview allocations
        // In production, this would fetch receipt items and calculate allocations
        let allocations: Vec<AllocationPreviewItem> = vec![];

        Ok(AllocationPreviewResponse {
            document_id,
            total_cost_amount: total_cost,
            allocation_method: doc.allocation_method,
            allocations,
        })
    }

    async fn post_document(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<PostLandedCostResponse, AppError> {
        // Get document
        let doc = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_draft_status(&doc)?;

        // Get lines to validate
        let lines = self
            .line_repo
            .find_by_document_id(tenant_id, document_id)
            .await?;
        if lines.is_empty() {
            return Err(AppError::ValidationError(
                "Document must have at least one cost line to post".to_string(),
            ));
        }

        let total_cost: i64 = lines.iter().map(|l| l.amount).sum();

        // Create allocations (simplified - in production would fetch receipt items)
        // For now, create a placeholder allocation for the receipt
        let now = Utc::now();
        let allocation = LandedCostAllocation {
            allocation_id: Uuid::now_v7(),
            tenant_id,
            document_id,
            receipt_item_id: doc.receipt_id, // Using receipt_id as placeholder
            allocated_amount: total_cost,
            original_unit_cost: 0,
            new_unit_cost: 0,
            created_at: now,
        };

        let allocations = self.allocation_repo.create_batch(vec![allocation]).await?;

        // Update document status to posted
        let updated_doc = self
            .document_repo
            .update_status(tenant_id, document_id, LandedCostStatus::Posted)
            .await?;

        Ok(PostLandedCostResponse {
            document_id,
            status: updated_doc.status,
            posted_at: updated_doc.posted_at.unwrap_or(now),
            allocations,
            items_affected: 1,
        })
    }

    async fn cancel_document(&self, tenant_id: Uuid, document_id: Uuid) -> Result<(), AppError> {
        // Get document
        let doc = self
            .document_repo
            .find_by_id(tenant_id, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost document not found".to_string()))?;

        self.validate_posted_status(&doc)?;

        // Delete existing allocations
        self.allocation_repo
            .delete_by_document_id(tenant_id, document_id)
            .await?;

        // Update document status to cancelled
        self.document_repo
            .update_status(tenant_id, document_id, LandedCostStatus::Cancelled)
            .await?;

        Ok(())
    }
}
