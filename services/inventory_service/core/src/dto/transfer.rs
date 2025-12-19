use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domains::inventory::transfer::{TransferPriority, TransferStatus, TransferType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateTransferRequest {
    pub tenant_id: Uuid,
    pub reference_number: Option<String>,
    pub source_warehouse_id: Uuid,
    pub destination_warehouse_id: Uuid,
    pub transfer_type: TransferType,
    pub priority: TransferPriority,
    pub scheduled_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub reason: Option<String>,
    pub items: Vec<CreateTransferItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CreateTransferItemRequest {
    pub product_id: Uuid,
    pub quantity: i64,
    pub uom_id: Option<Uuid>,
    pub unit_cost: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct UpdateTransferRequest {
    pub status: Option<TransferStatus>,
    pub notes: Option<String>,
    pub items: Option<Vec<UpdateTransferItemRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct UpdateTransferItemRequest {
    pub item_id: Uuid,
    pub quantity: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransferResponse {
    pub transfer_id: Uuid,
    pub tenant_id: Uuid,
    pub transfer_number: String,
    pub reference_number: Option<String>,
    pub source_warehouse_id: Uuid,
    pub destination_warehouse_id: Uuid,
    pub status: TransferStatus,
    pub transfer_type: TransferType,
    pub priority: TransferPriority,
    pub transfer_date: chrono::NaiveDateTime,
    pub scheduled_date: Option<chrono::NaiveDate>,
    pub started_at: Option<chrono::NaiveDateTime>,
    pub completed_at: Option<chrono::NaiveDateTime>,
    pub initiated_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::NaiveDateTime>,
    pub total_quantity: i64,
    pub total_value: i64,
    pub currency_code: String,
    pub notes: Option<String>,
    pub reason: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransferItemResponse {
    pub transfer_item_id: Uuid,
    pub tenant_id: Uuid,
    pub transfer_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i64,
    pub uom_id: Option<Uuid>,
    pub unit_cost: Option<i64>,
    pub line_total: i64,
    pub line_number: i32,
    pub notes: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}
