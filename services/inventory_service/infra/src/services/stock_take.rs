use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::stock_take::{StockTake, StockTakeStatus};
use inventory_service_core::dto::common::PaginationInfo;
use inventory_service_core::dto::stock_take::{
    CountStockTakeRequest, CountStockTakeResponse, CreateStockTakeRequest, CreateStockTakeResponse,
    FinalizeStockTakeRequest, FinalizeStockTakeResponse, StockAdjustment, StockTakeDetailResponse,
    StockTakeListQuery, StockTakeListResponse,
};
use inventory_service_core::models::CreateStockMoveRequest;

use inventory_service_core::repositories::product::ProductRepository;
use inventory_service_core::repositories::stock::InventoryLevelRepository;
use inventory_service_core::repositories::stock_take::{
    StockTakeLineCountUpdate, StockTakeLineRepository, StockTakeRepository,
};
use inventory_service_core::services::stock_take::StockTakeService;
use shared_error::AppError;

/// PostgreSQL implementation of StockTakeService
pub struct PgStockTakeService {
    pool: Arc<PgPool>,
    stock_take_repo: Arc<crate::repositories::stock_take::PgStockTakeRepository>,
    stock_take_line_repo: Arc<crate::repositories::stock_take::PgStockTakeLineRepository>,
    stock_move_repo: Arc<crate::repositories::stock::PgStockMoveRepository>,
    inventory_repo: Arc<crate::repositories::stock::PgInventoryLevelRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl PgStockTakeService {
    /// Create a new service instance
    pub fn new(
        pool: Arc<PgPool>,
        stock_take_repo: Arc<crate::repositories::stock_take::PgStockTakeRepository>,
        stock_take_line_repo: Arc<crate::repositories::stock_take::PgStockTakeLineRepository>,
        stock_move_repo: Arc<crate::repositories::stock::PgStockMoveRepository>,
        inventory_repo: Arc<crate::repositories::stock::PgInventoryLevelRepository>,
        product_repo: Arc<dyn ProductRepository>,
    ) -> Self {
        Self {
            pool,
            stock_take_repo,
            stock_take_line_repo,
            stock_move_repo,
            inventory_repo,
            product_repo,
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
        // P1: Warehouse-level locking - check for active stock take in this warehouse
        if let Some(active_stock_take) = self
            .stock_take_repo
            .has_active_stock_take(tenant_id, request.warehouse_id)
            .await?
        {
            return Err(AppError::ValidationError(format!(
                "Cannot create stock take: warehouse already has an active stock take '{}' in progress. \
                Please complete or cancel it before starting a new one.",
                active_stock_take.stock_take_number
            )));
        }

        // TODO: Replace with sequence-based generator in production to prevent collisions under concurrent load
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

        // Create stock take lines from current inventory; on failure, clean up the stock_take
        let _lines = match self
            .stock_take_line_repo
            .create_from_inventory(
                tenant_id,
                created_stock_take.stock_take_id,
                request.warehouse_id,
            )
            .await
        {
            Ok(lines) => lines,
            Err(e) => {
                let _ = self
                    .stock_take_repo
                    .delete(tenant_id, created_stock_take.stock_take_id, user_id)
                    .await;
                return Err(e);
            },
        };

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

        // Load valid line_ids for this stock_take
        let existing_lines = self
            .stock_take_line_repo
            .find_by_stock_take_id(tenant_id, stock_take_id)
            .await?;
        let valid_ids: std::collections::HashSet<Uuid> =
            existing_lines.iter().map(|l| l.line_id).collect();

        // Build batch only for lines belonging to this stock_take
        let counts: Vec<StockTakeLineCountUpdate> = request
            .items
            .into_iter()
            .map(|item| StockTakeLineCountUpdate {
                line_id: item.line_id,
                actual_quantity: item.actual_quantity,
                counted_by: user_id,
                notes: item.notes,
            })
            .filter(|count| valid_ids.contains(&count.line_id))
            .collect();

        if counts.is_empty() {
            return Err(AppError::ValidationError(
                "No valid lines to update for this stock take".to_string(),
            ));
        }

        self.stock_take_line_repo
            .batch_update_counts(tenant_id, stock_take_id, &counts)
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

        // P1: Product existence validation - ensure all products still exist
        let product_ids: Vec<Uuid> = lines.iter().map(|l| l.product_id).collect();
        let existing_products = self
            .product_repo
            .find_by_ids(tenant_id, &product_ids)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to validate products: {}", e)))?;

        let existing_product_ids: std::collections::HashSet<Uuid> =
            existing_products.iter().map(|p| p.product_id).collect();

        let missing_products: Vec<Uuid> = product_ids
            .iter()
            .filter(|id| !existing_product_ids.contains(id))
            .cloned()
            .collect();

        if !missing_products.is_empty() {
            return Err(AppError::ValidationError(format!(
                "Cannot finalize stock take: {} product(s) no longer exist. Missing IDs: {}",
                missing_products.len(),
                missing_products
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }

        // P0: Inventory floor check - validate no negative stock will result
        // Get current inventory levels for all products in this stock take
        let current_inventory = self
            .inventory_repo
            .find_by_products(tenant_id, stock_take.warehouse_id, &product_ids)
            .await?;

        // Check each line that would result in a negative adjustment
        let mut insufficient_stock_errors: Vec<String> = Vec::new();
        for line in &lines {
            let actual_quantity = line.actual_quantity.unwrap();
            let difference = actual_quantity - line.expected_quantity;

            // Only check negative adjustments (reducing stock)
            if difference < 0 {
                let current_available = current_inventory
                    .get(&line.product_id)
                    .map(|il| il.available_quantity)
                    .unwrap_or(0);

                // Check if applying this adjustment would result in negative stock
                let projected_quantity = current_available + difference;
                if projected_quantity < 0 {
                    insufficient_stock_errors.push(format!(
                        "Product {} would have negative stock: current={}, adjustment={}, projected={}",
                        line.product_id, current_available, difference, projected_quantity
                    ));
                }
            }
        }

        if !insufficient_stock_errors.is_empty() {
            return Err(AppError::ValidationError(format!(
                "Cannot finalize stock take: insufficient stock for {} product(s). Details: {}",
                insufficient_stock_errors.len(),
                insufficient_stock_errors.join("; ")
            )));
        }

        // Prepare adjustment data BEFORE transaction starts (to avoid borrow checker issues)
        let mut stock_moves_to_create = Vec::new();
        let mut inventory_updates: Vec<(Uuid, Uuid, Uuid, i64)> = Vec::new();
        let mut adjustments = Vec::new();

        for line in &lines {
            let actual_quantity = line.actual_quantity.unwrap();
            let difference = actual_quantity - line.expected_quantity;

            if difference != 0 {
                // Prepare stock move for adjustment
                // For stock take adjustments, both source and destination are NULL
                // This represents a warehouse-level inventory correction (not a location transfer)
                let stock_move = CreateStockMoveRequest {
                    product_id: line.product_id,
                    source_location_id: None,
                    destination_location_id: None,
                    move_type: "adjustment".to_string(),
                    quantity: difference,
                    unit_cost: None,
                    reference_type: "adjustment".to_string(),
                    reference_id: stock_take_id,
                    idempotency_key: format!("st-{}-line-{}", stock_take_id, line.line_id),
                    move_reason: Some(format!(
                        "Stock take {} adjustment",
                        stock_take.stock_take_number
                    )),
                    lot_serial_id: None, // TODO: Set if lot-tracked product
                    batch_info: None,
                    metadata: None,
                };
                stock_moves_to_create.push(stock_move);

                // Prepare inventory update
                inventory_updates.push((
                    tenant_id,
                    stock_take.warehouse_id,
                    line.product_id,
                    difference,
                ));

                adjustments.push(StockAdjustment {
                    adjustment_id: Uuid::now_v7(),
                    product_id: line.product_id,
                    warehouse_id: stock_take.warehouse_id,
                    quantity: difference,
                    reason: "Stock take discrepancy".to_string(),
                    adjusted_at: Utc::now(),
                });
            }
        }

        // Execute all operations within a single transaction scope
        let completed_at = Utc::now();

        let tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        // Create all stock moves in sequence
        let tx = {
            let mut current_tx = tx;
            for stock_move in &stock_moves_to_create {
                let (_move_id, new_tx) = self
                    .stock_move_repo
                    .create_with_tx(current_tx, stock_move.clone(), tenant_id)
                    .await?;
                current_tx = new_tx;
            }
            current_tx
        };

        // Update all inventory levels in sequence
        let tx = {
            let mut current_tx = tx;
            for (tenant_id_upd, warehouse_id, product_id, difference) in &inventory_updates {
                current_tx = self
                    .inventory_repo
                    .update_available_quantity_with_tx(
                        current_tx,
                        *tenant_id_upd,
                        *warehouse_id,
                        *product_id,
                        *difference,
                    )
                    .await?;
            }
            current_tx
        };

        // All borrowing operations done - now finalize and commit
        let finalized_tx = self
            .stock_take_repo
            .finalize_with_tx(tx, tenant_id, stock_take_id, completed_at, user_id)
            .await?;

        finalized_tx
            .commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        // Get updated stock take (after commit)
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
        let limit = query.limit.unwrap_or(50).clamp(1, 100) as i64;
        let page = query.page.unwrap_or(1).max(1) as i64;
        let offset = (page - 1) * limit;

        let stock_takes = self
            .stock_take_repo
            .list(tenant_id, query.warehouse_id, query.status.clone(), Some(limit), Some(offset))
            .await?;

        let total = self
            .stock_take_repo
            .count(tenant_id, query.warehouse_id, query.status)
            .await?;

        let total_pages = ((total + limit - 1) / limit).max(1) as u32;

        Ok(StockTakeListResponse {
            stock_takes,
            pagination: PaginationInfo {
                page: page as u32,
                page_size: limit as u32,
                total_items: total as u64,
                total_pages,
                has_next: page < total_pages as i64,
                has_prev: page > 1,
            },
        })
    }
}
