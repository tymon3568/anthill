//! Application state for inventory service
//!
//! This module contains the shared application state used across all handlers.

use std::sync::Arc;

use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_core::services::category::CategoryService;
// use inventory_service_core::services::delivery::DeliveryService;
use inventory_service_core::services::product::ProductService;
use inventory_service_core::services::receipt::ReceiptService;
use inventory_service_core::services::reconciliation::StockReconciliationService;
use inventory_service_core::services::stock_take::StockTakeService;
use inventory_service_core::services::transfer::TransferService;
use inventory_service_core::services::valuation::ValuationService;

use shared_auth::enforcer::SharedEnforcer;
use shared_auth::extractors::{JwtSecretProvider, KanidmClientProvider};
use shared_kanidm_client::KanidmClient;

/// Application state for inventory service
#[derive(Clone)]
pub struct AppState {
    pub category_service: Arc<dyn CategoryService>,
    pub product_service: Arc<dyn ProductService>,
    pub valuation_service: Arc<dyn ValuationService>,
    pub warehouse_repository: Arc<dyn WarehouseRepository>,
    pub receipt_service: Arc<dyn ReceiptService>,
    // pub delivery_service: Arc<dyn DeliveryService>,
    pub transfer_service: Arc<dyn TransferService>,
    pub stock_take_service: Arc<dyn StockTakeService>,
    pub reconciliation_service: Arc<dyn StockReconciliationService>,
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
    pub kanidm_client: KanidmClient,
}

impl JwtSecretProvider for AppState {
    fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}

impl KanidmClientProvider for AppState {
    fn get_kanidm_client(&self) -> &KanidmClient {
        &self.kanidm_client
    }
}
