use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::stock_take::{StockTake, StockTakeStatus};
use inventory_service_core::dto::stock_take::{
    CountStockTakeRequest, CountStockTakeResponse, CreateStockTakeRequest, CreateStockTakeResponse,
    FinalizeStockTakeRequest, FinalizeStockTakeResponse, PaginationInfo, StockAdjustment,
    StockTakeDetailResponse, StockTakeListQuery, StockTakeListResponse,
};
use inventory_service_core::models::CreateStockMoveRequest;
use inventory_service_core::repositories::stock::{InventoryLevelRepository, StockMoveRepository};
use inventory_service_core::repositories::stock_take::{
    StockTakeLineRepository, StockTakeRepository,
};
use inventory_service_core::services::stock_take::StockTakeService;
use shared_error::AppError;

/// PostgreSQL implementation of StockTakeService
pub struct PgStockTakeService {
    stock_take_repo: Arc<dyn StockTakeRepository>,
    stock_take_line_repo: Arc<dyn StockTakeLineRepository>,
    stock_move_repo: Arc<dyn StockMoveRepository>,
    inventory_repo: Arc<dyn InventoryLevelRepository>,
}

impl PgStockTakeService {
    /// Create a new service instance
    pub fn new(
        stock_take_repo: Arc<dyn StockTakeRepository>,
        stock_take_line_repo: Arc<dyn StockTakeLineRepository>,
        stock_move_repo: Arc<dyn StockMoveRepository>,
        inventory_repo: Arc<dyn InventoryLevelRepository>,
    ) -> Self {
        Self {
            stock_take_repo,
            stock_take_line_repo,
            stock_move_repo,
            inventory_repo,
        }
    }
}

#[async_trait]
impl StockTakeService for PgStockTakeService {
    async fn create_stock_take(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateStockTakeRequest,
    ) -> Result<CreateStockTakeResponse, AppError> {
        // Generate stock take number (simple increment, in production use sequence)
        let stock_take_number = format!("ST-{}", Utc::now().format("%Y%m%d-%H%M%S"));

        let stock_take = StockTake {
            stock_take_id: Uuid::now_v7(),
            tenant_id,
            stock_take_number,
            warehouse_id: request.warehouse_id,
            status: StockTakeStatus::Draft,
            started_at: Some(Utc::now()),
            completed_at: None,
            created_by: user_id,
            updated_by: Some(user_id),
            notes: request.notes,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            deleted_by: None,
        };

        let created_stock_take = self.stock_take_repo.create(tenant_id, &stock_take).await?;

        // Create stock take lines from current inventory
        let _lines = self
            .stock_take_line_repo
            .create_from_inventory(
                tenant_id,
                created_stock_take.stock_take_id,
                request.warehouse_id,
            )
            .await?;

        Ok(CreateStockTakeResponse {
            stock_take: created_stock_take,
        })
    }

    async fn count_stock_take(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        user_id: Uuid,
        request: CountStockTakeRequest,
    ) -> Result<CountStockTakeResponse, AppError> {
        // Verify stock take exists and is in correct status
        let stock_take = self
            .stock_take_repo
            .find_by_id(tenant_id, stock_take_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Stock take not found".to_string()))?;

        if stock_take.status != StockTakeStatus::Draft
            && stock_take.status != StockTakeStatus::InProgress
        {
            return Err(AppError::ValidationError(
                "Stock take must be in draft or in-progress status to submit counts".to_string(),
            ));
        }

        // Update status to InProgress if it's Draft
        if stock_take.status == StockTakeStatus::Draft {
            self.stock_take_repo
                .update_status(tenant_id, stock_take_id, StockTakeStatus::InProgress, user_id)
                .await?;
        }

        // Batch update counts
        let counts: Vec<(Uuid, i64, Uuid, Option<String>)> = request
            .items
            .into_iter()
            .map(|item| (item.product_id, item.actual_quantity, user_id, item.notes))
            .collect();

        self.stock_take_line_repo
            .batch_update_counts(tenant_id, &counts)
            .await?;

        // Get updated lines
        let lines = self
            .stock_take_line_repo
            .find_by_stock_take_id(tenant_id, stock_take_id)
            .await?;

        Ok(CountStockTakeResponse { lines })
    }

    async fn finalize_stock_take(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        user_id: Uuid,
        _request: FinalizeStockTakeRequest,
    ) -> Result<FinalizeStockTakeResponse, AppError> {
        // Verify stock take exists and is in correct status
        let stock_take = self
            .stock_take_repo
            .find_by_id(tenant_id, stock_take_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Stock take not found".to_string()))?;

        if stock_take.status != StockTakeStatus::InProgress {
            return Err(AppError::ValidationError(
                "Stock take must be in progress to finalize".to_string(),
            ));
        }

        // Get all lines
        let lines = self
            .stock_take_line_repo
            .find_by_stock_take_id(tenant_id, stock_take_id)
            .await?;

        // Check that all lines have been counted
        let uncounted_lines: Vec<_> = lines
            .iter()
            .filter(|line| line.actual_quantity.is_none())
            .collect();

        if !uncounted_lines.is_empty() {
            return Err(AppError::ValidationError(
                "All stock take lines must be counted before finalizing".to_string(),
            ));
        }

        let mut adjustments = Vec::new();

        // Create adjustments for discrepancies
        for line in &lines {
            let actual_quantity = line.actual_quantity.unwrap();
            let difference = actual_quantity - line.expected_quantity;

            if difference != 0 {
                // Create stock move for adjustment
                let stock_move = CreateStockMoveRequest {
                    product_id: line.product_id,
                    source_location_id: Some(stock_take.warehouse_id),
                    destination_location_id: Some(stock_take.warehouse_id), // Same warehouse for adjustment
                    move_type: "adjustment".to_string(),
                    quantity: difference as i64,
                    unit_cost: None,
                    reference_type: "stock_take".to_string(),
                    reference_id: stock_take_id,
                    idempotency_key: format!("st-{}-line-{}", stock_take_id, line.line_id),
                    move_reason: Some(format!(
                        "Stock take {} adjustment",
                        stock_take.stock_take_number
                    )),
                    batch_info: None,
                    metadata: None,
                };
                self.stock_move_repo.create(&stock_move, tenant_id).await?;

                // Update inventory level
                self.inventory_repo
                    .update_available_quantity(tenant_id, line.product_id, difference as i64)
                    .await?;

                adjustments.push(StockAdjustment {
                    adjustment_id: Uuid::now_v7(),
                    product_id: line.product_id,
                    warehouse_id: stock_take.warehouse_id,
                    quantity: difference as i64,
                    reason: "Stock take discrepancy".to_string(),
                    adjusted_at: Utc::now(),
                });
            }
        }

        // Finalize stock take
        let completed_at = Utc::now();
        self.stock_take_repo
            .finalize(tenant_id, stock_take_id, completed_at, user_id)
            .await?;

        // Get updated stock take
        let finalized_stock_take = self
            .stock_take_repo
            .find_by_id(tenant_id, stock_take_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalError("Failed to retrieve finalized stock take".to_string())
            })?;

        Ok(FinalizeStockTakeResponse {
            stock_take: finalized_stock_take,
            adjustments,
        })
    }

    async fn get_stock_take(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
    ) -> Result<StockTakeDetailResponse, AppError> {
        let stock_take = self
            .stock_take_repo
            .find_by_id(tenant_id, stock_take_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Stock take not found".to_string()))?;

        let lines = self
            .stock_take_line_repo
            .find_by_stock_take_id(tenant_id, stock_take_id)
            .await?;

        Ok(StockTakeDetailResponse { stock_take, lines })
    }

    async fn list_stock_takes(
        &self,
        tenant_id: Uuid,
        query: StockTakeListQuery,
    ) -> Result<StockTakeListResponse, AppError> {
        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.page.unwrap_or(1).saturating_sub(1) * limit;

        let stock_takes = self
            .stock_take_repo
            .list(
                tenant_id,
                query.warehouse_id,
                query.status.clone(),
                Some(limit as i64),
                Some(offset as i64),
            )
            .await?;

        let total = self
            .stock_take_repo
            .count(tenant_id, query.warehouse_id, query.status)
            .await?;

        let total_pages = (total as u32).div_ceil(limit);

        Ok(StockTakeListResponse {
            stock_takes,
            pagination: PaginationInfo {
                page: query.page.unwrap_or(1),
                limit,
                total: total as u64,
                total_pages,
            },
        })
    }
}
