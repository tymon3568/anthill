use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{LotSerial, LotSerialStatus, LotSerialTrackingType};
use crate::repositories::LotSerialRepository;
use shared_error::AppError;

#[async_trait]
pub trait LotSerialService: Send + Sync + 'static {
    async fn create_lot_serial(&self, lot_serial: &LotSerial) -> Result<(), AppError>;
    async fn get_lot_serial(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<Option<LotSerial>, AppError>;
    async fn list_lot_serials_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        tracking_type: Option<LotSerialTrackingType>,
        status: Option<LotSerialStatus>,
    ) -> Result<Vec<LotSerial>, AppError>;
    async fn update_lot_serial(&self, lot_serial: &LotSerial) -> Result<(), AppError>;
    async fn delete_lot_serial(&self, tenant_id: Uuid, lot_serial_id: Uuid)
        -> Result<(), AppError>;
    async fn quarantine_expired_lots(&self, tenant_id: Uuid) -> Result<i64, AppError>;
}

pub struct LotSerialServiceImpl<R: LotSerialRepository> {
    lot_serial_repo: R,
}

impl<R: LotSerialRepository> LotSerialServiceImpl<R> {
    pub fn new(lot_serial_repo: R) -> Self {
        Self { lot_serial_repo }
    }
}

#[async_trait]
impl<R: LotSerialRepository + 'static> LotSerialService for LotSerialServiceImpl<R> {
    async fn create_lot_serial(&self, lot_serial: &LotSerial) -> Result<(), AppError> {
        self.lot_serial_repo.create(lot_serial).await
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
