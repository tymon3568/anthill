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

/// Session entity
/// 
/// Represents an active user session with JWT tokens.
/// Tokens are hashed (SHA-256) for security.
#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,  // Stored as text for simplicity
    pub user_agent: Option<String>,
    pub device_info: Option<sqlx::types::Json<serde_json::Value>>,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// UserProfile entity
/// 
/// Extended user profile information with preferences and settings.
/// This is a separate entity from User to keep the core user table lean.
#[derive(Debug, Clone, FromRow)]
pub struct UserProfile {
    pub profile_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    
    // Extended profile information
    pub bio: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    
    // Social links and preferences (JSONB)
    pub social_links: sqlx::types::Json<serde_json::Value>,
    pub language: String,
    pub timezone: String,
    pub date_format: String,
    pub time_format: String,
    
    // Notification preferences (JSONB)
    pub notification_preferences: sqlx::types::Json<serde_json::Value>,
    
    // Privacy settings
    pub profile_visibility: String,
    pub show_email: bool,
    pub show_phone: bool,
    
    // Profile completeness
    pub completeness_score: i32,
    pub last_completeness_check_at: Option<DateTime<Utc>>,
    
    // Verification
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_badge: Option<String>,
    
    // Custom fields (JSONB)
    pub custom_fields: sqlx::types::Json<serde_json::Value>,
    
    // Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
