-- Migration: Make password_hash nullable for Kanidm integration
-- Description: Allow users to authenticate via Kanidm without passwords
-- Author: System
-- Date: 2025-01-10
-- Phase: 4.1 - Database Schema Updates

-- =============================================================================
-- STEP 1: Make password_hash nullable
-- =============================================================================

-- Allow password_hash to be NULL (for Kanidm-only users)
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- Update comment to reflect deprecation
COMMENT ON COLUMN users.password_hash IS 
  'DEPRECATED: Bcrypt password hash. NULL for Kanidm-only users. Will be removed after full migration to Kanidm OAuth2.';

-- =============================================================================
-- IMPACT ANALYSIS
-- =============================================================================

-- This migration is SAFE and NON-BREAKING:
-- ✅ Existing users: password_hash values unchanged
-- ✅ Application: Backward compatible (password auth still works)
-- ✅ NEW users: Can be created with password_hash = NULL (Kanidm-only)
-- ✅ Dual auth: Users can have BOTH password and Kanidm authentication

-- =============================================================================
-- VERIFICATION QUERIES
-- =============================================================================

-- Check password distribution after migration
-- SELECT 
--   COUNT(*) AS total_users,
--   COUNT(password_hash) AS users_with_password,
--   COUNT(kanidm_user_id) AS users_with_kanidm,
--   COUNT(*) FILTER (WHERE password_hash IS NOT NULL AND kanidm_user_id IS NOT NULL) AS dual_auth_users,
--   COUNT(*) FILTER (WHERE password_hash IS NULL AND kanidm_user_id IS NOT NULL) AS kanidm_only_users
-- FROM users 
-- WHERE deleted_at IS NULL;
