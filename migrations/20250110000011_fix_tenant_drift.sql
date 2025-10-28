-- Migration: Fix Tenant Drift Issue
-- Description: Add composite foreign key to prevent tenant drift in user_profiles
-- Author: Cascade
-- Date: 2025-10-28

-- =============================================================================
-- STEP 1: Add UNIQUE constraint on users table
-- =============================================================================

-- First, ensure we have the constraint (idempotent)
ALTER TABLE users 
ADD CONSTRAINT IF NOT EXISTS users_user_tenant_unique UNIQUE (user_id, tenant_id);

-- =============================================================================
-- STEP 2: Fix user_profiles foreign key
-- =============================================================================

-- Drop existing constraints
ALTER TABLE user_profiles 
DROP CONSTRAINT IF EXISTS user_profiles_user_id_key,
DROP CONSTRAINT IF EXISTS user_profiles_user_id_fkey;

-- Add composite foreign key
ALTER TABLE user_profiles 
ADD CONSTRAINT user_profiles_user_tenant_fk 
    FOREIGN KEY (user_id, tenant_id) 
    REFERENCES users(user_id, tenant_id) 
    ON DELETE CASCADE,
ADD CONSTRAINT user_profiles_user_tenant_unique UNIQUE (user_id, tenant_id);

-- =============================================================================
-- STEP 3: Update indexes
-- =============================================================================

DROP INDEX IF EXISTS idx_user_profiles_user;
CREATE INDEX idx_user_profiles_user_tenant ON user_profiles(user_id, tenant_id);

-- Comments
COMMENT ON CONSTRAINT user_profiles_user_tenant_fk ON user_profiles IS 'Composite foreign key to prevent tenant drift';
COMMENT ON CONSTRAINT user_profiles_user_tenant_unique ON user_profiles IS 'Ensure unique user-tenant combination';

-- =============================================================================
-- STEP 4: Verify data integrity
-- =============================================================================

-- This will fail if there are any inconsistencies
DO $$
DECLARE
    invalid_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO invalid_count
    FROM user_profiles up
    LEFT JOIN users u ON up.user_id = u.user_id AND up.tenant_id = u.tenant_id
    WHERE u.user_id IS NULL;
    
    IF invalid_count > 0 THEN
        RAISE EXCEPTION 'Found % user_profiles records with tenant drift', invalid_count;
    END IF;
END $$;
