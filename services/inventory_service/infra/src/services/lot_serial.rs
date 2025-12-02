use async_trait::async_trait;
use uuid::Uuid;

use std::sync::Arc;

use crate::repositories::stock::PgStockMoveRepository;
use crate::repositories::LotSerialRepositoryImpl;
use inventory_service_core::models::{
    LotSerial, LotSerialLifecycle, LotSerialStatus, LotSerialTrackingType,
};
use inventory_service_core::repositories::{
    LotSerialRepository, StockMoveRepository, WarehouseRepository,
};
use inventory_service_core::services::LotSerialService;
use shared_error::AppError;

pub struct LotSerialServiceImpl {
    lot_serial_repo: LotSerialRepositoryImpl,
    stock_move_repo: Arc<PgStockMoveRepository>,
    warehouse_repo: Arc<dyn WarehouseRepository>,
}

impl LotSerialServiceImpl {
    pub fn new(
        lot_serial_repo: LotSerialRepositoryImpl,
        stock_move_repo: Arc<PgStockMoveRepository>,
        warehouse_repo: Arc<dyn WarehouseRepository>,
    ) -> Self {
        Self {
            lot_serial_repo,
            stock_move_repo,
            warehouse_repo,
        }
    }
}

#[async_trait]
impl LotSerialService for LotSerialServiceImpl {
    async fn create_lot_serial(&self, lot_serial: &LotSerial) -> Result<(), AppError> {
        self.lot_serial_repo.create(lot_serial).await
    }

    async fn get_lifecycle(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<LotSerialLifecycle, AppError> {
        let lot_serial = self
            .lot_serial_repo
            .find_by_id(tenant_id, lot_serial_id)
            .await?
            .ok_or(AppError::NotFound("Lot serial not found".to_string()))?;

        let stock_moves = self
            .stock_move_repo
            .find_by_lot_serial(tenant_id, lot_serial_id)
            .await?;

        // Populate current_warehouse_name from warehouses
        let current_warehouse_name = if let Some(warehouse_id) = lot_serial.warehouse_id {
            self.warehouse_repo
                .find_by_id(tenant_id, warehouse_id)
                .await?
                .map(|warehouse| warehouse.warehouse_name)
        } else {
            None
        };

        // Populate current_location_code from warehouse_locations
        let current_location_code = if let Some(location_id) = lot_serial.location_id {
            self.warehouse_repo
                .find_location_by_id(tenant_id, location_id)
                .await?
                .map(|location| location.location_code)
        } else {
            None
        };

        Ok(LotSerialLifecycle {
            lot_serial,
            supplier_name: None,
            purchase_order_number: None,
            coa_link: None,
            stock_moves,
            current_warehouse_name,
            current_location_code,
            quality_checks: vec![],
        })
    }

    async fn get_lot_serial(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<Option<LotSerial>, AppError> {
        self.lot_serial_repo
            .find_by_id(tenant_id, lot_serial_id)
            .await
    }

    async fn list_lot_serials_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        tracking_type: Option<LotSerialTrackingType>,
        status: Option<LotSerialStatus>,
    ) -> Result<Vec<LotSerial>, AppError> {
        self.lot_serial_repo
            .find_by_product(tenant_id, product_id, tracking_type, status)
            .await
    }

    async fn update_lot_serial(&self, lot_serial: &LotSerial) -> Result<(), AppError> {
        self.lot_serial_repo.update(lot_serial).await
    }

    async fn delete_lot_serial(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<(), AppError> {
        self.lot_serial_repo.delete(tenant_id, lot_serial_id).await
    }

    async fn quarantine_expired_lots(&self, tenant_id: Uuid) -> Result<i64, AppError> {
        self.lot_serial_repo
            .quarantine_expired_lots(tenant_id)
            .await
    }
}
