use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::reconciliation::ReconciliationStatus;
use inventory_service_core::dto::reconciliation::{
    ApproveReconciliationRequest, ApproveReconciliationResponse, CountReconciliationRequest,
    CountReconciliationResponse, CreateReconciliationRequest, CreateReconciliationResponse,
    FinalizeReconciliationRequest, FinalizeReconciliationResponse, PaginationInfo,
    ReconciliationAnalyticsResponse, ReconciliationDetailResponse, ReconciliationListQuery,
    ReconciliationListResponse, ScanBarcodeRequest, ScanBarcodeResponse, VarianceAnalysisResponse,
    VarianceRange,
};
use inventory_service_core::dto::stock_take::StockAdjustment;
use inventory_service_core::models::CreateStockMoveRequest;
use inventory_service_core::repositories::reconciliation::{
    ReconciliationItemCountUpdate, StockReconciliationItemRepository, StockReconciliationRepository,
};
use inventory_service_core::services::reconciliation::StockReconciliationService;
use shared_error::AppError;

/// PostgreSQL implementation of ReconciliationService
pub struct PgStockReconciliationService {
    pool: Arc<PgPool>,
    reconciliation_repo: Arc<crate::repositories::reconciliation::PgStockReconciliationRepository>,
    reconciliation_item_repo: Arc<dyn StockReconciliationItemRepository>,
    stock_move_repo: Arc<crate::repositories::stock::PgStockMoveRepository>,
    inventory_repo: Arc<crate::repositories::stock::PgInventoryLevelRepository>,
    product_repo: Arc<dyn inventory_service_core::repositories::product::ProductRepository>,
}

impl PgStockReconciliationService {
    /// Create a new service instance
    pub fn new(
        pool: Arc<PgPool>,
        reconciliation_repo: Arc<
            crate::repositories::reconciliation::PgStockReconciliationRepository,
        >,
        reconciliation_item_repo: Arc<dyn StockReconciliationItemRepository>,
        stock_move_repo: Arc<crate::repositories::stock::PgStockMoveRepository>,
        inventory_repo: Arc<crate::repositories::stock::PgInventoryLevelRepository>,
        product_repo: Arc<dyn inventory_service_core::repositories::product::ProductRepository>,
    ) -> Self {
        Self {
            pool,
            reconciliation_repo,
            reconciliation_item_repo,
            stock_move_repo,
            inventory_repo,
            product_repo,
        }
    }

    /// Convert f64 to BIGINT cents
    fn f64_to_cents(f: f64) -> Result<i64, AppError> {
        const MAX_SAFE: f64 = i64::MAX as f64 / 100.0;
        const MIN_SAFE: f64 = i64::MIN as f64 / 100.0;
        if f > MAX_SAFE || f < MIN_SAFE {
            return Err(AppError::ValidationError(format!(
                "Value {} is out of range for currency conversion",
                f
            )));
        }
        let cents = (f * 100.0).round() as i64;
        Ok(cents)
    }
}

#[async_trait]
impl StockReconciliationService for PgStockReconciliationService {
    async fn create_reconciliation(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateReconciliationRequest,
    ) -> Result<CreateReconciliationResponse, AppError> {
        let reconciliation =
            inventory_service_core::domains::inventory::reconciliation::StockReconciliation {
                reconciliation_id: Uuid::now_v7(),
                tenant_id,
                reconciliation_number: String::new(), // Will be set by trigger
                name: request.name,
                description: request.description,
                status: ReconciliationStatus::Draft,
                cycle_type: request.cycle_type.clone(),
                warehouse_id: request.warehouse_id,
                location_filter: request.location_filter.clone(),
                product_filter: request.product_filter.clone(),
                total_items: 0,    // Will be updated by trigger
                counted_items: 0,  // Will be updated by trigger
                total_variance: 0, // Will be updated by trigger
                created_by: user_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                started_at: None,
                completed_at: None,
                approved_by: None,
                approved_at: None,
                notes: request.notes,
            };

        let created_reconciliation = self
            .reconciliation_repo
            .create(tenant_id, &reconciliation)
            .await?;

        // Create reconciliation items from inventory based on cycle type
        let _items = match self
            .reconciliation_item_repo
            .create_from_inventory(
                tenant_id,
                created_reconciliation.reconciliation_id,
                request.cycle_type.clone(),
                request.warehouse_id,
                request.location_filter,
                request.product_filter,
            )
            .await
        {
            Ok(items) => items,
            Err(e) => {
                if let Err(delete_err) = self
                    .reconciliation_repo
                    .delete(tenant_id, created_reconciliation.reconciliation_id, user_id)
                    .await
                {
                    tracing::warn!(
                        reconciliation_id = %created_reconciliation.reconciliation_id,
                        error = %delete_err,
                        "Failed to cleanup reconciliation after item creation failure"
                    );
                }
                return Err(e);
            },
        };

        // Get updated reconciliation with correct counts
        let final_reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, created_reconciliation.reconciliation_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalError("Failed to retrieve created reconciliation".to_string())
            })?;

        Ok(CreateReconciliationResponse {
            reconciliation: final_reconciliation,
        })
    }

    async fn count_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        user_id: Uuid,
        request: CountReconciliationRequest,
    ) -> Result<CountReconciliationResponse, AppError> {
        // Verify reconciliation exists and is in correct status
        let reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Reconciliation not found".to_string()))?;

        if reconciliation.status != ReconciliationStatus::Draft
            && reconciliation.status != ReconciliationStatus::InProgress
        {
            return Err(AppError::ValidationError(
                "Reconciliation must be in draft or in-progress status to submit counts"
                    .to_string(),
            ));
        }

        // Update status to InProgress if it's Draft
        if reconciliation.status == ReconciliationStatus::Draft {
            self.reconciliation_repo
                .update_status(
                    tenant_id,
                    reconciliation_id,
                    ReconciliationStatus::InProgress,
                    user_id,
                )
                .await?;
        }

        // Build batch update from request
        let counts: Vec<ReconciliationItemCountUpdate> = request
            .items
            .into_iter()
            .map(|item| ReconciliationItemCountUpdate {
                product_id: item.product_id,
                warehouse_id: item.warehouse_id,
                location_id: item.location_id,
                counted_quantity: item.counted_quantity,
                unit_cost: item.unit_cost,
                counted_by: user_id,
                notes: item.notes,
            })
            .collect();

        if counts.is_empty() {
            return Err(AppError::ValidationError("No items to count".to_string()));
        }

        self.reconciliation_item_repo
            .batch_update_counts(tenant_id, reconciliation_id, &counts)
            .await?;

        // Get updated items
        let items = self
            .reconciliation_item_repo
            .find_by_reconciliation_id(tenant_id, reconciliation_id)
            .await?;

        Ok(CountReconciliationResponse { items })
    }

    async fn finalize_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        _user_id: Uuid,
        _request: FinalizeReconciliationRequest,
    ) -> Result<FinalizeReconciliationResponse, AppError> {
        // Verify reconciliation exists and is in correct status
        let reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Reconciliation not found".to_string()))?;

        if reconciliation.status != ReconciliationStatus::InProgress {
            return Err(AppError::ValidationError(
                "Reconciliation must be in progress to finalize".to_string(),
            ));
        }

        // Get variance analysis
        let variance_result = self
            .reconciliation_item_repo
            .get_variance_analysis(tenant_id, reconciliation_id)
            .await?;
        let items = variance_result.items;

        // Check that all items have been counted
        let uncounted_items: Vec<_> = items
            .iter()
            .filter(|item| item.counted_quantity.is_none())
            .collect();

        if !uncounted_items.is_empty() {
            return Err(AppError::ValidationError(
                "All reconciliation items must be counted before finalizing".to_string(),
            ));
        }

        // Prepare all stock moves and inventory updates BEFORE transaction (to avoid borrow checker issues)
        let mut stock_moves_to_create = Vec::new();
        let mut inventory_updates: Vec<(Uuid, Uuid, Uuid, i64)> = Vec::new();
        let mut adjustments = Vec::new();

        // Create adjustments for variances
        for item in &items {
            let counted_quantity = item.counted_quantity.unwrap();
            let variance = counted_quantity - item.expected_quantity;

            if variance != 0 {
                // Skip adjustment if no unit cost available
                let unit_cost = match item.unit_cost {
                    Some(c) => c,
                    None => continue,
                };
                let unit_cost_cents = PgStockReconciliationService::f64_to_cents(unit_cost)?;
                let stock_move = CreateStockMoveRequest {
                    product_id: item.product_id,
                    source_location_id: Some(item.warehouse_id),
                    destination_location_id: Some(item.warehouse_id), // Same warehouse for adjustment
                    move_type: "adjustment".to_string(),
                    quantity: variance,
                    unit_cost: Some(unit_cost_cents),
                    reference_type: "reconciliation".to_string(),
                    reference_id: reconciliation_id,
                    idempotency_key: format!(
                        "rec-{}-item-{}-{}",
                        reconciliation_id, item.product_id, item.warehouse_id
                    ),
                    move_reason: Some(format!(
                        "Reconciliation {} adjustment",
                        reconciliation.reconciliation_number
                    )),
                    batch_info: None,
                    metadata: None,
                };
                stock_moves_to_create.push(stock_move);

                // Prepare inventory update
                inventory_updates.push((tenant_id, item.warehouse_id, item.product_id, variance));

                adjustments.push(StockAdjustment {
                    adjustment_id: Uuid::now_v7(),
                    product_id: item.product_id,
                    warehouse_id: item.warehouse_id,
                    quantity: variance,
                    reason: "Reconciliation discrepancy".to_string(),
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
                current_tx = self
                    .stock_move_repo
                    .create_with_tx(current_tx, stock_move.clone(), tenant_id)
                    .await?;
            }
            current_tx
        };

        // Update all inventory levels in sequence
        let tx = {
            let mut current_tx = tx;
            for (tenant_id_upd, warehouse_id, product_id, variance) in &inventory_updates {
                current_tx = self
                    .inventory_repo
                    .update_available_quantity_with_tx(
                        current_tx,
                        *tenant_id_upd,
                        *warehouse_id,
                        *product_id,
                        *variance,
                    )
                    .await?;
            }
            current_tx
        };

        // All operations done - now finalize and commit
        let finalized_tx = self
            .reconciliation_repo
            .finalize_with_tx(tx, tenant_id, reconciliation_id, completed_at)
            .await?;

        finalized_tx
            .commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        // Transaction committed successfully

        // Get updated reconciliation
        let finalized_reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalError("Failed to retrieve finalized reconciliation".to_string())
            })?;

        Ok(FinalizeReconciliationResponse {
            reconciliation: finalized_reconciliation,
            adjustments,
        })
    }

    async fn approve_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        user_id: Uuid,
        _request: ApproveReconciliationRequest,
    ) -> Result<ApproveReconciliationResponse, AppError> {
        // Verify reconciliation exists and is in correct status
        let reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Reconciliation not found".to_string()))?;

        if reconciliation.status != ReconciliationStatus::Completed {
            return Err(AppError::ValidationError(
                "Reconciliation must be completed to approve".to_string(),
            ));
        }

        let approved_at = Utc::now();
        self.reconciliation_repo
            .approve(tenant_id, reconciliation_id, user_id, approved_at)
            .await?;

        // Get updated reconciliation
        let approved_reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalError("Failed to retrieve approved reconciliation".to_string())
            })?;

        Ok(ApproveReconciliationResponse {
            reconciliation: approved_reconciliation,
        })
    }

    async fn get_reconciliation(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<ReconciliationDetailResponse, AppError> {
        let reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Reconciliation not found".to_string()))?;

        let items = self
            .reconciliation_item_repo
            .find_by_reconciliation_id(tenant_id, reconciliation_id)
            .await?;

        Ok(ReconciliationDetailResponse {
            reconciliation,
            items,
        })
    }

    async fn list_reconciliations(
        &self,
        tenant_id: Uuid,
        query: ReconciliationListQuery,
    ) -> Result<ReconciliationListResponse, AppError> {
        let limit = query.limit.unwrap_or(50).clamp(1, 100) as i64;
        let page = query.page.unwrap_or(1).max(1) as i64;
        let offset = (page - 1) * limit;

        let total = self
            .reconciliation_repo
            .count(tenant_id, query.warehouse_id, query.status.clone(), query.cycle_type.clone())
            .await?;

        let reconciliations = self
            .reconciliation_repo
            .list(
                tenant_id,
                query.warehouse_id,
                query.status,
                query.cycle_type,
                Some(limit as u32),
                Some(offset as u32),
            )
            .await?;

        let total_pages = ((total + limit - 1) / limit).max(1) as u32;

        Ok(ReconciliationListResponse {
            reconciliations,
            pagination: PaginationInfo {
                page: page as u32,
                limit: limit as u32,
                total: total as u64,
                total_pages,
            },
        })
    }

    async fn get_analytics(
        &self,
        _tenant_id: Uuid,
        _warehouse_id: Option<Uuid>,
    ) -> Result<ReconciliationAnalyticsResponse, AppError> {
        // This is a simplified implementation - in practice, you'd aggregate from the database
        // For now, return placeholder values
        Ok(ReconciliationAnalyticsResponse {
            total_reconciliations: 0,
            completed_reconciliations: 0,
            average_variance_percentage: None,
            total_variance_value: None,
            high_variance_items: 0,
            accuracy_rate: None,
        })
    }

    async fn get_variance_analysis(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<VarianceAnalysisResponse, AppError> {
        let reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Reconciliation not found".to_string()))?;

        let variance_result = self
            .reconciliation_item_repo
            .get_variance_analysis(tenant_id, reconciliation_id)
            .await?;
        let items = variance_result.items;

        // Calculate actual variance ranges
        let mut range_0_1 = VarianceRange {
            range: "0-1%".to_string(),
            count: 0,
            total_variance_value: Some(0.0),
        };
        let mut range_1_5 = VarianceRange {
            range: "1-5%".to_string(),
            count: 0,
            total_variance_value: Some(0.0),
        };
        let mut range_5_10 = VarianceRange {
            range: "5-10%".to_string(),
            count: 0,
            total_variance_value: Some(0.0),
        };
        let mut range_over_10 = VarianceRange {
            range: ">10%".to_string(),
            count: 0,
            total_variance_value: Some(0.0),
        };

        for item in &items {
            if let Some(variance_pct) = item.variance_percentage {
                let abs_pct = variance_pct.abs();
                let variance_value = item.variance_value.unwrap_or(0.0);

                if abs_pct <= 0.01 {
                    range_0_1.count += 1;
                    range_0_1.total_variance_value =
                        Some(range_0_1.total_variance_value.unwrap() + variance_value);
                } else if abs_pct <= 0.05 {
                    range_1_5.count += 1;
                    range_1_5.total_variance_value =
                        Some(range_1_5.total_variance_value.unwrap() + variance_value);
                } else if abs_pct <= 0.10 {
                    range_5_10.count += 1;
                    range_5_10.total_variance_value =
                        Some(range_5_10.total_variance_value.unwrap() + variance_value);
                } else {
                    range_over_10.count += 1;
                    range_over_10.total_variance_value =
                        Some(range_over_10.total_variance_value.unwrap() + variance_value);
                }
            }
        }

        let variance_ranges = vec![range_0_1, range_1_5, range_5_10, range_over_10];

        let top_variance_items = items
            .into_iter()
            .filter(|item| item.variance.is_some() && item.variance.unwrap().abs() > 0)
            .take(10)
            .collect();

        Ok(VarianceAnalysisResponse {
            reconciliation,
            variance_ranges,
            top_variance_items,
        })
    }

    async fn scan_barcode(
        &self,
        tenant_id: Uuid,
        reconciliation_id: Uuid,
        user_id: Uuid,
        request: ScanBarcodeRequest,
    ) -> Result<ScanBarcodeResponse, AppError> {
        // Validate reconciliation exists and is in correct status
        let reconciliation = self
            .reconciliation_repo
            .find_by_id(tenant_id, reconciliation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Reconciliation not found".to_string()))?;

        if reconciliation.status != ReconciliationStatus::Draft
            && reconciliation.status != ReconciliationStatus::InProgress
        {
            return Err(AppError::ValidationError(
                "Reconciliation must be in Draft or InProgress status to scan barcodes".to_string(),
            ));
        }

        // Look up product by barcode
        let product = self
            .product_repo
            .find_by_barcode(tenant_id, &request.barcode)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found for barcode".to_string()))?;

        let product_id = product.product_id;

        // Find reconciliation item for this product
        let items = self
            .reconciliation_item_repo
            .find_by_reconciliation_id(tenant_id, reconciliation_id)
            .await?;

        let item = items
            .into_iter()
            .find(|item| item.product_id == product_id)
            .ok_or_else(|| AppError::NotFound("Product not found in reconciliation".to_string()))?;

        // Check if this item already has a count
        let is_new_count = item.counted_quantity.is_none();

        // Update the count
        let update_request = vec![ReconciliationItemCountUpdate {
            product_id: item.product_id,
            warehouse_id: item.warehouse_id,
            location_id: request.location_id.or(item.location_id),
            counted_quantity: request.quantity,
            unit_cost: None, // Unit cost not provided in barcode scan
            counted_by: user_id,
            notes: request.notes.clone(),
        }];

        self.reconciliation_item_repo
            .batch_update_counts(tenant_id, reconciliation_id, &update_request)
            .await?;

        // If reconciliation was in Draft status, transition to InProgress
        if reconciliation.status == ReconciliationStatus::Draft {
            self.reconciliation_repo
                .update_status(
                    tenant_id,
                    reconciliation_id,
                    ReconciliationStatus::InProgress,
                    user_id,
                )
                .await?;
        }

        // Get the updated item
        let updated_item = self
            .reconciliation_item_repo
            .find_by_key(tenant_id, reconciliation_id, item.product_id, item.warehouse_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalError("Failed to retrieve updated item".to_string())
            })?;

        Ok(ScanBarcodeResponse {
            item: updated_item,
            is_new_count,
        })
    }
}
