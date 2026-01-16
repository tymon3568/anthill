//! Landed Cost service trait
//!
//! Defines the business logic interface for landed cost operations.
//! Supports creating, managing, and posting landed costs with allocations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::dto::landed_cost_dto::{
    AddLandedCostLineRequest, ComputeAllocationsRequest, ComputeAllocationsResponse,
    CreateLandedCostRequest, LandedCostDetailDto, LandedCostDto, LandedCostLineDto,
    ListLandedCostsRequest, ListLandedCostsResponse, PostLandedCostRequest, PostLandedCostResponse,
};
use crate::Result;

/// Service trait for landed cost business logic
///
/// This trait defines all business operations for landed cost management.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait LandedCostService: Send + Sync {
    /// Create a new landed cost document in draft status
    ///
    /// # Business Rules
    /// - Applies tenant isolation
    /// - Creates document in 'draft' status
    /// - Validates GRN exists if provided
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `user_id` - User creating the document
    /// * `request` - Creation request with optional GRN link
    ///
    /// # Returns
    /// Created landed cost document
    async fn create_draft(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateLandedCostRequest,
    ) -> Result<LandedCostDto>;

    /// Add a cost line to an existing landed cost
    ///
    /// # Business Rules
    /// - Document must be in 'draft' status
    /// - Amount must be positive
    /// - Clears any existing allocations (requires recompute)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `landed_cost_id` - Parent landed cost ID
    /// * `request` - Cost line details
    ///
    /// # Returns
    /// Created cost line
    ///
    /// # Errors
    /// - `NotFound` if landed cost doesn't exist
    /// - `ValidationError` if amount is not positive
    /// - `BusinessError` if document is not in draft status
    async fn add_line(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        request: AddLandedCostLineRequest,
    ) -> Result<LandedCostLineDto>;

    /// Compute allocations for all cost lines
    ///
    /// # Business Rules
    /// - Document must be in 'draft' status
    /// - GRN must be linked (directly or via request)
    /// - Total target value must be > 0
    /// - Uses proportional allocation by value (MVP)
    /// - Idempotent: replaces any existing draft allocations
    /// - Ensures allocated total equals cost line total (handles rounding)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `landed_cost_id` - Landed cost to compute allocations for
    /// * `request` - Computation options
    ///
    /// # Returns
    /// Computed allocations summary
    ///
    /// # Errors
    /// - `NotFound` if landed cost doesn't exist
    /// - `ValidationError` if no targets or zero total value
    /// - `BusinessError` if document is not in draft status
    async fn compute_allocations(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        request: ComputeAllocationsRequest,
    ) -> Result<ComputeAllocationsResponse>;

    /// Post a landed cost document
    ///
    /// # Business Rules
    /// - Document must be in 'draft' status
    /// - Allocations must exist
    /// - Creates valuation adjustment entries atomically
    /// - Marks document as 'posted' with timestamp
    /// - Idempotent: posting twice returns success without double-applying
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `user_id` - User posting the document
    /// * `landed_cost_id` - Landed cost to post
    /// * `request` - Post options (idempotency key)
    ///
    /// # Returns
    /// Post result with adjustment count
    ///
    /// # Errors
    /// - `NotFound` if landed cost doesn't exist
    /// - `ValidationError` if no allocations exist
    /// - `BusinessError` if document is not in draft status
    async fn post(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        landed_cost_id: Uuid,
        request: PostLandedCostRequest,
    ) -> Result<PostLandedCostResponse>;

    /// Get a landed cost by ID with all lines and allocations
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `landed_cost_id` - Landed cost ID
    ///
    /// # Returns
    /// Full landed cost details
    ///
    /// # Errors
    /// - `NotFound` if landed cost doesn't exist
    async fn get_by_id(&self, tenant_id: Uuid, landed_cost_id: Uuid)
        -> Result<LandedCostDetailDto>;

    /// List landed costs with filtering
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `request` - Filter and pagination options
    ///
    /// # Returns
    /// Paginated list of landed costs
    async fn list(
        &self,
        tenant_id: Uuid,
        request: ListLandedCostsRequest,
    ) -> Result<ListLandedCostsResponse>;

    /// Cancel a landed cost document
    ///
    /// # Business Rules
    /// - Only draft documents can be cancelled
    /// - Posted documents cannot be cancelled (use reversing entry)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant context
    /// * `landed_cost_id` - Landed cost to cancel
    ///
    /// # Returns
    /// Cancelled landed cost
    ///
    /// # Errors
    /// - `NotFound` if landed cost doesn't exist
    /// - `BusinessError` if document is not in draft status
    async fn cancel(&self, tenant_id: Uuid, landed_cost_id: Uuid) -> Result<LandedCostDto>;
}
