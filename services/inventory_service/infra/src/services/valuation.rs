//! Valuation service implementation
//!
//! Business logic implementation for inventory valuation operations.
//! Supports FIFO, AVCO, and Standard costing methods with cost layer management.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::valuation_dto::{
    CostAdjustmentRequest, GetValuationHistoryRequest, GetValuationLayersRequest,
    GetValuationRequest, RevaluationRequest, SetStandardCostRequest, SetValuationMethodRequest,
    ValuationDto, ValuationHistoryDto, ValuationHistoryResponse, ValuationLayersResponse,
};
use inventory_service_core::domains::inventory::valuation::{
    Valuation, ValuationHistory, ValuationMethod,
};
use inventory_service_core::repositories::valuation::{
    ValuationHistoryRepository, ValuationLayerRepository, ValuationRepository,
};
use inventory_service_core::services::valuation::ValuationService;
use inventory_service_core::Result;

/// Implementation of ValuationService
///
/// Provides business logic for inventory valuation operations including:
/// - Valuation method management (FIFO, AVCO, Standard)
/// - Cost layer management for FIFO costing
/// - Stock movement processing with automatic cost calculation
/// - Cost adjustments and revaluations with audit trails
/// - Historical valuation tracking and reporting
pub struct ValuationServiceImpl {
    /// Repository for core valuation operations
    valuation_repo: Arc<dyn ValuationRepository>,
    /// Repository for FIFO cost layer management
    layer_repo: Arc<dyn ValuationLayerRepository>,
    /// Repository for valuation history and audit trails
    history_repo: Arc<dyn ValuationHistoryRepository>,
}

impl ValuationServiceImpl {
    /// Create new service instance
    ///
    /// # Arguments
    /// * `valuation_repo` - Repository for valuation operations
    /// * `layer_repo` - Repository for cost layer management
    /// * `history_repo` - Repository for historical tracking
    ///
    /// # Returns
    /// New ValuationServiceImpl instance
    pub fn new(
        valuation_repo: Arc<dyn ValuationRepository>,
        layer_repo: Arc<dyn ValuationLayerRepository>,
        history_repo: Arc<dyn ValuationHistoryRepository>,
    ) -> Self {
        Self {
            valuation_repo,
            layer_repo,
            history_repo,
        }
    }
}

#[async_trait]
impl ValuationService for ValuationServiceImpl {
    /// Get current valuation for a product
    ///
    /// # Arguments
    /// * `request` - Request containing tenant_id and product_id
    ///
    /// # Returns
    /// Current valuation data as DTO
    async fn get_valuation(&self, request: GetValuationRequest) -> Result<ValuationDto> {
        let valuation = self
            .valuation_repo
            .find_by_product_id(request.tenant_id, request.product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        Ok(self.valuation_to_dto(valuation))
    }

    /// Set the valuation method for a product
    ///
    /// Creates valuation record if it doesn't exist.
    /// For FIFO method, initializes cost layers from existing stock.
    ///
    /// # Arguments
    /// * `request` - Request with tenant_id, product_id, and new valuation method
    ///
    /// # Returns
    /// Updated valuation data as DTO
    async fn set_valuation_method(
        &self,
        request: SetValuationMethodRequest,
    ) -> Result<ValuationDto> {
        // Check if valuation exists, create if not
        let existing = self
            .valuation_repo
            .find_by_product_id(request.tenant_id, request.product_id)
            .await?;

        // Reject changing to FIFO if product has existing inventory
        if matches!(request.valuation_method, ValuationMethod::Fifo) {
            if let Some(ref val) = existing {
                if val.total_quantity > 0 {
                    return Err(shared_error::AppError::BusinessError(
                        "Cannot change to FIFO valuation method when product has existing inventory. \
                         FIFO requires cost layer initialization from stock move history.".to_string(),
                    ));
                }
            }
        }

        let valuation = if let Some(_val) = existing {
            // Update existing
            self.valuation_repo
                .set_valuation_method(
                    request.tenant_id,
                    request.product_id,
                    request.valuation_method.clone(),
                    None, // TODO: get from auth context
                )
                .await?
        } else {
            // Create new valuation
            let new_valuation = Valuation::new(
                request.tenant_id,
                request.product_id,
                request.valuation_method.clone(),
            );
            self.valuation_repo.create(&new_valuation).await?
        };

        Ok(self.valuation_to_dto(valuation))
    }

    /// Set the standard cost for a product
    ///
    /// Only allowed for products using Standard costing method.
    /// Recalculates total value and creates history record.
    ///
    /// # Arguments
    /// * `request` - Request with tenant_id, product_id, and new standard cost
    ///
    /// # Returns
    /// Updated valuation data as DTO
    async fn set_standard_cost(&self, request: SetStandardCostRequest) -> Result<ValuationDto> {
        // Validate cost is positive
        if request.standard_cost <= 0 {
            return Err(shared_error::AppError::ValidationError(
                "Standard cost must be positive".to_string(),
            ));
        }

        // Get current valuation (pre-change state)
        let pre_change_valuation = self
            .valuation_repo
            .find_by_product_id(request.tenant_id, request.product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        // Check if using standard costing
        if !matches!(pre_change_valuation.valuation_method, ValuationMethod::Standard) {
            return Err(shared_error::AppError::BusinessError(
                "Standard cost can only be set for products using Standard costing method"
                    .to_string(),
            ));
        }

        // Create history record with pre-change state
        let history = ValuationHistory::new(
            pre_change_valuation.valuation_id,
            pre_change_valuation.tenant_id,
            pre_change_valuation.product_id,
            pre_change_valuation.valuation_method.clone(),
            pre_change_valuation.current_unit_cost,
            pre_change_valuation.total_quantity,
            pre_change_valuation.total_value,
            pre_change_valuation.standard_cost,
            None, // TODO: get from auth context
            Some(format!("Standard cost updated to {}", request.standard_cost)),
        );
        self.history_repo.create(&history).await?;

        let updated = self
            .valuation_repo
            .set_standard_cost(
                request.tenant_id,
                request.product_id,
                request.standard_cost,
                None, // TODO: get from auth context
            )
            .await?;

        // Recalculate total value based on new standard cost
        let new_total_value = updated.total_quantity * request.standard_cost;
        let mut final_valuation = updated.clone();
        final_valuation.total_value = new_total_value;

        self.valuation_repo
            .update(request.tenant_id, request.product_id, &final_valuation)
            .await?;

        Ok(self.valuation_to_dto(final_valuation))
    }

    /// Get all active cost layers for a product
    ///
    /// Returns layers with remaining quantity > 0, ordered by creation time.
    ///
    /// # Arguments
    /// * `request` - Request with tenant_id and product_id
    ///
    /// # Returns
    /// Response containing vector of valuation layers
    async fn get_valuation_layers(
        &self,
        request: GetValuationLayersRequest,
    ) -> Result<ValuationLayersResponse> {
        let layers = self
            .layer_repo
            .find_active_by_product_id(request.tenant_id, request.product_id)
            .await?;

        let layer_dtos = layers
            .into_iter()
            .map(|layer| {
                inventory_service_core::domains::inventory::dto::valuation_dto::ValuationLayerDto {
                    layer_id: layer.layer_id,
                    tenant_id: layer.tenant_id,
                    product_id: layer.product_id,
                    quantity: layer.quantity,
                    unit_cost: layer.unit_cost,
                    total_value: layer.total_value,
                    created_at: layer.created_at,
                }
            })
            .collect();

        Ok(ValuationLayersResponse { layers: layer_dtos })
    }

    /// Get valuation history for a product
    ///
    /// Returns historical snapshots with pagination support.
    ///
    /// # Arguments
    /// * `request` - Request with tenant_id, product_id, limit, and offset
    ///
    /// # Returns
    /// Response containing history records and total count
    async fn get_valuation_history(
        &self,
        request: GetValuationHistoryRequest,
    ) -> Result<ValuationHistoryResponse> {
        let history = self
            .history_repo
            .find_by_product_id(
                request.tenant_id,
                request.product_id,
                request.limit,
                request.offset,
            )
            .await?;

        let total_count = self
            .history_repo
            .count_by_product_id(request.tenant_id, request.product_id)
            .await?;

        let history_dtos = history
            .into_iter()
            .map(|h| ValuationHistoryDto {
                history_id: h.history_id,
                valuation_id: h.valuation_id,
                tenant_id: h.tenant_id,
                product_id: h.product_id,
                valuation_method: h.valuation_method,
                unit_cost: h.unit_cost,
                total_quantity: h.total_quantity,
                total_value: h.total_value,
                standard_cost: h.standard_cost,
                changed_at: h.changed_at,
                change_reason: h.change_reason,
            })
            .collect();

        Ok(ValuationHistoryResponse {
            history: history_dtos,
            total_count,
        })
    }

    /// Adjust inventory cost without changing quantity
    ///
    /// Adds/subtracts from total value and creates audit trail.
    ///
    /// # Arguments
    /// * `request` - Request with tenant_id, product_id, adjustment amount, and reason
    ///
    /// # Returns
    /// Updated valuation data as DTO
    async fn adjust_cost(&self, request: CostAdjustmentRequest) -> Result<ValuationDto> {
        let updated = self
            .valuation_repo
            .adjust_cost(
                request.tenant_id,
                request.product_id,
                request.adjustment_amount,
                &request.reason,
                None, // TODO: get from auth context
            )
            .await?;

        Ok(self.valuation_to_dto(updated))
    }

    /// Revalue entire inventory at new unit cost
    ///
    /// Recalculates total value based on current quantity and new cost.
    /// Creates audit trail entry.
    ///
    /// # Arguments
    /// * `request` - Request with tenant_id, product_id, new unit cost, and reason
    ///
    /// # Returns
    /// Updated valuation data as DTO
    async fn revalue_inventory(&self, request: RevaluationRequest) -> Result<ValuationDto> {
        // Validate new cost is positive
        if request.new_unit_cost <= 0 {
            return Err(shared_error::AppError::ValidationError(
                "New unit cost must be positive".to_string(),
            ));
        }

        let updated = self
            .valuation_repo
            .revalue_inventory(
                request.tenant_id,
                request.product_id,
                request.new_unit_cost,
                &request.reason,
                None, // TODO: get from auth context
            )
            .await?;

        Ok(self.valuation_to_dto(updated))
    }

    /// Process stock movement and update valuation
    ///
    /// Handles receipts and deliveries for all valuation methods.
    /// Creates history record with pre-change state.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Positive for receipts, negative for deliveries
    /// * `unit_cost` - Cost per unit (required for receipts)
    /// * `user_id` - User who initiated the movement
    ///
    /// # Returns
    /// Updated valuation data as DTO
    async fn process_stock_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto> {
        // Get current valuation (pre-change state)
        let pre_change_valuation = self
            .valuation_repo
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        let result = match pre_change_valuation.valuation_method {
            ValuationMethod::Fifo => {
                self.process_fifo_movement(
                    tenant_id,
                    product_id,
                    quantity_change,
                    unit_cost,
                    user_id,
                )
                .await
            },
            ValuationMethod::Avco => {
                self.process_avco_movement(
                    tenant_id,
                    product_id,
                    quantity_change,
                    unit_cost,
                    user_id,
                )
                .await
            },
            ValuationMethod::Standard => {
                self.process_standard_movement(tenant_id, product_id, quantity_change, user_id)
                    .await
            },
        };

        // Create history record with pre-change state
        if let Ok(_dto) = &result {
            let history = ValuationHistory::new(
                pre_change_valuation.valuation_id,
                pre_change_valuation.tenant_id,
                pre_change_valuation.product_id,
                pre_change_valuation.valuation_method.clone(),
                pre_change_valuation.current_unit_cost,
                pre_change_valuation.total_quantity,
                pre_change_valuation.total_value,
                pre_change_valuation.standard_cost,
                user_id,
                Some(format!("Stock movement: {} units", quantity_change)),
            );
            // Log history creation failures for audit trail monitoring
            if let Err(e) = self.history_repo.create(&history).await {
                tracing::error!("Failed to create history record: {:?}", e);
                // Consider: should this fail the operation for compliance?
            }
        }

        result
    }

    /// Calculate current inventory value
    ///
    /// For FIFO: sums all active layer values
    /// For AVCO/Standard: returns stored total_value
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Current inventory value in cents
    async fn calculate_inventory_value(&self, tenant_id: Uuid, product_id: Uuid) -> Result<i64> {
        let valuation = self
            .valuation_repo
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        match valuation.valuation_method {
            ValuationMethod::Fifo => {
                // Sum of all active layer values
                let layers = self
                    .layer_repo
                    .find_active_by_product_id(tenant_id, product_id)
                    .await?;
                Ok(layers.iter().map(|l| l.total_value).sum())
            },
            ValuationMethod::Avco | ValuationMethod::Standard => {
                // Use stored total_value
                Ok(valuation.total_value)
            },
        }
    }

    /// Get the current valuation method for a product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    ///
    /// # Returns
    /// Current valuation method
    async fn get_valuation_method(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<ValuationMethod> {
        let valuation = self
            .valuation_repo
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        Ok(valuation.valuation_method)
    }
}

impl ValuationServiceImpl {
    /// Process FIFO stock movement
    ///
    /// Creates new layers for receipts, consumes existing layers for deliveries.
    /// Updates valuation totals accordingly.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Movement quantity
    /// * `unit_cost` - Unit cost for receipts
    /// * `user_id` - User ID for attribution
    ///
    /// # Returns
    /// Updated valuation DTO
    async fn process_fifo_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto> {
        let updated = self
            .valuation_repo
            .update_from_stock_move(tenant_id, product_id, quantity_change, unit_cost, user_id)
            .await?;

        Ok(self.valuation_to_dto(updated))
    }

    /// Process AVCO stock movement
    ///
    /// Recalculates weighted average cost on receipts.
    /// Uses current average for deliveries.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Movement quantity
    /// * `unit_cost` - Unit cost for receipts
    /// * `user_id` - User ID for attribution
    ///
    /// # Returns
    /// Updated valuation DTO
    async fn process_avco_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto> {
        let updated = self
            .valuation_repo
            .update_from_stock_move(tenant_id, product_id, quantity_change, unit_cost, user_id)
            .await?;

        Ok(self.valuation_to_dto(updated))
    }

    /// Process Standard costing stock movement
    ///
    /// Updates quantity and recalculates value using standard cost.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `product_id` - Product identifier
    /// * `quantity_change` - Movement quantity
    /// * `user_id` - User ID for attribution
    ///
    /// # Returns
    /// Updated valuation DTO
    async fn process_standard_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto> {
        let updated = self
            .valuation_repo
            .update_from_stock_move(tenant_id, product_id, quantity_change, None, user_id)
            .await?;

        Ok(self.valuation_to_dto(updated))
    }

    /// Convert Valuation entity to ValuationDto
    ///
    /// # Arguments
    /// * `valuation` - Valuation entity
    ///
    /// # Returns
    /// ValuationDto for API responses
    fn valuation_to_dto(&self, valuation: Valuation) -> ValuationDto {
        ValuationDto {
            valuation_id: valuation.valuation_id,
            tenant_id: valuation.tenant_id,
            product_id: valuation.product_id,
            valuation_method: valuation.valuation_method,
            current_unit_cost: valuation.current_unit_cost,
            total_quantity: valuation.total_quantity,
            total_value: valuation.total_value,
            standard_cost: valuation.standard_cost,
            last_updated: valuation.last_updated,
        }
    }
}
