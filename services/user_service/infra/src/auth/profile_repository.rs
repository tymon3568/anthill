use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use user_service_core::domains::auth::domain::{
    model::UserProfile,
    profile_repository::UserProfileRepository,
};
use user_service_core::domains::auth::dto::profile_dto::{UpdateProfileRequest, ProfileCompletenessResponse};
use shared_error::AppError;

/// PostgreSQL implementation of UserProfileRepository
pub struct PgUserProfileRepository {
    pool: PgPool,
}

impl PgUserProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserProfileRepository for PgUserProfileRepository {
    async fn find_by_user_id(&self, user_id: Uuid, tenant_id: Uuid) -> Result<Option<UserProfile>, AppError> {
        let profile = sqlx::query_as::<_, UserProfile>(
            "SELECT * FROM user_profiles WHERE user_id = $1 AND tenant_id = $2"
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(profile)
    }
    
    async fn create(&self, profile: &UserProfile) -> Result<UserProfile, AppError> {
        let profile = sqlx::query_as::<_, UserProfile>(
            r#"
            INSERT INTO user_profiles (
                user_id, tenant_id, bio, title, department, location, website_url,
                social_links, language, timezone, date_format, time_format,
                notification_preferences, profile_visibility, show_email, show_phone,
                completeness_score, verified, verification_badge, custom_fields
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            RETURNING *
            "#
        )
        .bind(&profile.user_id)
        .bind(&profile.tenant_id)
        .bind(&profile.bio)
        .bind(&profile.title)
        .bind(&profile.department)
        .bind(&profile.location)
        .bind(&profile.website_url)
        .bind(&profile.social_links)
        .bind(&profile.language)
        .bind(&profile.timezone)
        .bind(&profile.date_format)
        .bind(&profile.time_format)
        .bind(&profile.notification_preferences)
        .bind(&profile.profile_visibility)
        .bind(profile.show_email)
        .bind(profile.show_phone)
        .bind(profile.completeness_score)
        .bind(profile.verified)
        .bind(&profile.verification_badge)
        .bind(&profile.custom_fields)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(profile)
    }
    
    async fn update(&self, user_id: Uuid, tenant_id: Uuid, request: &UpdateProfileRequest) -> Result<UserProfile, AppError> {
        // Build dynamic UPDATE query based on provided fields
        let mut query = String::from("UPDATE user_profiles SET updated_at = NOW()");
        let mut param_count = 3; // Starting from $3 (user_id=$1, tenant_id=$2)
        
        // Build the query dynamically
        if request.bio.is_some() {
            query.push_str(&format!(", bio = ${}", param_count));
            param_count += 1;
        }
        if request.title.is_some() {
            query.push_str(&format!(", title = ${}", param_count));
            param_count += 1;
        }
        if request.department.is_some() {
            query.push_str(&format!(", department = ${}", param_count));
            param_count += 1;
        }
        if request.location.is_some() {
            query.push_str(&format!(", location = ${}", param_count));
            param_count += 1;
        }
        if request.website_url.is_some() {
            query.push_str(&format!(", website_url = ${}", param_count));
            param_count += 1;
        }
        if request.social_links.is_some() {
            query.push_str(&format!(", social_links = ${}", param_count));
            param_count += 1;
        }
        if request.language.is_some() {
            query.push_str(&format!(", language = ${}", param_count));
            param_count += 1;
        }
        if request.timezone.is_some() {
            query.push_str(&format!(", timezone = ${}", param_count));
            param_count += 1;
        }
        if request.date_format.is_some() {
            query.push_str(&format!(", date_format = ${}", param_count));
            param_count += 1;
        }
        if request.time_format.is_some() {
            query.push_str(&format!(", time_format = ${}", param_count));
            param_count += 1;
        }
        if request.notification_preferences.is_some() {
            query.push_str(&format!(", notification_preferences = ${}", param_count));
            param_count += 1;
        }
        if request.profile_visibility.is_some() {
            query.push_str(&format!(", profile_visibility = ${}", param_count));
            param_count += 1;
        }
        if request.show_email.is_some() {
            query.push_str(&format!(", show_email = ${}", param_count));
            param_count += 1;
        }
        if request.show_phone.is_some() {
            query.push_str(&format!(", show_phone = ${}", param_count));
            param_count += 1;
        }
        if request.custom_fields.is_some() {
            query.push_str(&format!(", custom_fields = ${}", param_count));
            param_count += 1;
        }
        
        query.push_str(" WHERE user_id = $1 AND tenant_id = $2 RETURNING *");
        
        // Build the query with parameters
        let mut q = sqlx::query_as::<_, UserProfile>(&query)
            .bind(user_id)
            .bind(tenant_id);
        
        // Bind parameters in the same order as the query
        if let Some(ref bio) = request.bio {
            q = q.bind(bio);
        }
        if let Some(ref title) = request.title {
            q = q.bind(title);
        }
        if let Some(ref department) = request.department {
            q = q.bind(department);
        }
        if let Some(ref location) = request.location {
            q = q.bind(location);
        }
        if let Some(ref website_url) = request.website_url {
            q = q.bind(website_url);
        }
        if let Some(ref social_links) = request.social_links {
            q = q.bind(serde_json::to_value(social_links).map_err(|e| AppError::InternalError(e.to_string()))?);
        }
        if let Some(ref language) = request.language {
            q = q.bind(language);
        }
        if let Some(ref timezone) = request.timezone {
            q = q.bind(timezone);
        }
        if let Some(ref date_format) = request.date_format {
            q = q.bind(date_format);
        }
        if let Some(ref time_format) = request.time_format {
            q = q.bind(time_format);
        }
        if let Some(ref notification_preferences) = request.notification_preferences {
            q = q.bind(serde_json::to_value(notification_preferences).map_err(|e| AppError::InternalError(e.to_string()))?);
        }
        if let Some(ref profile_visibility) = request.profile_visibility {
            q = q.bind(profile_visibility);
        }
        if let Some(show_email) = request.show_email {
            q = q.bind(show_email);
        }
        if let Some(show_phone) = request.show_phone {
            q = q.bind(show_phone);
        }
        if let Some(ref custom_fields) = request.custom_fields {
            q = q.bind(serde_json::to_value(custom_fields).map_err(|e| AppError::InternalError(e.to_string()))?);
        }
        
        let profile = q.fetch_optional(&self.pool).await?;
        
        match profile {
            Some(profile) => Ok(profile),
            None => Err(AppError::NotFound("Profile not found".to_string())),
        }
    }
    
    async fn update_visibility(&self, user_id: Uuid, tenant_id: Uuid, visibility: &str, show_email: bool, show_phone: bool) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE user_profiles 
            SET profile_visibility = $3, show_email = $4, show_phone = $5, updated_at = NOW()
            WHERE user_id = $1 AND tenant_id = $2
            "#
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(visibility)
        .bind(show_email)
        .bind(show_phone)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Profile not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn update_notification_preferences(&self, user_id: Uuid, tenant_id: Uuid, preferences: &serde_json::Value) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE user_profiles 
            SET notification_preferences = $3, updated_at = NOW()
            WHERE user_id = $1 AND tenant_id = $2
            "#
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(preferences)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Profile not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn calculate_completeness(&self, user_id: Uuid) -> Result<ProfileCompletenessResponse, AppError> {
        // Call the database function to calculate completeness
        let score: i32 = sqlx::query_scalar(
            "SELECT calculate_profile_completeness($1)"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        
        // Update the score in the profile
        sqlx::query(
            r#"
            UPDATE user_profiles 
            SET completeness_score = $2, last_completeness_check_at = NOW()
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .bind(score)
        .execute(&self.pool)
        .await?;
        
        // Determine missing fields and suggestions
        let mut missing_fields = Vec::new();
        let mut suggestions = Vec::new();
        
        // Get user and profile data to check what's missing
        let user_data: Option<(Option<String>, Option<String>, Option<String>, bool)> = sqlx::query_as(
            "SELECT full_name, avatar_url, phone, email_verified FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        let profile_data: Option<(Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT bio, title, department, location FROM user_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some((full_name, avatar_url, phone, email_verified)) = user_data {
            if full_name.is_none() || full_name.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("full_name".to_string());
                suggestions.push("Add your full name to help others identify you".to_string());
            }
            if avatar_url.is_none() || avatar_url.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("avatar_url".to_string());
                suggestions.push("Upload a profile picture to personalize your account".to_string());
            }
            if phone.is_none() || phone.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("phone".to_string());
                suggestions.push("Add your phone number for better account security".to_string());
            }
            if !email_verified {
                missing_fields.push("email_verified".to_string());
                suggestions.push("Verify your email address to unlock all features".to_string());
            }
        }
        
        if let Some((bio, title, department, location)) = profile_data {
            if bio.is_none() || bio.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("bio".to_string());
                suggestions.push("Add a bio to tell others about yourself".to_string());
            }
            if title.is_none() || title.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("title".to_string());
                suggestions.push("Add your job title or position".to_string());
            }
            if department.is_none() || department.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("department".to_string());
                suggestions.push("Specify your department or team".to_string());
            }
            if location.is_none() || location.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                missing_fields.push("location".to_string());
                suggestions.push("Add your location to connect with nearby colleagues".to_string());
            }
        }
        
        Ok(ProfileCompletenessResponse {
            score,
            missing_fields,
            suggestions,
        })
    }
    
    async fn search(
        &self,
        tenant_id: Uuid,
        query: Option<&str>,
        department: Option<&str>,
        location: Option<&str>,
        verified_only: bool,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<UserProfile>, i64), AppError> {
        let offset = (page - 1) * per_page;
        
        // Build dynamic WHERE clause
        let mut where_clauses = vec![
            "up.tenant_id = $1".to_string(),
            "up.profile_visibility = 'public'".to_string(), // Only show public profiles
        ];
        let mut param_count = 2;
        
        if query.is_some() {
            where_clauses.push(format!(
                "(u.full_name ILIKE ${} OR up.title ILIKE ${} OR up.bio ILIKE ${})",
                param_count, param_count, param_count
            ));
            param_count += 1;
        }
        
        if department.is_some() {
            where_clauses.push(format!("up.department = ${}", param_count));
            param_count += 1;
        }
        
        if location.is_some() {
            where_clauses.push(format!("up.location = ${}", param_count));
            param_count += 1;
        }
        
        if verified_only {
            where_clauses.push("up.verified = true".to_string());
        }
        
        let where_clause = where_clauses.join(" AND ");
        
        // Count total
        let count_query = format!(
            "SELECT COUNT(*) FROM user_profiles up JOIN users u ON up.user_id = u.user_id WHERE {}",
            where_clause
        );
        
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query)
            .bind(tenant_id);
        
        if let Some(q) = query {
            count_q = count_q.bind(format!("%{}%", q));
        }
        if let Some(dept) = department {
            count_q = count_q.bind(dept);
        }
        if let Some(loc) = location {
            count_q = count_q.bind(loc);
        }
        
        let total = count_q.fetch_one(&self.pool).await?;
        
        // Fetch profiles
        let profiles_query = format!(
            "SELECT up.* FROM user_profiles up JOIN users u ON up.user_id = u.user_id WHERE {} ORDER BY up.completeness_score DESC, up.updated_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_count, param_count + 1
        );
        
        let mut profiles_q = sqlx::query_as::<_, UserProfile>(&profiles_query)
            .bind(tenant_id);
        
        if let Some(q) = query {
            profiles_q = profiles_q.bind(format!("%{}%", q));
        }
        if let Some(dept) = department {
            profiles_q = profiles_q.bind(dept);
        }
        if let Some(loc) = location {
            profiles_q = profiles_q.bind(loc);
        }
        
        profiles_q = profiles_q.bind(per_page).bind(offset);
        
        let profiles = profiles_q.fetch_all(&self.pool).await?;
        
        Ok((profiles, total))
    }
    
    async fn get_public_profiles(&self, tenant_id: Uuid, page: i32, per_page: i32) -> Result<(Vec<UserProfile>, i64), AppError> {
        let offset = (page - 1) * per_page;
        
        // Count total public profiles
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_profiles WHERE tenant_id = $1 AND profile_visibility = 'public'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;
        
        // Fetch public profiles
        let profiles = sqlx::query_as::<_, UserProfile>(
            r#"
            SELECT * FROM user_profiles 
            WHERE tenant_id = $1 AND profile_visibility = 'public'
            ORDER BY completeness_score DESC, verified DESC, updated_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(tenant_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        Ok((profiles, total))
    }
    
    async fn update_verification(&self, user_id: Uuid, tenant_id: Uuid, verified: bool, badge: Option<&str>) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE user_profiles 
            SET verified = $3, verification_badge = $4, verified_at = CASE WHEN $3 = true THEN NOW() ELSE NULL END, updated_at = NOW()
            WHERE user_id = $1 AND tenant_id = $2
            "#
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(verified)
        .bind(badge)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Profile not found".to_string()));
        }
        
        Ok(())
    }
}
