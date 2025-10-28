-- Migration: Create User Profiles Table
-- Description: Extended user profile information with preferences and settings
-- Author: Cascade
-- Date: 2025-10-27

-- =============================================================================
-- TABLE: user_profiles
-- =============================================================================

CREATE TABLE user_profiles (
    profile_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    
    -- Extended Profile Information
    bio TEXT, -- User biography/description
    title VARCHAR(255), -- Job title or position
    department VARCHAR(255), -- Department or team
    location VARCHAR(255), -- Physical location or timezone
    website_url TEXT, -- Personal or company website
    
    -- Social Links (JSONB for flexibility)
    social_links JSONB DEFAULT '{}',
    -- Example:
    -- {
    --   "linkedin": "https://linkedin.com/in/username",
    --   "twitter": "https://twitter.com/username",
    --   "github": "https://github.com/username"
    -- }
    
    -- Preferences
    language VARCHAR(10) DEFAULT 'en', -- UI language preference (en, vi, etc.)
    timezone VARCHAR(100) DEFAULT 'UTC', -- User timezone
    date_format VARCHAR(50) DEFAULT 'YYYY-MM-DD', -- Date format preference
    time_format VARCHAR(50) DEFAULT '24h', -- 12h or 24h
    
    -- Notification Preferences (JSONB)
    notification_preferences JSONB DEFAULT '{}',
    -- Example:
    -- {
    --   "email_notifications": true,
    --   "push_notifications": false,
    --   "sms_notifications": false,
    --   "notification_types": {
    --     "order_updates": true,
    --     "inventory_alerts": true,
    --     "system_announcements": false
    --   }
    -- }
    
    -- Privacy Settings
    profile_visibility VARCHAR(50) DEFAULT 'private', -- public, private, team_only
    show_email BOOLEAN DEFAULT FALSE,
    show_phone BOOLEAN DEFAULT FALSE,
    
    -- Profile Completeness
    completeness_score INTEGER DEFAULT 0, -- 0-100 score
    last_completeness_check_at TIMESTAMPTZ,
    
    -- Profile Verification
    verified BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    verification_badge VARCHAR(50), -- verified, trusted, expert, etc.
    
    -- Custom Fields (JSONB for extensibility)
    custom_fields JSONB DEFAULT '{}',
    -- Example:
    -- {
    --   "employee_id": "EMP-12345",
    --   "manager": "John Doe",
    --   "start_date": "2024-01-15"
    -- }
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT user_profiles_visibility_check CHECK (profile_visibility IN ('public', 'private', 'team_only')),
    CONSTRAINT user_profiles_completeness_check CHECK (completeness_score >= 0 AND completeness_score <= 100),
    CONSTRAINT user_profiles_language_check CHECK (language IN ('en', 'vi', 'zh', 'ja', 'ko', 'es', 'fr', 'de')),
    CONSTRAINT user_profiles_time_format_check CHECK (time_format IN ('12h', '24h')),
    CONSTRAINT user_profiles_date_format_check CHECK (date_format IN ('YYYY-MM-DD', 'DD-MM-YYYY', 'MM-DD-YYYY', 'DD/MM/YYYY', 'MM/DD/YYYY')),
    CONSTRAINT user_profiles_verified_at_check CHECK ((verified = TRUE AND verified_at IS NOT NULL) OR (verified = FALSE AND verified_at IS NULL))
);

-- Add UNIQUE constraint on users table to support composite FK
ALTER TABLE users 
ADD CONSTRAINT IF NOT EXISTS users_user_tenant_unique UNIQUE (user_id, tenant_id);

-- Add composite foreign key constraint for tenant isolation
ALTER TABLE user_profiles 
ADD CONSTRAINT user_profiles_user_tenant_fk 
FOREIGN KEY (user_id, tenant_id) 
REFERENCES users(user_id, tenant_id) 
ON DELETE CASCADE;

-- Indexes
CREATE INDEX idx_user_profiles_user ON user_profiles(user_id);
CREATE INDEX idx_user_profiles_tenant ON user_profiles(tenant_id);
CREATE INDEX idx_user_profiles_visibility ON user_profiles(profile_visibility);
CREATE INDEX idx_user_profiles_verified ON user_profiles(verified) WHERE verified = TRUE;
CREATE INDEX idx_user_profiles_completeness ON user_profiles(completeness_score);

-- Auto-update updated_at
CREATE TRIGGER update_user_profiles_updated_at
    BEFORE UPDATE ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE user_profiles IS 'Extended user profile information with preferences and settings';
COMMENT ON COLUMN user_profiles.bio IS 'User biography or description (max 5000 chars)';
COMMENT ON COLUMN user_profiles.social_links IS 'Social media links in JSONB format';
COMMENT ON COLUMN user_profiles.notification_preferences IS 'User notification preferences in JSONB format';
COMMENT ON COLUMN user_profiles.profile_visibility IS 'Profile visibility setting (public, private, team_only)';
COMMENT ON COLUMN user_profiles.completeness_score IS 'Profile completeness score (0-100)';
COMMENT ON COLUMN user_profiles.custom_fields IS 'Extensible custom fields in JSONB format';

-- =============================================================================
-- FUNCTION: Calculate Profile Completeness Score
-- =============================================================================

CREATE OR REPLACE FUNCTION calculate_profile_completeness(p_user_id UUID, p_tenant_id UUID)
RETURNS INTEGER AS $$
DECLARE
    v_score INTEGER := 0;
    v_user_record RECORD;
    v_profile_record RECORD;
BEGIN
    -- Get user basic info
    SELECT full_name, avatar_url, phone, email_verified
    INTO v_user_record
    FROM users
    WHERE user_id = p_user_id AND tenant_id = p_tenant_id;
    
    -- Get profile info
    SELECT bio, title, department, location, social_links
    INTO v_profile_record
    FROM user_profiles
    WHERE user_id = p_user_id AND tenant_id = p_tenant_id;
    
    -- Calculate score (weights total 100; see individual increments below)
    IF v_user_record.full_name IS NOT NULL AND v_user_record.full_name != '' THEN
        v_score := v_score + 15;
    END IF;
    
    IF v_user_record.avatar_url IS NOT NULL AND v_user_record.avatar_url != '' THEN
        v_score := v_score + 15;
    END IF;
    
    IF v_user_record.phone IS NOT NULL AND v_user_record.phone != '' THEN
        v_score := v_score + 10;
    END IF;
    
    IF v_user_record.email_verified = TRUE THEN
        v_score := v_score + 20;
    END IF;
    
    IF v_profile_record.bio IS NOT NULL AND v_profile_record.bio != '' THEN
        v_score := v_score + 10;
    END IF;
    
    IF v_profile_record.title IS NOT NULL AND v_profile_record.title != '' THEN
        v_score := v_score + 10;
    END IF;
    
    IF v_profile_record.department IS NOT NULL AND v_profile_record.department != '' THEN
        v_score := v_score + 5;
    END IF;
    
    IF v_profile_record.location IS NOT NULL AND v_profile_record.location != '' THEN
        v_score := v_score + 5;
    END IF;
    
    IF v_profile_record.social_links IS NOT NULL AND v_profile_record.social_links != '{}' THEN
        v_score := v_score + 10;
    END IF;
    
    RETURN v_score;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION calculate_profile_completeness IS 'Calculate user profile completeness score (0-100)';

-- =============================================================================
-- TRIGGER: Auto-create profile on user creation
-- =============================================================================

CREATE OR REPLACE FUNCTION create_user_profile_on_user_insert()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_profiles (user_id, tenant_id)
    VALUES (NEW.user_id, NEW.tenant_id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_create_user_profile
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_user_profile_on_user_insert();

COMMENT ON FUNCTION create_user_profile_on_user_insert IS 'Automatically create a profile entry when a new user is created';
