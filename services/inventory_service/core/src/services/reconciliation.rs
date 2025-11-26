//! Stock Reconciliation service trait
//!
//! This module defines the service trait for reconciliation operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::reconciliation::{
    ApproveReconciliationRequest, ApproveReconciliationResponse, CountReconciliationRequest,
    CountReconciliationResponse, CreateReconciliationRequest, CreateReconciliationResponse,
    FinalizeReconciliationRequest, FinalizeReconciliationResponse, ReconciliationAnalyticsResponse,
    ReconciliationDetailResponse, ReconciliationListQuery, ReconciliationListResponse,
    VarianceAnalysisResponse,
};
use shared_error::AppError;

/// Service trait for reconciliation operations
#[async_trait]
pub trait StockReconciliationService: Send + Sync {
    /// Create a new reconciliation session
    ///
    /// Creates a reconciliation in draft status, selects products based on cycle type and filters,
    /// snapshots current inventory levels, and creates reconciliation items.
    async fn create_reconciliation(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateReconciliationRequest,
    ) -> Result<CreateReconciliationResponse, AppError>;

    /// Submit counted quantities for reconciliation items
    ///
    /// Updates counted quantities for the specified products, calculates variances,
    /// and updates the reconciliation status and summary.
    async fn count_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        user_id: Uuid,
        request: CountReconciliationRequest,
    ) -> Result<CountReconciliationResponse, AppError>;

    /// Finalize the reconciliation and generate inventory adjustments
    ///
    /// Marks the reconciliation as completed, generates stock adjustments for discrepancies,
    /// and updates inventory levels accordingly.
    async fn finalize_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        user_id: Uuid,
        request: FinalizeReconciliationRequest,
    ) -> Result<FinalizeReconciliationResponse, AppError>;

    /// Approve the reconciliation
    ///
    /// Marks the reconciliation as approved, typically after review of variances.
    async fn approve_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        user_id: Uuid,
        request: ApproveReconciliationRequest,
    ) -> Result<ApproveReconciliationResponse, AppError>;

    /// Get reconciliation details with items
    async fn get_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<ReconciliationDetailResponse, AppError>;

    /// List reconciliations with filtering
    async fn list_reconciliations(
        &self,
        tenant_id: Uuid,
        query: ReconciliationListQuery,
    ) -> Result<ReconciliationListResponse, AppError>;

    /// Get reconciliation analytics
    async fn get_analytics(
        &self,
        tenant_id: Uuid,
        warehouse_id: Option<Uuid>,
    ) -> Result<ReconciliationAnalyticsResponse, AppError>;

    /// Get variance analysis for a specific reconciliation
    async fn get_variance_analysis(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<VarianceAnalysisResponse, AppError>;
}
