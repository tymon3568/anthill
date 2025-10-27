use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// DTO for getting user profile (combines User + UserProfile data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    // User basic info
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub email_verified: bool,
    
    // Extended profile info
    pub bio: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    
    // Social links
    pub social_links: HashMap<String, String>,
    
    // Preferences
    pub language: String,
    pub timezone: String,
    pub date_format: String,
    pub time_format: String,
    
    // Notification preferences
    pub notification_preferences: NotificationPreferences,
    
    // Privacy settings
    pub profile_visibility: String,
    pub show_email: bool,
    pub show_phone: bool,
    
    // Profile metadata
    pub completeness_score: i32,
    pub verified: bool,
    pub verification_badge: Option<String>,
    
    // Custom fields
    pub custom_fields: HashMap<String, serde_json::Value>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for updating user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    // User basic info (optional updates)
    pub full_name: Option<String>,
    pub phone: Option<String>,
    
    // Extended profile info
    pub bio: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    
    // Social links
    pub social_links: Option<HashMap<String, String>>,
    
    // Preferences
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    
    // Notification preferences
    pub notification_preferences: Option<NotificationPreferences>,
    
    // Privacy settings
    pub profile_visibility: Option<String>,
    pub show_email: Option<bool>,
    pub show_phone: Option<bool>,
    
    // Custom fields
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Notification preferences structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sms_notifications: bool,
    pub notification_types: NotificationTypes,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            email_notifications: true,
            push_notifications: false,
            sms_notifications: false,
            notification_types: NotificationTypes::default(),
        }
    }
}

/// Specific notification type preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTypes {
    pub order_updates: bool,
    pub inventory_alerts: bool,
    pub system_announcements: bool,
    pub security_alerts: bool,
    pub marketing_emails: bool,
}

impl Default for NotificationTypes {
    fn default() -> Self {
        Self {
            order_updates: true,
            inventory_alerts: true,
            system_announcements: true,
            security_alerts: true,
            marketing_emails: false,
        }
    }
}

/// DTO for profile avatar upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadAvatarRequest {
    pub file_name: String,
    pub file_size: usize,
    pub content_type: String,
}

/// DTO for avatar upload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadAvatarResponse {
    pub avatar_url: String,
    pub uploaded_at: DateTime<Utc>,
}

/// DTO for profile visibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileVisibilityRequest {
    pub profile_visibility: String, // public, private, team_only
    pub show_email: bool,
    pub show_phone: bool,
}

/// DTO for profile completeness response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCompletenessResponse {
    pub score: i32,
    pub missing_fields: Vec<String>,
    pub suggestions: Vec<String>,
}

/// DTO for profile search/discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSearchRequest {
    pub query: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub verified_only: Option<bool>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// DTO for public profile view (limited info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicProfileResponse {
    pub user_id: Uuid,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub verified: bool,
    pub verification_badge: Option<String>,
    pub social_links: HashMap<String, String>,
}
