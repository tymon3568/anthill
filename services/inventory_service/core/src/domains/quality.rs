use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlPoint {
    pub qc_point_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub r#type: QcPointType,
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QcPointType {
    Incoming,
    Outgoing,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQualityControlPoint {
    pub name: String,
    pub r#type: QcPointType,
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQualityControlPoint {
    pub name: Option<String>,
    pub r#type: Option<QcPointType>,
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub active: Option<bool>,
}
