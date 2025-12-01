use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{LotSerial, LotSerialStatus, LotSerialTrackingType};

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
