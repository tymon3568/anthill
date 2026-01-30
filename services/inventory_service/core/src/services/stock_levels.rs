//! Stock Levels Service trait
//!
//! This module defines the service trait for listing inventory stock levels
//! with product and warehouse details.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::stock_levels::{StockLevelListQuery, StockLevelListResponse};
use shared_error::AppError;

/// Service for querying stock levels with details
#[async_trait]
pub trait StockLevelsService: Send + Sync {
    /// List stock levels with pagination and filtering
    ///
    /// Returns inventory levels joined with product and warehouse information,
    /// along with summary statistics.
    async fn list_stock_levels(
        &self,
        tenant_id: Uuid,
        query: StockLevelListQuery,
    ) -> Result<StockLevelListResponse, AppError>;
}
