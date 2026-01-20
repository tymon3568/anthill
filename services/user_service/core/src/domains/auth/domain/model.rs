use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// User entity (domain model)
///
/// This represents the user in the business domain,
/// mapped directly to the database table.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub user_id: Uuid, // Changed from 'id' to match schema
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: Option<String>, // Optional - NULL for users without password auth
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub full_name: Option<String>, // Now optional
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub status: String, // Changed from is_active to status
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub auth_method: String, // 'password' only
    pub migration_invited_at: Option<DateTime<Utc>>,
    pub migration_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>, // Soft delete
}

/// Tenant entity
#[derive(Debug, Clone, FromRow)]
pub struct Tenant {
    pub tenant_id: Uuid, // Changed from 'id' to match schema
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub settings: sqlx::types::Json<serde_json::Value>, // JSONB
    pub status: String,                                 // Changed from is_active
    pub owner_user_id: Option<Uuid>,                    // Owner of the tenant (set on registration)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>, // Soft delete
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
    pub access_token_hash: Option<String>, // Optional - for JWT sessions
    pub refresh_token_hash: Option<String>, // Optional - for JWT sessions
    pub ip_address: Option<String>,        // Stored as text for simplicity
    pub user_agent: Option<String>,
    pub device_info: Option<sqlx::types::Json<serde_json::Value>>,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub auth_method: String, // 'jwt' only
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

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

impl std::fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "pending"),
            InvitationStatus::Accepted => write!(f, "accepted"),
            InvitationStatus::Expired => write!(f, "expired"),
            InvitationStatus::Revoked => write!(f, "revoked"),
        }
    }
}

/// Invitation entity
///
/// Represents a secure user invitation with hash-at-rest tokens.
/// Tokens are never stored in plaintext - only SHA-256 hashes.
#[derive(Clone, FromRow)]
pub struct Invitation {
    pub invitation_id: Uuid,
    pub tenant_id: Uuid,
    pub token_hash: String, // SHA-256 hash of the token
    pub email: String,
    pub invited_role: String,
    pub invited_by_user_id: Uuid,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub accepted_user_id: Option<Uuid>, // User created on acceptance
    pub invited_from_ip: Option<String>,
    pub invited_from_user_agent: Option<String>,
    pub accepted_from_ip: Option<String>,
    pub accepted_from_user_agent: Option<String>,
    pub accept_attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub custom_message: Option<String>,
    pub metadata: sqlx::types::Json<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>, // Soft delete
}

/// Email Verification Token entity
///
/// Represents a secure email verification token sent to users during registration.
/// Tokens are hashed (SHA-256) at rest and expire after 24 hours.
#[derive(Debug, Clone, FromRow)]
pub struct EmailVerificationToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub token_hash: String, // SHA-256 hash of the token
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_from_ip: Option<String>,
    pub verified_from_user_agent: Option<String>,
    pub verification_attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

/// Password Reset Token entity
///
/// Represents a secure password reset token for forgot-password flow.
/// Tokens are hashed (SHA-256) at rest and expire after 1 hour.
/// Single-use: once used_at is set, the token cannot be reused.
#[derive(Debug, Clone, FromRow)]
pub struct PasswordResetToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub token_hash: String, // SHA-256 hash of the token
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub reset_from_ip: Option<String>,
    pub reset_from_user_agent: Option<String>,
}

/// Password Reset Audit entry
///
/// Records password reset events for security monitoring and audit trail.
#[derive(Debug, Clone, FromRow)]
pub struct PasswordResetAudit {
    pub audit_id: Uuid,
    pub user_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub email: String,
    pub event_type: String, // 'requested', 'completed', 'failed', 'expired', 'rate_limited'
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// PasswordResetToken Implementation
// ============================================================================

impl PasswordResetToken {
    /// Create a new password reset token
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        token_hash: String,
        expiry_hours: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            token_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            token_hash,
            expires_at: Utc::now() + chrono::Duration::hours(expiry_hours),
            created_at: Utc::now(),
            used_at: None,
            ip_address,
            user_agent,
            reset_from_ip: None,
            reset_from_user_agent: None,
        }
    }

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token has been used
    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    /// Check if the token is valid (not expired and not used)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used()
    }
}

// ============================================================================
// PasswordResetAudit Implementation
// ============================================================================

impl PasswordResetAudit {
    /// Create a new audit entry
    pub fn new(
        user_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
        email: String,
        event_type: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
        failure_reason: Option<String>,
    ) -> Self {
        Self {
            audit_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            email,
            event_type: event_type.to_string(),
            ip_address,
            user_agent,
            failure_reason,
            created_at: Utc::now(),
        }
    }

    /// Create audit entry for password reset requested
    pub fn requested(
        user_id: Uuid,
        tenant_id: Uuid,
        email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self::new(Some(user_id), Some(tenant_id), email, "requested", ip_address, user_agent, None)
    }

    /// Create audit entry for password reset completed
    pub fn completed(
        user_id: Uuid,
        tenant_id: Uuid,
        email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self::new(Some(user_id), Some(tenant_id), email, "completed", ip_address, user_agent, None)
    }

    /// Create audit entry for password reset failed
    pub fn failed(
        user_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
        email: String,
        failure_reason: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self::new(
            user_id,
            tenant_id,
            email,
            "failed",
            ip_address,
            user_agent,
            Some(failure_reason.to_string()),
        )
    }

    /// Create audit entry for rate limited request
    pub fn rate_limited(
        email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self::new(
            None,
            None,
            email,
            "rate_limited",
            ip_address,
            user_agent,
            Some("Too many password reset requests".to_string()),
        )
    }
}
