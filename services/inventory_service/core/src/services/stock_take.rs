//! Stock Take service trait
//!
//! This module defines the service trait for stock take operations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::stock_take::{
    CountStockTakeRequest, CountStockTakeResponse, CreateStockTakeRequest, CreateStockTakeResponse,
    FinalizeStockTakeRequest, FinalizeStockTakeResponse, StockTakeDetailResponse,
    StockTakeListQuery, StockTakeListResponse,
};
use shared_error::AppError;

/// Service trait for stock take operations
#[async_trait]
pub trait StockTakeService: Send + Sync {
    /// Create a new stock take session
    ///
    /// Creates a stock take in draft status, snapshots current inventory levels
    /// for all products in the warehouse, and creates stock take lines.
    async fn create_stock_take(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        request: CreateStockTakeRequest,
    ) -> Result<CreateStockTakeResponse, AppError>;

    /// Submit counted quantities for stock take lines
    ///
    /// Updates actual quantities for the specified products, calculates differences,
    /// and updates the stock take status if all items are counted.
    async fn count_stock_take(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        user_id: Uuid,
        request: CountStockTakeRequest,
    ) -> Result<CountStockTakeResponse, AppError>;

    /// Finalize the stock take and generate inventory adjustments
    ///
    /// Marks the stock take as completed, generates stock adjustments for discrepancies,
    /// and updates inventory levels accordingly.
    async fn finalize_stock_take(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
        user_id: Uuid,
        request: FinalizeStockTakeRequest,
    ) -> Result<FinalizeStockTakeResponse, AppError>;

    /// Get stock take details with lines
    async fn get_stock_take(
        &self,
        tenant_id: Uuid,
        stock_take_id: Uuid,
    ) -> Result<StockTakeDetailResponse, AppError>;

    /// List stock takes with filtering
    async fn list_stock_takes(
        &self,
        tenant_id: Uuid,
        query: StockTakeListQuery,
    ) -> Result<StockTakeListResponse, AppError>;
}
