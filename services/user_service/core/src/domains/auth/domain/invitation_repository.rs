use crate::domains::auth::domain::model::{Invitation, InvitationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_error::AppError;
use uuid::Uuid;

/// Repository trait for managing user invitations
#[async_trait]
pub trait InvitationRepository: Send + Sync {
    /// Create a new invitation
    async fn create(&self, invitation: &Invitation) -> Result<Invitation, AppError>;

    /// Find invitation by ID (tenant-scoped for isolation)
    async fn find_by_id(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Option<Invitation>, AppError>;

    /// Find pending invitation by token hash (for acceptance)
    async fn find_pending_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Invitation>, AppError>;

    /// Find pending invitation by tenant and email (for uniqueness check)
    async fn find_pending_by_tenant_and_email(
        &self,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<Option<Invitation>, AppError>;

    /// List invitations for a tenant with pagination
    async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Invitation>, AppError>;

    /// Count invitations for a tenant
    async fn count_by_tenant(&self, tenant_id: Uuid, status: Option<&str>)
        -> Result<i64, AppError>;

    /// Update invitation status (tenant-scoped)
    async fn update_status(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        status: InvitationStatus,
    ) -> Result<(), AppError>;

    /// Update invitation for resend (new token hash, expiry, reset attempts) - tenant-scoped
    async fn update_for_resend(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
        invited_from_ip: Option<&str>,
        invited_from_user_agent: Option<&str>,
    ) -> Result<Invitation, AppError>;

    /// Mark invitation as accepted (tenant-scoped)
    async fn mark_accepted(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        accepted_user_id: Uuid,
        accepted_from_ip: Option<&str>,
        accepted_from_user_agent: Option<&str>,
    ) -> Result<(), AppError>;

    /// Mark invitation as expired (tenant-scoped)
    async fn mark_expired(&self, tenant_id: Uuid, invitation_id: Uuid) -> Result<(), AppError>;

    /// Revoke invitation (tenant-scoped, enforces pending status)
    async fn revoke(&self, tenant_id: Uuid, invitation_id: Uuid) -> Result<(), AppError>;

    /// Increment accept attempts counter (tenant-scoped)
    async fn increment_accept_attempts(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), AppError>;

    /// Soft delete invitation (tenant-scoped)
    async fn soft_delete(&self, tenant_id: Uuid, invitation_id: Uuid) -> Result<(), AppError>;

    /// Cleanup expired invitations (mark as expired)
    async fn cleanup_expired(&self) -> Result<i64, AppError>;

    /// Hard delete old invitations (for cleanup)
    async fn hard_delete_old(&self, before_date: DateTime<Utc>) -> Result<i64, AppError>;
}
