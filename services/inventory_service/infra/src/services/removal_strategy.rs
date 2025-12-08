//! Removal strategy service implementation
//!
//! This module provides the concrete implementation of the RemovalStrategyService trait
//! using the repository layer for data access.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::repositories::RemovalStrategyRepositoryImpl;
use inventory_service_core::domains::inventory::removal_strategy::RemovalStrategy;
use inventory_service_core::dto::removal_strategy::{
    RemovalStrategyCreateRequest, RemovalStrategyListQuery, RemovalStrategyListResponse,
    RemovalStrategyResponse, RemovalStrategyUpdateRequest, StrategyAnalyticsResponse,
    SuggestRemovalRequest, SuggestRemovalResponse,
};
use inventory_service_core::services::RemovalStrategyService;
use shared_error::AppError;

pub struct RemovalStrategyServiceImpl {
    repo: RemovalStrategyRepositoryImpl,
}

impl RemovalStrategyServiceImpl {
    pub fn new(repo: RemovalStrategyRepositoryImpl) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RemovalStrategyService for RemovalStrategyServiceImpl {
    async fn create_strategy(
        &self,
        tenant_id: Uuid,
        request: RemovalStrategyCreateRequest,
        created_by: Uuid,
    ) -> Result<RemovalStrategy, AppError> {
        self.repo.create(tenant_id, request, created_by).await
    }

    async fn get_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
    ) -> Result<Option<RemovalStrategy>, AppError> {
        self.repo.find_by_id(tenant_id, strategy_id).await
    }

    async fn get_strategy_by_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<Option<RemovalStrategy>, AppError> {
        self.repo.find_by_name(tenant_id, name).await
    }

    async fn list_strategies(
        &self,
        tenant_id: Uuid,
        query: RemovalStrategyListQuery,
    ) -> Result<RemovalStrategyListResponse, AppError> {
        let (strategies, total_count) = self.repo.list(tenant_id, query.clone()).await?;

        let pagination = inventory_service_core::dto::removal_strategy::PaginationInfo::new(
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
            total_count,
        );

        let responses = strategies
            .into_iter()
            .map(|s| RemovalStrategyResponse {
                strategy_id: s.strategy_id,
                tenant_id: s.tenant_id,
                name: s.name,
                strategy_type: s.strategy_type,
                strategy_type_display: s.strategy_type_display(),
                warehouse_id: s.warehouse_id,
                product_id: s.product_id,
                active: s.active,
                config: s.config,
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect();

        Ok(RemovalStrategyListResponse {
            strategies: responses,
            pagination,
        })
    }

    async fn update_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        request: RemovalStrategyUpdateRequest,
        updated_by: Uuid,
    ) -> Result<RemovalStrategy, AppError> {
        self.repo
            .update(tenant_id, strategy_id, request, updated_by)
            .await
    }

    async fn delete_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<bool, AppError> {
        self.repo.delete(tenant_id, strategy_id, deleted_by).await
    }

    async fn toggle_strategy_active(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        active: bool,
        updated_by: Uuid,
    ) -> Result<RemovalStrategy, AppError> {
        self.repo
            .toggle_active(tenant_id, strategy_id, active, updated_by)
            .await
    }

    async fn suggest_removal(
        &self,
        tenant_id: Uuid,
        request: SuggestRemovalRequest,
    ) -> Result<SuggestRemovalResponse, AppError> {
        self.repo.suggest_removal(tenant_id, request).await
    }

    async fn get_available_stock_locations(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<
        Vec<inventory_service_core::repositories::removal_strategy::StockLocationInfo>,
        AppError,
    > {
        self.repo
            .get_available_stock_locations(tenant_id, warehouse_id, product_id)
            .await
    }

    async fn get_applicable_strategies(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<Vec<RemovalStrategy>, AppError> {
        self.repo
            .find_active_for_scope(tenant_id, warehouse_id, product_id)
            .await
    }

    async fn select_best_strategy(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        strategies: Vec<RemovalStrategy>,
    ) -> Result<(RemovalStrategy, String), AppError> {
        // Simple selection: prefer product-specific, then warehouse-specific, then global
        // In a real implementation, this could be more sophisticated
        if let Some(strategy) = strategies.iter().find(|s| s.product_id.is_some()) {
            Ok((strategy.clone(), "Product-specific strategy".to_string()))
        } else if let Some(strategy) = strategies.iter().find(|s| s.warehouse_id.is_some()) {
            Ok((strategy.clone(), "Warehouse-specific strategy".to_string()))
        } else if let Some(strategy) = strategies.first() {
            Ok((strategy.clone(), "Global strategy".to_string()))
        } else {
            Err(AppError::NotFound("No applicable strategies found".to_string()))
        }
    }

    async fn validate_strategy(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
    ) -> Result<bool, AppError> {
        // Basic validation: check if strategy exists and is active
        if let Some(strategy) = self.repo.find_by_id(tenant_id, strategy_id).await? {
            Ok(strategy.active)
        } else {
            Ok(false)
        }
    }

    async fn record_strategy_usage(
        &self,
        tenant_id: Uuid,
        strategy_id: Uuid,
        product_id: Uuid,
        quantity: i64,
        pick_time_seconds: Option<f64>,
    ) -> Result<bool, AppError> {
        self.repo
            .record_strategy_usage(tenant_id, strategy_id, product_id, quantity, pick_time_seconds)
            .await
    }

    async fn get_strategy_analytics(
        &self,
        tenant_id: Uuid,
        strategy_id: Option<Uuid>,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<StrategyAnalyticsResponse>, AppError> {
        // For now, return empty vec. In a real implementation, you'd query usage data
        // This would require a usage tracking table
        Ok(vec![])
    }
}
