use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// User entity (domain model)
/// 
/// This represents the user in the business domain, 
/// mapped directly to the database table.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub user_id: Uuid,  // Changed from 'id' to match schema
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub full_name: Option<String>,  // Now optional
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub status: String,  // Changed from is_active to status
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,  // Soft delete
}

/// Tenant entity
#[derive(Debug, Clone, FromRow)]
pub struct Tenant {
    pub tenant_id: Uuid,  // Changed from 'id' to match schema
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub settings: sqlx::types::Json<serde_json::Value>,  // JSONB
    pub status: String,  // Changed from is_active
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,  // Soft delete
}
