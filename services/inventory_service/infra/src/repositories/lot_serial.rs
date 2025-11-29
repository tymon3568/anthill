use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing;
use uuid::Uuid;

use inventory_service_core::models::{LotSerial, LotSerialStatus, LotSerialTrackingType};
use inventory_service_core::repositories::lot_serial::LotSerialRepository;
use shared_error::AppError;

/// PostgreSQL implementation of LotSerialRepository
pub struct LotSerialRepositoryImpl {
    pool: PgPool,
}

impl LotSerialRepositoryImpl {
    fn map_row_to_lot_serial(r: sqlx::postgres::PgRow) -> LotSerial {
        LotSerial {
            lot_serial_id: r.get("lot_serial_id"),
            tenant_id: r.get("tenant_id"),
            product_id: r.get("product_id"),
            tracking_type: {
                let raw = r.get::<String, _>("tracking_type");
                raw.parse().unwrap_or_else(|_| {
                    tracing::warn!(raw = %raw, lot_serial_id = ?r.get::<Uuid, _>("lot_serial_id"), "Unknown tracking_type, defaulting to Lot");
                    LotSerialTrackingType::Lot
                })
            },
            lot_number: r.get("lot_number"),
            serial_number: r.get("serial_number"),
            initial_quantity: r.get("initial_quantity"),
            remaining_quantity: r.get("remaining_quantity"),
            expiry_date: r.get("expiry_date"),
            status: {
                let raw = r.get::<String, _>("status");
                raw.parse().unwrap_or_else(|_| {
                    tracing::warn!(raw = %raw, lot_serial_id = ?r.get::<Uuid, _>("lot_serial_id"), "Unknown status, defaulting to Active");
                    LotSerialStatus::Active
                })
            },
            warehouse_id: r.get("warehouse_id"),
            location_id: r.get("location_id"),
            created_by: r.get("created_by"),
            updated_by: r.get("updated_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            deleted_at: r.get("deleted_at"),
        }
    }

    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LotSerialRepository for LotSerialRepositoryImpl {
    async fn create(&self, lot_serial: &LotSerial) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO lots_serial_numbers (
                lot_serial_id, tenant_id, product_id, tracking_type, lot_number,
                serial_number, initial_quantity, remaining_quantity, expiry_date,
                status, warehouse_id, location_id, created_by, updated_by,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            "#,
        )
        .bind(lot_serial.lot_serial_id)
        .bind(lot_serial.tenant_id)
        .bind(lot_serial.product_id)
        .bind(lot_serial.tracking_type.to_string())
        .bind(lot_serial.lot_number.clone())
        .bind(lot_serial.serial_number.clone())
        .bind(lot_serial.initial_quantity)
        .bind(lot_serial.remaining_quantity)
        .bind(lot_serial.expiry_date)
        .bind(lot_serial.status.to_string())
        .bind(lot_serial.warehouse_id)
        .bind(lot_serial.location_id)
        .bind(lot_serial.created_by)
        .bind(lot_serial.updated_by)
        .bind(lot_serial.created_at)
        .bind(lot_serial.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
    ) -> Result<Option<LotSerial>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                lot_serial_id, tenant_id, product_id, tracking_type, lot_number,
                serial_number, initial_quantity, remaining_quantity, expiry_date,
                status, warehouse_id, location_id, created_by, updated_by,
                created_at, updated_at, deleted_at
            FROM lots_serial_numbers
            WHERE tenant_id = $1 AND lot_serial_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(lot_serial_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row_to_lot_serial))
    }

    async fn find_by_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        tracking_type: Option<LotSerialTrackingType>,
        status: Option<LotSerialStatus>,
    ) -> Result<Vec<LotSerial>, AppError> {
        let mut query = sqlx::QueryBuilder::new(
            r#"
            SELECT
                lot_serial_id, tenant_id, product_id, tracking_type, lot_number,
                serial_number, initial_quantity, remaining_quantity, expiry_date,
                status, warehouse_id, location_id, created_by, updated_by,
                created_at, updated_at, deleted_at
            FROM lots_serial_numbers
            WHERE tenant_id =
            "#,
        );
        query.push_bind(tenant_id);
        query.push(" AND product_id = ");
        query.push_bind(product_id);
        query.push(" AND deleted_at IS NULL");

        if let Some(tt) = tracking_type {
            query.push(" AND tracking_type = ");
            query.push_bind(tt.to_string());
        }

        if let Some(s) = status {
            query.push(" AND status = ");
            query.push_bind(s.to_string());
        }

        query.push(" ORDER BY expiry_date ASC NULLS LAST");

        let rows = query.build().fetch_all(&self.pool).await?;

        let lot_serials = rows.into_iter().map(Self::map_row_to_lot_serial).collect();

        Ok(lot_serials)
    }

    async fn find_available_for_picking(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        warehouse_id: Option<Uuid>,
        quantity_needed: i64,
    ) -> Result<Vec<LotSerial>, AppError> {
        let mut query = sqlx::QueryBuilder::new(
            r#"
            SELECT
                lot_serial_id, tenant_id, product_id, tracking_type, lot_number,
                serial_number, initial_quantity, remaining_quantity, expiry_date,
                status, warehouse_id, location_id, created_by, updated_by,
                created_at, updated_at, deleted_at
            FROM lots_serial_numbers
            WHERE tenant_id =
            "#,
        );
        query.push_bind(tenant_id);
        query.push(" AND product_id = ");
        query.push_bind(product_id);
        query.push(" AND deleted_at IS NULL");
        query.push(" AND status = ");
        query.push_bind(LotSerialStatus::Active.to_string());
        query.push(" AND remaining_quantity > 0");
        query.push(" AND (expiry_date IS NULL OR expiry_date > CURRENT_DATE)");

        if let Some(wh_id) = warehouse_id {
            query.push(" AND warehouse_id = ");
            query.push_bind(wh_id);
        }

        query.push(" ORDER BY expiry_date ASC NULLS LAST, created_at ASC");

        let rows = query.build().fetch_all(&self.pool).await?;

        let lot_serials: Vec<LotSerial> =
            rows.into_iter().map(Self::map_row_to_lot_serial).collect();

        // Take only enough to cover quantity_needed
        let mut selected = Vec::new();
        let mut remaining_needed = quantity_needed;

        for ls in lot_serials {
            if remaining_needed <= 0 {
                break;
            }
            let available = ls.remaining_quantity.unwrap_or(0);
            if available > 0 {
                selected.push(ls);
                remaining_needed -= available;
            }
        }

        Ok(selected)
    }

    async fn update(&self, lot_serial: &LotSerial) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE lots_serial_numbers SET
                tracking_type = $3, lot_number = $4, serial_number = $5,
                initial_quantity = $6, remaining_quantity = $7, expiry_date = $8,
                status = $9, warehouse_id = $10, location_id = $11,
                updated_by = $12, updated_at = $13
            WHERE tenant_id = $1 AND lot_serial_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(lot_serial.tenant_id)
        .bind(lot_serial.lot_serial_id)
        .bind(lot_serial.tracking_type.to_string())
        .bind(lot_serial.lot_number.clone())
        .bind(lot_serial.serial_number.clone())
        .bind(lot_serial.initial_quantity)
        .bind(lot_serial.remaining_quantity)
        .bind(lot_serial.expiry_date)
        .bind(lot_serial.status.to_string())
        .bind(lot_serial.warehouse_id)
        .bind(lot_serial.location_id)
        .bind(lot_serial.updated_by)
        .bind(lot_serial.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_remaining_quantity(
        &self,
        tenant_id: Uuid,
        lot_serial_id: Uuid,
        new_remaining_quantity: i64,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE lots_serial_numbers SET
                remaining_quantity = $3, updated_at = NOW()
            WHERE tenant_id = $1 AND lot_serial_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            lot_serial_id,
            new_remaining_quantity
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: Uuid, lot_serial_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE lots_serial_numbers SET
                deleted_at = NOW()
            WHERE tenant_id = $1 AND lot_serial_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            lot_serial_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
