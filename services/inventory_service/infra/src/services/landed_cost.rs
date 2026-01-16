//! Landed Cost Service Implementation
//!
//! PostgreSQL implementation of the LandedCostService trait.
//! Follows the 3-crate pattern: api → infra → core → shared/*

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::landed_cost_dto::{
    AddLandedCostLineRequest, ComputeAllocationsRequest, ComputeAllocationsResponse,
    CreateLandedCostRequest, LandedCostAllocationDto, LandedCostDetailDto, LandedCostDto,
    LandedCostLineDto, ListLandedCostsRequest, ListLandedCostsResponse, PostLandedCostRequest,
    PostLandedCostResponse,
};
use inventory_service_core::domains::inventory::landed_cost::{
    LandedCost, LandedCostAllocation, LandedCostLine, LandedCostStatus, TargetType,
};
use inventory_service_core::services::landed_cost::LandedCostService;
use inventory_service_core::Result;
use shared_error::AppError;

use crate::repositories::landed_cost::{
    AllocationTargetRepository, LandedCostAllocationRepository, LandedCostLineRepository,
    LandedCostRepository, PgLandedCostRepository,
};

/// PostgreSQL implementation of LandedCostService
pub struct PgLandedCostService {
    repo: Arc<PgLandedCostRepository>,
}

impl PgLandedCostService {
    /// Create a new landed cost service instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: Arc::new(PgLandedCostRepository::new(pool)),
        }
    }

    /// Convert domain entity to DTO
    fn to_dto(lc: &LandedCost, total_amount_cents: i64, line_count: i64) -> LandedCostDto {
        LandedCostDto {
            landed_cost_id: lc.landed_cost_id,
            tenant_id: lc.tenant_id,
            reference: lc.reference.clone(),
            status: lc.status,
            grn_id: lc.grn_id,
            notes: lc.notes.clone(),
            posted_at: lc.posted_at,
            posted_by: lc.posted_by,
            created_by: lc.created_by,
            created_at: lc.created_at,
            updated_at: lc.updated_at,
            total_amount_cents,
            line_count,
        }
    }

    /// Convert line entity to DTO
    fn line_to_dto(line: &LandedCostLine) -> LandedCostLineDto {
        LandedCostLineDto {
            landed_cost_line_id: line.landed_cost_line_id,
            landed_cost_id: line.landed_cost_id,
            cost_type: line.cost_type,
            description: line.description.clone(),
            amount_cents: line.amount_cents,
            allocation_method: line.allocation_method,
            created_at: line.created_at,
        }
    }

    /// Convert allocation entity to DTO
    fn allocation_to_dto(alloc: &LandedCostAllocation) -> LandedCostAllocationDto {
        LandedCostAllocationDto {
            landed_cost_allocation_id: alloc.landed_cost_allocation_id,
            landed_cost_id: alloc.landed_cost_id,
            landed_cost_line_id: alloc.landed_cost_line_id,
            target_type: alloc.target_type,
            target_id: alloc.target_id,
            allocated_amount_cents: alloc.allocated_amount_cents,
            created_at: alloc.created_at,
        }
    }

    /// Compute proportional allocation with rounding reconciliation
    ///
    /// Uses the "largest remainder" method to ensure total equals exactly the cost amount:
    /// 1. Calculate proportional allocation for each target
    /// 2. Sum truncated values
    /// 3. Distribute remainder to targets with largest fractional parts
    fn compute_proportional_allocation(
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        line: &LandedCostLine,
        targets: &[(Uuid, i64)], // (target_id, value_cents)
        target_type: TargetType,
    ) -> Result<Vec<LandedCostAllocation>> {
        let total_value: i64 = targets.iter().map(|(_, v)| v).sum();

        if total_value == 0 {
            return Err(AppError::ValidationError(
                "Cannot allocate: total target value is zero".to_string(),
            ));
        }

        let cost_amount = line.amount_cents;

        // Calculate proportional allocations with fractional parts
        let mut allocations_with_fractions: Vec<(Uuid, i64, f64)> = targets
            .iter()
            .map(|(target_id, value)| {
                let proportion = (*value as f64) / (total_value as f64);
                let exact = proportion * (cost_amount as f64);
                let truncated = exact.floor() as i64;
                let fraction = exact - (truncated as f64);
                (*target_id, truncated, fraction)
            })
            .collect();

        // Calculate remainder to distribute
        let allocated_sum: i64 = allocations_with_fractions.iter().map(|(_, a, _)| a).sum();
        let mut remainder = cost_amount - allocated_sum;

        // Sort by fractional part descending to distribute remainder fairly
        allocations_with_fractions
            .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Distribute remainder one cent at a time to targets with largest fractions
        let mut final_allocations: Vec<LandedCostAllocation> = Vec::with_capacity(targets.len());
        for (target_id, mut amount, _) in allocations_with_fractions {
            if remainder > 0 {
                amount += 1;
                remainder -= 1;
            }

            final_allocations.push(LandedCostAllocation::new(
                tenant_id,
                landed_cost_id,
                line.landed_cost_line_id,
                target_type,
                target_id,
                amount,
            ));
        }

        Ok(final_allocations)
    }
}

#[async_trait]
impl LandedCostService for PgLandedCostService {
    async fn create_draft(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateLandedCostRequest,
    ) -> Result<LandedCostDto> {
        let mut landed_cost = LandedCost::new(tenant_id, user_id);
        landed_cost.reference = request.reference;
        landed_cost.grn_id = request.grn_id;
        landed_cost.notes = request.notes;

        LandedCostRepository::insert(&*self.repo, &landed_cost).await?;

        Ok(Self::to_dto(&landed_cost, 0, 0))
    }

    async fn add_line(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        request: AddLandedCostLineRequest,
    ) -> Result<LandedCostLineDto> {
        // Validate landed cost exists and is in draft status
        let landed_cost = self
            .repo
            .find_by_id(tenant_id, landed_cost_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost not found".to_string()))?;

        if !landed_cost.is_modifiable() {
            return Err(AppError::BusinessError("Landed cost is not in draft status".to_string()));
        }

        // Validate amount
        if request.amount_cents <= 0 {
            return Err(AppError::ValidationError("Amount must be positive".to_string()));
        }

        // Create and insert line
        let mut line =
            LandedCostLine::new(tenant_id, landed_cost_id, request.cost_type, request.amount_cents);
        line.description = request.description;
        line.allocation_method = request.allocation_method;

        LandedCostLineRepository::insert(self.repo.as_ref(), &line).await?;

        // Clear existing allocations (require recompute)
        LandedCostAllocationRepository::delete_by_landed_cost_id(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        Ok(Self::line_to_dto(&line))
    }

    async fn compute_allocations(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
        request: ComputeAllocationsRequest,
    ) -> Result<ComputeAllocationsResponse> {
        // Validate landed cost exists and is in draft status
        let landed_cost = self
            .repo
            .find_by_id(tenant_id, landed_cost_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost not found".to_string()))?;

        if !landed_cost.is_modifiable() {
            return Err(AppError::BusinessError("Landed cost is not in draft status".to_string()));
        }

        // Get GRN ID
        let grn_id = landed_cost.grn_id.ok_or_else(|| {
            AppError::ValidationError("Landed cost must be linked to a GRN".to_string())
        })?;

        // Get cost lines
        let lines = LandedCostLineRepository::find_by_landed_cost_id(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        if lines.is_empty() {
            return Err(AppError::ValidationError("No cost lines to allocate".to_string()));
        }

        // Get allocation targets
        let targets =
            AllocationTargetRepository::get_grn_item_targets(self.repo.as_ref(), tenant_id, grn_id)
                .await?;

        if targets.is_empty() {
            return Err(AppError::ValidationError("No targets found for allocation".to_string()));
        }

        // Convert targets to (id, value) tuples
        let target_tuples: Vec<(Uuid, i64)> = targets
            .iter()
            .map(|t| (t.target_id, t.value_cents))
            .collect();

        // Delete existing allocations (idempotent recompute)
        LandedCostAllocationRepository::delete_by_landed_cost_id(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        // Compute allocations for each line
        let mut all_allocations: Vec<LandedCostAllocation> = Vec::new();
        for line in &lines {
            let line_allocations = Self::compute_proportional_allocation(
                tenant_id,
                landed_cost_id,
                line,
                &target_tuples,
                request.target_type,
            )?;
            all_allocations.extend(line_allocations);
        }

        // Insert allocations
        LandedCostAllocationRepository::insert_batch(self.repo.as_ref(), &all_allocations).await?;

        let total_allocated: i64 = all_allocations
            .iter()
            .map(|a| a.allocated_amount_cents)
            .sum();
        let allocation_dtos: Vec<LandedCostAllocationDto> = all_allocations
            .iter()
            .map(Self::allocation_to_dto)
            .collect();

        Ok(ComputeAllocationsResponse {
            landed_cost_id,
            allocations_count: all_allocations.len() as i64,
            total_allocated_cents: total_allocated,
            allocations: allocation_dtos,
        })
    }

    async fn post(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        landed_cost_id: Uuid,
        _request: PostLandedCostRequest,
    ) -> Result<PostLandedCostResponse> {
        // Validate landed cost exists
        let landed_cost = self
            .repo
            .find_by_id(tenant_id, landed_cost_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost not found".to_string()))?;

        // Idempotency: if already posted, return success
        if landed_cost.status == LandedCostStatus::Posted {
            return Ok(PostLandedCostResponse {
                landed_cost_id,
                status: LandedCostStatus::Posted,
                posted_at: landed_cost.posted_at.unwrap_or_else(Utc::now),
                adjustments_created: 0,
            });
        }

        if !landed_cost.can_post() {
            return Err(AppError::BusinessError("Landed cost cannot be posted".to_string()));
        }

        // Validate allocations exist
        let has_allocations = LandedCostAllocationRepository::has_allocations(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        if !has_allocations {
            return Err(AppError::ValidationError(
                "No allocations found. Run compute first.".to_string(),
            ));
        }

        // Update status to posted
        self.repo
            .update_status(tenant_id, landed_cost_id, LandedCostStatus::Posted, Some(user_id))
            .await?;

        // TODO: Create valuation adjustment entries
        // This would integrate with the valuation system to adjust inventory costs
        // For MVP, we mark as posted and the adjustments can be created later
        let adjustments_created = 0;

        Ok(PostLandedCostResponse {
            landed_cost_id,
            status: LandedCostStatus::Posted,
            posted_at: Utc::now(),
            adjustments_created,
        })
    }

    async fn get_by_id(
        &self,
        tenant_id: Uuid,
        landed_cost_id: Uuid,
    ) -> Result<LandedCostDetailDto> {
        let landed_cost = self
            .repo
            .find_by_id(tenant_id, landed_cost_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost not found".to_string()))?;

        let lines = LandedCostLineRepository::find_by_landed_cost_id(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        let allocations = LandedCostAllocationRepository::find_by_landed_cost_id(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        let total_amount: i64 = lines.iter().map(|l| l.amount_cents).sum();

        Ok(LandedCostDetailDto {
            landed_cost: Self::to_dto(&landed_cost, total_amount, lines.len() as i64),
            lines: lines.iter().map(Self::line_to_dto).collect(),
            allocations: allocations.iter().map(Self::allocation_to_dto).collect(),
        })
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        request: ListLandedCostsRequest,
    ) -> Result<ListLandedCostsResponse> {
        let (items, total) = self
            .repo
            .list(tenant_id, request.status, request.grn_id, request.limit, request.offset)
            .await?;

        // Get totals for each item
        let mut dtos = Vec::with_capacity(items.len());
        for item in &items {
            let total_amount = LandedCostLineRepository::get_total_amount(
                self.repo.as_ref(),
                tenant_id,
                item.landed_cost_id,
            )
            .await?;

            let lines = LandedCostLineRepository::find_by_landed_cost_id(
                self.repo.as_ref(),
                tenant_id,
                item.landed_cost_id,
            )
            .await?;

            dtos.push(Self::to_dto(item, total_amount, lines.len() as i64));
        }

        Ok(ListLandedCostsResponse {
            items: dtos,
            total_count: total,
            limit: request.limit,
            offset: request.offset,
        })
    }

    async fn cancel(&self, tenant_id: Uuid, landed_cost_id: Uuid) -> Result<LandedCostDto> {
        let landed_cost = self
            .repo
            .find_by_id(tenant_id, landed_cost_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost not found".to_string()))?;

        if landed_cost.status != LandedCostStatus::Draft {
            return Err(AppError::BusinessError(
                "Only draft landed costs can be cancelled".to_string(),
            ));
        }

        self.repo
            .update_status(tenant_id, landed_cost_id, LandedCostStatus::Cancelled, None)
            .await?;

        let updated = self
            .repo
            .find_by_id(tenant_id, landed_cost_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Landed cost not found".to_string()))?;

        let total_amount = LandedCostLineRepository::get_total_amount(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        let lines = LandedCostLineRepository::find_by_landed_cost_id(
            self.repo.as_ref(),
            tenant_id,
            landed_cost_id,
        )
        .await?;

        Ok(Self::to_dto(&updated, total_amount, lines.len() as i64))
    }
}
