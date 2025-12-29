//! Cycle Counting Service Trait
//!
//! This module defines the service trait for cycle counting operations.
//! No implementations here - pure interfaces following the 3-crate pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::cycle_count::{
    CreateCycleCountRequest, CycleCountListQuery, CycleCountListResponse, CycleCountResponse,
    CycleCountWithLinesResponse, GenerateLinesRequest, ReconcileRequest, ReconcileResponse,
    SkipLinesRequest, SubmitCountsRequest,
};
use shared_error::AppError;

/// Service trait for cycle counting operations
///
/// Cycle counting provides a workflow for performing partial inventory counts
/// on a scheduled or ad-hoc basis, with reconciliation to stock adjustments.
///
/// Key features:
/// - Session-based counting with as-of snapshot semantics
/// - Scope filtering by warehouse/location/product/category
/// - Multiple count submissions per session
/// - Reconciliation generates stock adjustments atomically
/// - Idempotent reconciliation (retry-safe)
#[async_trait]
pub trait CycleCountingService: Send + Sync {
    /// Create a new cycle count session
    ///
    /// Creates a session in Draft status with an as-of snapshot timestamp.
    /// Lines can be generated immediately or via a separate call.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `user_id` - User creating the session
    /// * `request` - Session creation parameters
    ///
    /// # Returns
    /// The created cycle count session
    async fn create_session(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateCycleCountRequest,
    ) -> Result<CycleCountResponse, AppError>;

    /// Get a cycle count session by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    ///
    /// # Returns
    /// The cycle count session with its lines and summary
    async fn get_session(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
    ) -> Result<CycleCountWithLinesResponse, AppError>;

    /// List cycle count sessions with filtering
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `query` - Filter and pagination parameters
    ///
    /// # Returns
    /// Paginated list of cycle count sessions
    async fn list_sessions(
        &self,
        tenant_id: Uuid,
        query: CycleCountListQuery,
    ) -> Result<CycleCountListResponse, AppError>;

    /// Generate or refresh count lines for a session
    ///
    /// Generates lines based on current inventory filtered by scope.
    /// Expected quantities are computed as of the session's as_of timestamp.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    /// * `user_id` - User performing the operation
    /// * `request` - Line generation parameters
    ///
    /// # Returns
    /// The updated session with generated lines
    ///
    /// # Errors
    /// - Session not found
    /// - Session status does not allow line generation
    async fn generate_lines(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
        request: GenerateLinesRequest,
    ) -> Result<CycleCountWithLinesResponse, AppError>;

    /// Submit counts for one or more lines
    ///
    /// Updates the counted quantities for specified lines.
    /// Automatically transitions session from Draft to InProgress on first count.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    /// * `user_id` - User submitting the counts
    /// * `request` - Count submissions
    ///
    /// # Returns
    /// The updated session with lines
    ///
    /// # Errors
    /// - Session not found
    /// - Session status does not allow counting
    /// - Line IDs not found or don't belong to this session
    /// - Negative quantities
    async fn submit_counts(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
        request: SubmitCountsRequest,
    ) -> Result<CycleCountWithLinesResponse, AppError>;

    /// Skip one or more lines
    ///
    /// Marks lines as skipped so they don't require counting for closure.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    /// * `user_id` - User performing the skip
    /// * `request` - Line IDs to skip
    ///
    /// # Returns
    /// The updated session with lines
    async fn skip_lines(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
        request: SkipLinesRequest,
    ) -> Result<CycleCountWithLinesResponse, AppError>;

    /// Close a cycle count session (mark ready for reconciliation)
    ///
    /// Validates that all lines are either counted or skipped.
    /// Transitions status to ReadyToReconcile.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    /// * `user_id` - User closing the session
    ///
    /// # Returns
    /// The updated session
    ///
    /// # Errors
    /// - Session not found
    /// - Session not in InProgress status
    /// - Uncounted lines exist (not skipped)
    async fn close_session(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
    ) -> Result<CycleCountResponse, AppError>;

    /// Reconcile differences and generate stock adjustments
    ///
    /// Atomically creates stock adjustments for all variances.
    /// This operation is idempotent - if already reconciled, returns existing result.
    ///
    /// MVP behavior for movements after as_of:
    /// - If force=false and movements detected, returns error
    /// - If force=true, proceeds with reconciliation anyway
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    /// * `user_id` - User performing reconciliation
    /// * `request` - Reconciliation options
    ///
    /// # Returns
    /// Reconciliation result with adjustment details
    ///
    /// # Errors
    /// - Session not found
    /// - Session not in ReadyToReconcile status
    /// - Stock movements detected after as_of (unless force=true)
    async fn reconcile(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
        request: ReconcileRequest,
    ) -> Result<ReconcileResponse, AppError>;

    /// Cancel a cycle count session
    ///
    /// Only Draft or InProgress sessions can be cancelled.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant isolation key
    /// * `cycle_count_id` - Session ID
    /// * `user_id` - User cancelling the session
    ///
    /// # Returns
    /// The cancelled session
    ///
    /// # Errors
    /// - Session not found
    /// - Session already reconciled or cancelled
    async fn cancel_session(
        &self,
        tenant_id: Uuid,
        cycle_count_id: Uuid,
        user_id: Uuid,
    ) -> Result<CycleCountResponse, AppError>;
}
