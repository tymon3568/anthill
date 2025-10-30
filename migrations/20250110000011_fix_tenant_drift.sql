-- Migration: Fix Tenant Drift Issue
-- Description: Add composite foreign key to prevent tenant drift in user_profiles
-- Author: Cascade
-- Date: 2025-10-28

-- =============================================================================
-- STEP 1: Add UNIQUE constraint on users table
-- =============================================================================

-- Note: This constraint is now created in migration 20250110000010_create_user_profiles.sql
-- This step is kept for backwards compatibility with existing databases
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'users_user_tenant_unique'
    ) THEN
        ALTER TABLE users 
        ADD CONSTRAINT users_user_tenant_unique UNIQUE (user_id, tenant_id);
    END IF;
END $$;

-- =============================================================================
-- STEP 2: Fix user_profiles foreign key
-- =============================================================================

-- Drop existing constraints if they exist
DO $$
BEGIN
    -- Drop user_id_key if exists
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_profiles_user_id_key' AND conrelid = 'user_profiles'::regclass) THEN
        ALTER TABLE user_profiles DROP CONSTRAINT user_profiles_user_id_key;
    END IF;
    
    -- Drop user_id_fkey if exists
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_profiles_user_id_fkey' AND conrelid = 'user_profiles'::regclass) THEN
        ALTER TABLE user_profiles DROP CONSTRAINT user_profiles_user_id_fkey;
    END IF;
END $$;

-- Add composite foreign key if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_profiles_user_tenant_fk' AND conrelid = 'user_profiles'::regclass) THEN
        ALTER TABLE user_profiles 
        ADD CONSTRAINT user_profiles_user_tenant_fk 
            FOREIGN KEY (user_id, tenant_id) 
            REFERENCES users(user_id, tenant_id) 
            ON DELETE CASCADE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_profiles_user_tenant_unique' AND conrelid = 'user_profiles'::regclass) THEN
        ALTER TABLE user_profiles 
        ADD CONSTRAINT user_profiles_user_tenant_unique UNIQUE (user_id, tenant_id);
    END IF;
END $$;

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
