use async_trait::async_trait;
use uuid::Uuid;
use super::model::UserProfile;
use crate::domains::auth::dto::profile_dto::{UpdateProfileRequest, ProfileCompletenessResponse};
use shared_error::AppError;

/// UserProfile repository trait
/// 
/// Defines the interface for user profile data access operations.
/// All operations are tenant-isolated.
#[async_trait]
pub trait UserProfileRepository: Send + Sync {
    /// Find profile by user_id within a tenant
    async fn find_by_user_id(&self, user_id: Uuid, tenant_id: Uuid) -> Result<Option<UserProfile>, AppError>;
    
    /// Create a new user profile
    async fn create(&self, profile: &UserProfile) -> Result<UserProfile, AppError>;
    
    /// Update user profile
    async fn update(&self, user_id: Uuid, tenant_id: Uuid, request: &UpdateProfileRequest) -> Result<UserProfile, AppError>;
    
    /// Update profile visibility settings
    async fn update_visibility(&self, user_id: Uuid, tenant_id: Uuid, visibility: &str, show_email: bool, show_phone: bool) -> Result<(), AppError>;
    
    /// Update notification preferences
    async fn update_notification_preferences(&self, user_id: Uuid, tenant_id: Uuid, preferences: &serde_json::Value) -> Result<(), AppError>;
    
    /// Calculate and update profile completeness score
    async fn calculate_completeness(&self, user_id: Uuid, tenant_id: Uuid) -> Result<ProfileCompletenessResponse, AppError>;
    
    /// Search profiles within tenant
    async fn search(
        &self,
        tenant_id: Uuid,
        query: Option<&str>,
        department: Option<&str>,
        location: Option<&str>,
        verified_only: bool,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<UserProfile>, i64), AppError>;
    
    /// Get public profiles (for discovery features)
    async fn get_public_profiles(&self, tenant_id: Uuid, page: i32, per_page: i32) -> Result<(Vec<UserProfile>, i64), AppError>;
    
    /// Update profile verification status
    async fn update_verification(&self, user_id: Uuid, tenant_id: Uuid, verified: bool, badge: Option<&str>) -> Result<(), AppError>;
}
