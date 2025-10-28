use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use user_service_core::domains::auth::domain::{
    profile_service::ProfileService,
    profile_repository::UserProfileRepository,
    repository::UserRepository,
};
use user_service_core::domains::auth::dto::profile_dto::{
    ProfileResponse, UpdateProfileRequest, UploadAvatarRequest, UploadAvatarResponse,
    ProfileVisibilityRequest, ProfileCompletenessResponse, ProfileSearchRequest,
    PublicProfileResponse, NotificationPreferences,
};
use shared_error::AppError;
use chrono::Utc;
use std::collections::HashMap;

/// Profile service implementation
pub struct ProfileServiceImpl {
    profile_repo: Arc<dyn UserProfileRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl ProfileServiceImpl {
    pub fn new(
        profile_repo: Arc<dyn UserProfileRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            profile_repo,
            user_repo,
        }
    }
}

#[async_trait]
impl ProfileService for ProfileServiceImpl {
    async fn get_profile(&self, user_id: Uuid, tenant_id: Uuid) -> Result<ProfileResponse, AppError> {
        // Get user basic info
        let user = self.user_repo.find_by_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        // Get user profile
        let profile = self.profile_repo.find_by_user_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::NotFound("User profile not found".to_string()))?;
        
        // Parse social_links from JSONB
        let social_links: HashMap<String, String> = serde_json::from_value(profile.social_links.0.clone())
            .map_err(|e| AppError::InternalError(format!("Failed to parse social_links JSON: {}", e)))?;
        
        // Parse notification_preferences from JSONB
        let notification_preferences: NotificationPreferences = serde_json::from_value(profile.notification_preferences.0.clone())
            .map_err(|e| AppError::InternalError(format!("Failed to parse notification_preferences JSON: {}", e)))?;
        
        // Parse custom_fields from JSONB
        let custom_fields: HashMap<String, serde_json::Value> = serde_json::from_value(profile.custom_fields.0.clone())
            .map_err(|e| AppError::InternalError(format!("Failed to parse custom_fields JSON: {}", e)))?;
        
        Ok(ProfileResponse {
            user_id: user.user_id,
            tenant_id: user.tenant_id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            phone: user.phone,
            role: user.role,
            email_verified: user.email_verified,
            bio: profile.bio,
            title: profile.title,
            department: profile.department,
            location: profile.location,
            website_url: profile.website_url,
            social_links,
            language: profile.language,
            timezone: profile.timezone,
            date_format: profile.date_format,
            time_format: profile.time_format,
            notification_preferences,
            profile_visibility: profile.profile_visibility,
            show_email: profile.show_email,
            show_phone: profile.show_phone,
            completeness_score: profile.completeness_score,
            verified: profile.verified,
            verification_badge: profile.verification_badge,
            custom_fields,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        })
    }
    
    async fn update_profile(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<ProfileResponse, AppError> {
        // Update profile in database
        let _updated_profile = self.profile_repo.update(user_id, tenant_id, &request).await?;
        
        // Update user basic fields if provided (full_name, phone)
        if request.full_name.is_some() || request.phone.is_some() {
            let mut user = self.user_repo.find_by_id(user_id, tenant_id).await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
            
            if let Some(full_name) = request.full_name {
                user.full_name = Some(full_name);
            }
            if let Some(phone) = request.phone {
                user.phone = Some(phone);
            }
            
            self.user_repo.update(&user).await?;
        }
        
        // Return updated profile
        self.get_profile(user_id, tenant_id).await
    }
    
    async fn upload_avatar(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        _request: UploadAvatarRequest,
        _file_data: Vec<u8>,
    ) -> Result<UploadAvatarResponse, AppError> {
        // TODO: Implement file upload to S3 or similar storage
        // For now, return a placeholder URL
        let avatar_url = format!("https://storage.example.com/avatars/{}.jpg", user_id);
        
        // Update user avatar_url
        let mut user = self.user_repo.find_by_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        user.avatar_url = Some(avatar_url.clone());
        self.user_repo.update(&user).await?;
        
        Ok(UploadAvatarResponse {
            avatar_url,
            uploaded_at: Utc::now(),
        })
    }
    
    async fn update_visibility(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        request: ProfileVisibilityRequest,
    ) -> Result<(), AppError> {
        self.profile_repo.update_visibility(
            user_id,
            tenant_id,
            &request.profile_visibility,
            request.show_email,
            request.show_phone,
        ).await
    }
    
    async fn get_completeness(&self, user_id: Uuid, tenant_id: Uuid) -> Result<ProfileCompletenessResponse, AppError> {
        self.profile_repo.calculate_completeness(user_id, tenant_id).await
    }
    
    async fn search_profiles(
        &self,
        tenant_id: Uuid,
        request: ProfileSearchRequest,
    ) -> Result<(Vec<PublicProfileResponse>, i64), AppError> {
        // Clamp paging parameters to safe ranges
        let page = request.page.unwrap_or(1).max(1);
        let per_page = request.per_page.unwrap_or(20).clamp(1, 100);
        
        let (profiles, total) = self.profile_repo.search(
            tenant_id,
            request.query.as_deref(),
            request.department.as_deref(),
            request.location.as_deref(),
            request.verified_only.unwrap_or(false),
            page,
            per_page,
        ).await?;
        
        // Convert to public profile responses with user fields
        let mut public_profiles = Vec::with_capacity(profiles.len());
        for p in profiles {
            let social_links: HashMap<String, String> = 
                serde_json::from_value(p.social_links.0.clone())
                .map_err(|e| AppError::InternalError(format!("Failed to parse social_links JSON for user {}: {}", p.user_id, e)))?;
            
            // Fetch user data to get full_name and avatar_url
            let user = self.user_repo
                .find_by_id(p.user_id, tenant_id)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
            
            public_profiles.push(PublicProfileResponse {
                user_id: p.user_id,
                full_name: user.full_name,
                avatar_url: user.avatar_url,
                title: p.title,
                department: p.department,
                location: p.location,
                bio: p.bio,
                verified: p.verified,
                verification_badge: p.verification_badge,
                social_links,
            });
        }
        
        Ok((public_profiles, total))
    }
    
    async fn get_public_profile(&self, user_id: Uuid, tenant_id: Uuid) -> Result<PublicProfileResponse, AppError> {
        let profile = self.profile_repo.find_by_user_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;
        
        // Check if profile is public
        if profile.profile_visibility != "public" {
            return Err(AppError::Forbidden("Profile is not public".to_string()));
        }
        
        let user = self.user_repo.find_by_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        
        let social_links: HashMap<String, String> = serde_json::from_value(profile.social_links.0.clone())
            .map_err(|e| AppError::InternalError(format!("Failed to parse social_links JSON for user {}: {}", user_id, e)))?;
        
        Ok(PublicProfileResponse {
            user_id: profile.user_id,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            title: profile.title,
            department: profile.department,
            location: profile.location,
            bio: profile.bio,
            verified: profile.verified,
            verification_badge: profile.verification_badge,
            social_links,
        })
    }
    
    async fn update_verification(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        verified: bool,
        badge: Option<String>,
    ) -> Result<(), AppError> {
        self.profile_repo.update_verification(
            user_id,
            tenant_id,
            verified,
            badge.as_deref(),
        ).await
    }
}
