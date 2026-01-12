use super::model::Invitation;
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Service trait for managing user invitations
#[async_trait]
pub trait InvitationService: Send + Sync {
    /// Create a new invitation
    async fn create_invitation(
        &self,
        tenant_id: Uuid,
        email: &str,
        invited_role: &str,
        invited_by_user_id: Uuid,
        custom_message: Option<&str>,
        invited_from_ip: Option<&str>,
        invited_from_user_agent: Option<&str>,
    ) -> Result<(Invitation, String), AppError>; // Returns (invitation, plaintext_token)

    /// Accept an invitation with a token
    async fn accept_invitation(
        &self,
        token: &str,
        password: &str,
        full_name: Option<&str>,
        accepted_from_ip: Option<&str>,
        accepted_from_user_agent: Option<&str>,
    ) -> Result<Invitation, AppError>;

    /// Get invitation by ID (tenant-scoped)
    async fn get_invitation(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Option<Invitation>, AppError>;

    /// List invitations for a tenant
    async fn list_invitations(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Invitation>, AppError>;

    /// Count invitations for a tenant
    async fn count_invitations(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
    ) -> Result<i64, AppError>;

    /// Revoke an invitation (tenant-scoped)
    async fn revoke_invitation(&self, tenant_id: Uuid, invitation_id: Uuid)
        -> Result<(), AppError>;

    /// Resend an invitation (create new token) - tenant-scoped
    async fn resend_invitation(
        &self,
        tenant_id: Uuid,
        invitation_id: Uuid,
        invited_from_ip: Option<&str>,
        invited_from_user_agent: Option<&str>,
    ) -> Result<(Invitation, String), AppError>; // Returns (updated_invitation, new_plaintext_token)

    /// Cleanup expired invitations
    async fn cleanup_expired_invitations(&self) -> Result<i64, AppError>;
}
