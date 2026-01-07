use super::model::Invitation;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_error::AppError;
use uuid::Uuid;

/// Repository trait for managing user invitations
#[async_trait]
pub trait InvitationRepository: Send + Sync {
    /// Create a new invitation
    async fn create(&self, invitation: &Invitation) -> Result<Invitation, AppError>;

    /// Find invitation by ID
    async fn find_by_id(&self, invitation_id: Uuid) -> Result<Option<Invitation>, AppError>;

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

    /// Update invitation status
    async fn update_status(&self, invitation_id: Uuid, status: &str) -> Result<(), AppError>;

    /// Mark invitation as accepted
    async fn mark_accepted(
        &self,
        invitation_id: Uuid,
        accepted_user_id: Uuid,
        accepted_from_ip: Option<&str>,
        accepted_from_user_agent: Option<&str>,
    ) -> Result<(), AppError>;

    /// Mark invitation as expired
    async fn mark_expired(&self, invitation_id: Uuid) -> Result<(), AppError>;

    /// Revoke invitation
    async fn revoke(&self, invitation_id: Uuid) -> Result<(), AppError>;

    /// Increment accept attempts counter
    async fn increment_accept_attempts(&self, invitation_id: Uuid) -> Result<(), AppError>;

    /// Soft delete invitation
    async fn soft_delete(&self, invitation_id: Uuid) -> Result<(), AppError>;

    /// Cleanup expired invitations (mark as expired)
    async fn cleanup_expired(&self) -> Result<i64, AppError>;

    /// Hard delete old invitations (for cleanup)
    async fn hard_delete_old(&self, before_date: DateTime<Utc>) -> Result<i64, AppError>;
}
