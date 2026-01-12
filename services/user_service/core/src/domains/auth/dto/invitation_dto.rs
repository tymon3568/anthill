use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::auth_dto::UserInfo;

/// Request to create a new invitation
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateInvitationRequest {
    /// Email address of the user to invite
    #[validate(email)]
    pub email: String,

    /// Role to assign to the invited user (default: "user")
    #[validate(length(min = 1, max = 50))]
    pub role: Option<String>,

    /// Optional custom message from the inviter
    #[validate(length(max = 1000))]
    pub custom_message: Option<String>,
}

/// Response for invitation creation
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CreateInvitationResponse {
    /// Unique invitation ID
    pub invitation_id: Uuid,

    /// Email address invited
    pub email: String,

    /// Role assigned
    pub role: String,

    /// When the invitation expires
    pub expires_at: DateTime<Utc>,

    /// Invitation link (shown only once for security)
    pub invite_link: String,
}

/// Request to accept an invitation
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct AcceptInvitationRequest {
    /// Invitation token (from email link)
    #[validate(length(min = 1))]
    pub token: String,

    /// Password for the new account
    #[validate(length(min = 8, max = 128))]
    pub password: String,

    /// Full name (optional)
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
}

/// Response for invitation acceptance
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AcceptInvitationResponse {
    /// Access token
    pub access_token: String,

    /// Refresh token
    pub refresh_token: String,

    /// Token type
    pub token_type: String,

    /// Access token expiry in seconds
    pub expires_in: i64,

    /// User information
    pub user: UserInfo,
}

/// Invitation list item (for admin listing)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InvitationListItem {
    /// Unique invitation ID
    pub invitation_id: Uuid,

    /// Email address invited
    pub email: String,

    /// Role assigned
    pub role: String,

    /// Current status
    pub status: String,

    /// Information about who invited
    pub invited_by: InvitedByInfo,

    /// When the invitation expires
    pub expires_at: DateTime<Utc>,

    /// When invitation was created
    pub created_at: DateTime<Utc>,
}

/// Information about the inviter
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InvitedByInfo {
    /// User ID of inviter
    pub user_id: Uuid,

    /// Email of inviter
    pub email: String,

    /// Full name of inviter
    pub full_name: Option<String>,
}

/// Invitation details (full information)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InvitationDetails {
    /// Unique invitation ID
    pub invitation_id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Email address invited
    pub email: String,

    /// Role assigned
    pub role: String,

    /// Current status
    pub status: String,

    /// Information about who invited
    pub invited_by: InvitedByInfo,

    /// When the invitation expires
    pub expires_at: DateTime<Utc>,

    /// When invitation was accepted (if applicable)
    pub accepted_at: Option<DateTime<Utc>>,

    /// IP address used for invitation
    pub invited_from_ip: Option<String>,

    /// User agent used for invitation
    pub invited_from_user_agent: Option<String>,

    /// IP address used for acceptance
    pub accepted_from_ip: Option<String>,

    /// User agent used for acceptance
    pub accepted_from_user_agent: Option<String>,

    /// Number of acceptance attempts
    pub accept_attempts: i32,

    /// Custom message from inviter
    pub custom_message: Option<String>,

    /// When invitation was created
    pub created_at: DateTime<Utc>,
}

/// Request to resend an invitation
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ResendInvitationRequest {
    /// Optional new custom message
    #[validate(length(max = 1000))]
    pub custom_message: Option<String>,
}

/// Response for resending an invitation
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResendInvitationResponse {
    /// Unique invitation ID
    pub invitation_id: Uuid,

    /// Email address
    pub email: String,

    /// New expiry time
    pub new_expires_at: DateTime<Utc>,

    /// New invitation link (shown only once)
    pub invite_link: String,
}

/// Query parameters for listing invitations
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ListInvitationsQuery {
    /// Filter by status (pending, accepted, expired, revoked)
    pub status: Option<String>,

    /// Page number (1-based)
    #[validate(range(min = 1))]
    pub page: Option<i64>,

    /// Items per page
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<i64>,
}

/// Paginated response for invitation listing
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListInvitationsResponse {
    /// List of invitations
    pub invitations: Vec<InvitationListItem>,

    /// Total number of invitations matching the query
    pub total: i64,

    /// Current page number
    pub page: i64,

    /// Items per page
    pub page_size: i64,
}
