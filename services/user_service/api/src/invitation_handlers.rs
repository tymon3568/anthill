use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};

use shared_auth::extractors::RequireAdmin;
use shared_error::AppError;
use shared_jwt::{encode_jwt, Claims};
use user_service_core::domains::auth::{
    domain::service::AuthService,
    dto::{
        auth_dto::{ErrorResp, UserInfo},
        invitation_dto::{
            AcceptInvitationRequest, AcceptInvitationResponse, CreateInvitationRequest,
            CreateInvitationResponse, InvitationListItem, InvitedByInfo, ListInvitationsQuery,
            ListInvitationsResponse,
        },
    },
};
use uuid::Uuid;

use crate::handlers::AppState;

/// Create a new user invitation (Admin only)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/invite",
    tag = "invitations",
    operation_id = "admin_create_invitation",
    request_body = CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitation created successfully", body = CreateInvitationResponse),
        (status = 400, description = "Invalid request", body = ErrorResp),
        (status = 401, description = "Unauthorized", body = ErrorResp),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResp),
        (status = 409, description = "Pending invitation already exists", body = ErrorResp),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_invitation<S>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<CreateInvitationResponse>), AppError>
where
    S: AuthService + Send + Sync,
{
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let invitation_service = state
        .invitation_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Invitation service not available".into()))?;

    // Create invitation
    let (invitation, plaintext_token) = invitation_service
        .create_invitation(
            admin_user.tenant_id,
            &payload.email,
            &payload.role.unwrap_or_else(|| "user".to_string()),
            admin_user.user_id,
            payload.custom_message.as_deref(),
            client_info.ip_address.as_deref(),
            client_info.user_agent.as_deref(),
        )
        .await?;

    // For security, we don't return the token in the response
    // The admin should get it from the invite link construction
    let response = CreateInvitationResponse {
        invitation_id: invitation.invitation_id,
        email: invitation.email,
        role: invitation.invited_role,
        expires_at: invitation.expires_at,
        invite_link: format!("https://app.example.com/invite/{}", plaintext_token), // TODO: Use config
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Accept a user invitation (Public endpoint)
#[utoipa::path(
    post,
    path = "/api/v1/auth/accept-invite",
    tag = "invitations",
    operation_id = "accept_invitation",
    request_body = AcceptInvitationRequest,
    responses(
        (status = 201, description = "Invitation accepted successfully", body = AcceptInvitationResponse),
        (status = 400, description = "Invalid request", body = ErrorResp),
        (status = 401, description = "Invalid or expired invitation", body = ErrorResp),
        (status = 410, description = "Invitation has expired", body = ErrorResp),
        (status = 429, description = "Too many acceptance attempts", body = ErrorResp),
    )
)]
pub async fn accept_invitation<S>(
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<(StatusCode, Json<AcceptInvitationResponse>), AppError>
where
    S: AuthService + Send + Sync,
{
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let invitation_service = state
        .invitation_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Invitation service not available".into()))?;

    // Accept invitation
    let invitation = invitation_service
        .accept_invitation(
            &payload.token,
            &payload.password,
            payload.full_name.as_deref(),
            client_info.ip_address.as_deref(),
            client_info.user_agent.as_deref(),
        )
        .await?;

    // Generate JWT tokens for the new user
    let user_id = invitation
        .accepted_user_id
        .ok_or_else(|| AppError::InternalError("Accepted invitation missing user ID".into()))?;

    let access_claims = Claims::new_access(
        user_id,
        invitation.tenant_id,
        invitation.invited_role.clone(),
        900, // 15 minutes - TODO: from config
    );
    let refresh_claims = Claims::new_refresh(
        user_id,
        invitation.tenant_id,
        invitation.invited_role.clone(),
        604800, // 7 days - TODO: from config
    );

    let access_token = encode_jwt(&access_claims, &state.jwt_secret)?;
    let refresh_token = encode_jwt(&refresh_claims, &state.jwt_secret)?;

    let response = AcceptInvitationResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 900,
        user: UserInfo {
            id: user_id,
            email: invitation.email,
            full_name: payload.full_name,
            tenant_id: invitation.tenant_id,
            role: invitation.invited_role,
            created_at: invitation.accepted_at.unwrap(),
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// List invitations for the tenant (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/invitations",
    tag = "invitations",
    operation_id = "admin_list_invitations",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i64>, Query, description = "Page size (default: 20)"),
        ("status" = Option<String>, Query, description = "Filter by status (pending, accepted, expired, revoked)"),
    ),
    responses(
        (status = 200, description = "List of invitations", body = ListInvitationsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResp),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResp),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_invitations<S>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<Json<ListInvitationsResponse>, AppError>
where
    S: AuthService + Send + Sync,
{
    let invitation_service = state
        .invitation_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Invitation service not available".into()))?;

    // Validate and clamp pagination parameters
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    // Calculate offset
    let offset = (page - 1) * page_size;

    // Get invitations
    let invitations = invitation_service
        .list_invitations(admin_user.tenant_id, query.status.as_deref(), page_size, offset)
        .await?;

    // Get total count
    let total = invitation_service
        .count_invitations(admin_user.tenant_id, query.status.as_deref())
        .await?;

    // Convert to response items
    let items = invitations
        .into_iter()
        .map(|inv| InvitationListItem {
            invitation_id: inv.invitation_id,
            email: inv.email,
            role: inv.invited_role,
            status: inv.status,
            invited_by: InvitedByInfo {
                user_id: inv.invited_by_user_id,
                email: "admin@example.com".to_string(), // TODO: Get from user repo
                full_name: None,
            },
            expires_at: inv.expires_at,
            created_at: inv.created_at,
        })
        .collect();

    let response = ListInvitationsResponse {
        invitations: items,
        total,
        page,
        page_size,
    };

    Ok(Json(response))
}

/// Revoke an invitation (Admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/invitations/{invitation_id}",
    tag = "invitations",
    operation_id = "admin_revoke_invitation",
    params(
        ("invitation_id" = uuid::Uuid, Path, description = "Invitation ID"),
    ),
    responses(
        (status = 204, description = "Invitation revoked successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResp),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResp),
        (status = 404, description = "Invitation not found", body = ErrorResp),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn revoke_invitation<S>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(invitation_id): Path<Uuid>,
) -> Result<StatusCode, AppError>
where
    S: AuthService + Send + Sync,
{
    let invitation_service = state
        .invitation_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Invitation service not available".into()))?;

    invitation_service
        .revoke_invitation(admin_user.tenant_id, invitation_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Resend an invitation (Admin only)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/invitations/{invitation_id}/resend",
    tag = "invitations",
    operation_id = "admin_resend_invitation",
    params(
        ("invitation_id" = uuid::Uuid, Path, description = "Invitation ID"),
    ),
    responses(
        (status = 200, description = "Invitation resent successfully", body = CreateInvitationResponse),
        (status = 400, description = "Invalid request", body = ErrorResp),
        (status = 401, description = "Unauthorized", body = ErrorResp),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResp),
        (status = 404, description = "Invitation not found", body = ErrorResp),
        (status = 409, description = "Can only resend pending invitations", body = ErrorResp),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn resend_invitation<S>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<CreateInvitationResponse>, AppError>
where
    S: AuthService + Send + Sync,
{
    let invitation_service = state
        .invitation_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Invitation service not available".into()))?;

    // Resend invitation
    let (invitation, plaintext_token) = invitation_service
        .resend_invitation(
            admin_user.tenant_id,
            invitation_id,
            client_info.ip_address.as_deref(),
            client_info.user_agent.as_deref(),
        )
        .await?;

    let response = CreateInvitationResponse {
        invitation_id: invitation.invitation_id,
        email: invitation.email,
        role: invitation.invited_role,
        expires_at: invitation.expires_at,
        invite_link: format!("https://app.example.com/invite/{}", plaintext_token), // TODO: Use config
    };

    Ok(Json(response))
}
