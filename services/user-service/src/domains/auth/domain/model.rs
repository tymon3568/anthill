use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// User entity (domain model)
/// 
/// This represents the user in the business domain, 
/// mapped directly to the database table.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub tenant_id: Uuid,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tenant entity
#[derive(Debug, Clone, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
