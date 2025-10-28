use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;
use std::sync::Arc;
use user_service_core::domains::auth::domain::profile_service::ProfileService;
use user_service_core::domains::auth::dto::profile_dto::{
    ProfileResponse, UpdateProfileRequest, ProfileVisibilityRequest,
    ProfileCompletenessResponse, ProfileSearchRequest, PublicProfileResponse,
};
use uuid::Uuid;

/// Application state for profile handlers
pub struct ProfileAppState<S: ProfileService> {
    pub profile_service: Arc<S>,
}

impl<S: ProfileService> Clone for ProfileAppState<S> {
    fn clone(&self) -> Self {
        Self {
            profile_service: Arc::clone(&self.profile_service),
        }
    }
}

/// Get current user's profile
#[utoipa::path(
    get,
    path = "/api/v1/users/profile",
    tag = "profile",
    operation_id = "get_user_profile",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
    )
)]
pub async fn get_profile<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let profile = state.profile_service
        .get_profile(auth_user.user_id, auth_user.tenant_id)
        .await?;
    
    Ok(Json(profile))
}

/// Update current user's profile
#[utoipa::path(
    put,
    path = "/api/v1/users/profile",
    tag = "profile",
    operation_id = "update_user_profile",
    security(
        ("bearer_auth" = [])
    ),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = ProfileResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
    )
)]
pub async fn update_profile<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    let profile = state.profile_service
        .update_profile(auth_user.user_id, auth_user.tenant_id, request)
        .await?;
    
    Ok(Json(profile))
}

/// Update profile visibility settings
#[utoipa::path(
    put,
    path = "/api/v1/users/profile/visibility",
    tag = "profile",
    operation_id = "update_profile_visibility",
    security(
        ("bearer_auth" = [])
    ),
    request_body = ProfileVisibilityRequest,
    responses(
        (status = 200, description = "Visibility settings updated successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_visibility<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
    Json(request): Json<ProfileVisibilityRequest>,
) -> Result<StatusCode, AppError> {
    state.profile_service
        .update_visibility(auth_user.user_id, auth_user.tenant_id, request)
        .await?;
    
    Ok(StatusCode::OK)
}

/// Get profile completeness score
#[utoipa::path(
    get,
    path = "/api/v1/users/profile/completeness",
    tag = "profile",
    operation_id = "get_profile_completeness",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Profile completeness retrieved successfully", body = ProfileCompletenessResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_completeness<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
) -> Result<Json<ProfileCompletenessResponse>, AppError> {
    let completeness = state.profile_service
        .get_completeness(auth_user.user_id, auth_user.tenant_id)
        .await?;
    
    Ok(Json(completeness))
}

/// Search profiles within tenant
#[utoipa::path(
    post,
    path = "/api/v1/users/profiles/search",
    tag = "profile",
    operation_id = "search_profiles",
    security(
        ("bearer_auth" = [])
    ),
    request_body = ProfileSearchRequest,
    responses(
        (status = 200, description = "Profiles retrieved successfully", body = ProfileSearchResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn search_profiles<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
    Json(request): Json<ProfileSearchRequest>,
) -> Result<Json<ProfileSearchResponse>, AppError> {
    let (profiles, total) = state.profile_service
        .search_profiles(auth_user.tenant_id, request)
        .await?;
    
    Ok(Json(ProfileSearchResponse {
        profiles,
        total,
    }))
}

/// Get public profile by user ID
#[utoipa::path(
    get,
    path = "/api/v1/users/profiles/{user_id}",
    tag = "profile",
    operation_id = "get_public_profile",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Public profile retrieved successfully", body = PublicProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Profile is not public"),
        (status = 404, description = "Profile not found"),
    )
)]
pub async fn get_public_profile<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<PublicProfileResponse>, AppError> {
    let profile = state.profile_service
        .get_public_profile(user_id, auth_user.tenant_id)
        .await?;
    
    Ok(Json(profile))
}

/// Update profile verification (admin only)
#[utoipa::path(
    put,
    path = "/api/v1/users/profiles/{user_id}/verification",
    tag = "profile",
    operation_id = "update_profile_verification",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateVerificationRequest,
    responses(
        (status = 200, description = "Verification status updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Profile not found"),
    )
)]
pub async fn update_verification<S: ProfileService>(
    State(state): State<ProfileAppState<S>>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateVerificationRequest>,
) -> Result<StatusCode, AppError> {
    // Check if user is admin (role check should be done by middleware)
    if auth_user.role != "admin" && auth_user.role != "super_admin" {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    
    state.profile_service
        .update_verification(user_id, auth_user.tenant_id, request.verified, request.badge)
        .await?;
    
    Ok(StatusCode::OK)
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProfileSearchResponse {
    pub profiles: Vec<PublicProfileResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateVerificationRequest {
    pub verified: bool,
    pub badge: Option<String>,
}
