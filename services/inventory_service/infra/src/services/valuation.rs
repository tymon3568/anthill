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
    Valuation, ValuationHistory, ValuationLayer, ValuationMethod,
};
use inventory_service_core::repositories::valuation::{
    ValuationHistoryRepository, ValuationLayerRepository, ValuationRepository,
};
use inventory_service_core::services::valuation::ValuationService;
use inventory_service_core::Result;

/// Implementation of ValuationService
pub struct ValuationServiceImpl {
    valuation_repo: Arc<dyn ValuationRepository>,
    layer_repo: Arc<dyn ValuationLayerRepository>,
    history_repo: Arc<dyn ValuationHistoryRepository>,
}

impl ValuationServiceImpl {
    /// Create new service instance
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
    async fn get_valuation(&self, request: GetValuationRequest) -> Result<ValuationDto> {
        let valuation = self
            .valuation_repo
            .find_by_product_id(request.tenant_id, request.product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        Ok(ValuationDto {
            valuation_id: valuation.valuation_id,
            tenant_id: valuation.tenant_id,
            product_id: valuation.product_id,
            valuation_method: valuation.valuation_method,
            current_unit_cost: valuation.current_unit_cost,
            total_quantity: valuation.total_quantity,
            total_value: valuation.total_value,
            standard_cost: valuation.standard_cost,
            last_updated: valuation.last_updated,
        })
    }

    async fn set_valuation_method(
        &self,
        request: SetValuationMethodRequest,
    ) -> Result<ValuationDto> {
        // Check if valuation exists, create if not
        let existing = self
            .valuation_repo
            .find_by_product_id(request.tenant_id, request.product_id)
            .await?;

        let valuation = if let Some(val) = existing {
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

        // For FIFO, initialize with existing quantity if any
        if matches!(request.valuation_method, ValuationMethod::Fifo) {
            // TODO: Initialize layers from existing stock moves
            // This would require access to stock move history
        }

        Ok(ValuationDto {
            valuation_id: valuation.valuation_id,
            tenant_id: valuation.tenant_id,
            product_id: valuation.product_id,
            valuation_method: valuation.valuation_method,
            current_unit_cost: valuation.current_unit_cost,
            total_quantity: valuation.total_quantity,
            total_value: valuation.total_value,
            standard_cost: valuation.standard_cost,
            last_updated: valuation.last_updated,
        })
    }

    async fn set_standard_cost(&self, request: SetStandardCostRequest) -> Result<ValuationDto> {
        // Validate cost is positive
        if request.standard_cost <= 0 {
            return Err(shared_error::AppError::ValidationError(
                "Standard cost must be positive".to_string(),
            ));
        }

        // Get current valuation
        let valuation = self
            .valuation_repo
            .find_by_product_id(request.tenant_id, request.product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        // Check if using standard costing
        if !matches!(valuation.valuation_method, ValuationMethod::Standard) {
            return Err(shared_error::AppError::BusinessError(
                "Standard cost can only be set for products using Standard costing method"
                    .to_string(),
            ));
        }

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

        Ok(ValuationDto {
            valuation_id: final_valuation.valuation_id,
            tenant_id: final_valuation.tenant_id,
            product_id: final_valuation.product_id,
            valuation_method: final_valuation.valuation_method,
            current_unit_cost: final_valuation.current_unit_cost,
            total_quantity: final_valuation.total_quantity,
            total_value: final_valuation.total_value,
            standard_cost: final_valuation.standard_cost,
            last_updated: final_valuation.last_updated,
        })
    }

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

        Ok(ValuationDto {
            valuation_id: updated.valuation_id,
            tenant_id: updated.tenant_id,
            product_id: updated.product_id,
            valuation_method: updated.valuation_method,
            current_unit_cost: updated.current_unit_cost,
            total_quantity: updated.total_quantity,
            total_value: updated.total_value,
            standard_cost: updated.standard_cost,
            last_updated: updated.last_updated,
        })
    }

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

        Ok(ValuationDto {
            valuation_id: updated.valuation_id,
            tenant_id: updated.tenant_id,
            product_id: updated.product_id,
            valuation_method: updated.valuation_method,
            current_unit_cost: updated.current_unit_cost,
            total_quantity: updated.total_quantity,
            total_value: updated.total_value,
            standard_cost: updated.standard_cost,
            last_updated: updated.last_updated,
        })
    }

    async fn process_stock_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto> {
        // Get current valuation
        let valuation = self
            .valuation_repo
            .find_by_product_id(tenant_id, product_id)
            .await?
            .ok_or_else(|| shared_error::AppError::NotFound("Valuation not found".to_string()))?;

        match valuation.valuation_method {
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
        }
    }

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
    async fn process_fifo_movement(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        quantity_change: i64,
        unit_cost: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<ValuationDto> {
        if quantity_change > 0 {
            // Receipt: create new cost layer
            let unit_cost = unit_cost.ok_or_else(|| {
                shared_error::AppError::ValidationError(
                    "Unit cost required for receipt".to_string(),
                )
            })?;

            let layer = ValuationLayer::new(tenant_id, product_id, quantity_change, unit_cost);
            self.layer_repo.create(&layer).await?;
        } else if quantity_change < 0 {
            // Delivery: consume from layers
            let quantity_to_consume = quantity_change.abs();
            self.layer_repo
                .consume_layers(tenant_id, product_id, quantity_to_consume)
                .await?;
        }

        // Update valuation totals
        let total_quantity = self
            .layer_repo
            .get_total_quantity(tenant_id, product_id)
            .await?;
        let total_value = self
            .calculate_inventory_value(tenant_id, product_id)
            .await?;

        let mut valuation = self
            .valuation_repo
            .find_by_product_id(tenant_id, product_id)
            .await?
            .unwrap();
        valuation.total_quantity = total_quantity;
        valuation.total_value = total_value;

        let updated = self
            .valuation_repo
            .update(tenant_id, product_id, &valuation)
            .await?;

        Ok(ValuationDto {
            valuation_id: updated.valuation_id,
            tenant_id: updated.tenant_id,
            product_id: updated.product_id,
            valuation_method: updated.valuation_method,
            current_unit_cost: updated.current_unit_cost,
            total_quantity: updated.total_quantity,
            total_value: updated.total_value,
            standard_cost: updated.standard_cost,
            last_updated: updated.last_updated,
        })
    }

    /// Process AVCO stock movement
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

        Ok(ValuationDto {
            valuation_id: updated.valuation_id,
            tenant_id: updated.tenant_id,
            product_id: updated.product_id,
            valuation_method: updated.valuation_method,
            current_unit_cost: updated.current_unit_cost,
            total_quantity: updated.total_quantity,
            total_value: updated.total_value,
            standard_cost: updated.standard_cost,
            last_updated: updated.last_updated,
        })
    }

    /// Process Standard costing stock movement
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

        Ok(ValuationDto {
            valuation_id: updated.valuation_id,
            tenant_id: updated.tenant_id,
            product_id: updated.product_id,
            valuation_method: updated.valuation_method,
            current_unit_cost: updated.current_unit_cost,
            total_quantity: updated.total_quantity,
            total_value: updated.total_value,
            standard_cost: updated.standard_cost,
            last_updated: updated.last_updated,
        })
    }
}
