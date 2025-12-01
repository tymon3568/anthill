use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{LotSerial, LotSerialStatus, LotSerialTrackingType};
use shared_error::AppError;

#[async_trait]
pub trait LotSerialRepository: Send + Sync + 'static {
    async fn create(&self, lot_serial: &LotSerial) -> Result<(), AppError>;
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<Option<LotSerial>, AppError>;
    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        tracking_type: Option<LotSerialTrackingType>,
        status: Option<LotSerialStatus>,
    ) -> Result<Vec<LotSerial>, AppError>;
    async fn find_available_for_picking(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
        quantity_needed: i64,
    ) -> Result<Vec<LotSerial>, AppError>;
    async fn update(&self, lot_serial: &LotSerial) -> Result<(), AppError>;
    async fn update_remaining_quantity(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
        new_remaining_quantity: i64,
    ) -> Result<(), AppError>;
    async fn delete(&self, tenant_id: Uuid, lot_serial_id: Uuid) -> Result<(), AppError>;
    async fn quarantine_expired_lots(&self, tenant_id: Uuid) -> Result<i64, AppError>;
}
