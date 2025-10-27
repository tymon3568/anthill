use async_trait::async_trait;
use uuid::Uuid;
use shared_error::AppError;
use crate::domains::auth::dto::profile_dto::{
    ProfileResponse, UpdateProfileRequest, UploadAvatarRequest, UploadAvatarResponse,
    ProfileVisibilityRequest, ProfileCompletenessResponse, ProfileSearchRequest,
    PublicProfileResponse,
};

/// Profile service trait
/// 
/// Defines the business logic interface for user profile operations.
#[async_trait]
pub trait ProfileService: Send + Sync {
    /// Get current user's profile (combines User + UserProfile data)
    async fn get_profile(&self, user_id: Uuid, tenant_id: Uuid) -> Result<ProfileResponse, AppError>;
    
    /// Update current user's profile
    async fn update_profile(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<ProfileResponse, AppError>;
    
    /// Upload profile avatar
    async fn upload_avatar(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        request: UploadAvatarRequest,
        file_data: Vec<u8>,
    ) -> Result<UploadAvatarResponse, AppError>;
    
    /// Update profile visibility settings
    async fn update_visibility(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        request: ProfileVisibilityRequest,
    ) -> Result<(), AppError>;
    
    /// Get profile completeness score and suggestions
    async fn get_completeness(&self, user_id: Uuid) -> Result<ProfileCompletenessResponse, AppError>;
    
    /// Search profiles within tenant
    async fn search_profiles(
        &self,
        tenant_id: Uuid,
        request: ProfileSearchRequest,
    ) -> Result<(Vec<PublicProfileResponse>, i64), AppError>;
    
    /// Get public profile by user_id
    async fn get_public_profile(&self, user_id: Uuid, tenant_id: Uuid) -> Result<PublicProfileResponse, AppError>;
    
    /// Update profile verification status (admin only)
    async fn update_verification(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        verified: bool,
        badge: Option<String>,
    ) -> Result<(), AppError>;
}
