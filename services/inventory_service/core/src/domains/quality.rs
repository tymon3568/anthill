use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QualityControlPoint {
    pub qc_point_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub qc_type: QcPointType,
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "qc_point_type", rename_all = "snake_case")]
pub enum QcPointType {
    Incoming,
    Outgoing,
    Internal,
}

impl From<String> for QcPointType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "incoming" => QcPointType::Incoming,
            "outgoing" => QcPointType::Outgoing,
            "internal" => QcPointType::Internal,
            _ => panic!("Invalid qc_point_type: {}", s),
        }
    }
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
