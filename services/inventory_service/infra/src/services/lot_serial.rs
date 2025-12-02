use async_trait::async_trait;
use uuid::Uuid;

use crate::repositories::stock::PgStockMoveRepository;
use crate::repositories::LotSerialRepositoryImpl;
use inventory_service_core::models::{
    LotSerial, LotSerialLifecycle, LotSerialStatus, LotSerialTrackingType,
};
use inventory_service_core::repositories::LotSerialRepository;
use inventory_service_core::services::LotSerialService;
use shared_error::AppError;

pub struct LotSerialServiceImpl {
    lot_serial_repo: LotSerialRepositoryImpl,
    stock_move_repo: PgStockMoveRepository,
}

impl LotSerialServiceImpl {
    pub fn new(
        lot_serial_repo: LotSerialRepositoryImpl,
        stock_move_repo: PgStockMoveRepository,
    ) -> Self {
        Self {
            lot_serial_repo,
            stock_move_repo,
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

        Ok(LotSerialLifecycle {
            lot_serial,
            supplier_name: None,
            purchase_order_number: None,
            coa_link: None,
            stock_moves,
            current_warehouse_name: None,
            current_location_code: None,
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
