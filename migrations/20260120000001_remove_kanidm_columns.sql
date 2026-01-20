-- Migration: Remove Kanidm Integration
-- Description: Remove all Kanidm-related columns and tables as the system now uses self-hosted JWT auth only
-- Author: System
-- Date: 2026-01-20

-- =============================================================================
-- STEP 1: Drop Kanidm-related views
-- =============================================================================

-- Drop session stats view that references kanidm columns
DROP VIEW IF EXISTS v_session_stats;

-- =============================================================================
-- STEP 2: Drop Kanidm-related indexes and constraints
-- =============================================================================

-- Drop indexes on users table
DROP INDEX IF EXISTS idx_users_kanidm_id;
DROP INDEX IF EXISTS idx_users_pending_migration;

-- Drop unique constraint on kanidm_user_id
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_kanidm_user_id_key;

-- Drop auth_method constraint on users table (will be updated later)
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_auth_method_check;

-- Drop indexes on sessions table
DROP INDEX IF EXISTS idx_sessions_kanidm_session;

-- =============================================================================
-- STEP 3: Remove Kanidm columns from sessions table
-- =============================================================================

-- Remove kanidm_session_id column
ALTER TABLE sessions DROP COLUMN IF EXISTS kanidm_session_id;

-- Update auth_method constraint to only allow 'jwt' and 'password'
ALTER TABLE sessions DROP CONSTRAINT IF EXISTS sessions_auth_method_check;
ALTER TABLE sessions ADD CONSTRAINT sessions_auth_method_check
  CHECK (auth_method IN ('jwt', 'password'));

-- Update existing 'kanidm' or 'dual' auth methods to 'jwt'
UPDATE sessions SET auth_method = 'jwt' WHERE auth_method IN ('kanidm', 'dual');

-- =============================================================================
-- STEP 4: Remove Kanidm columns from users table
-- =============================================================================

-- Remove kanidm_user_id column (CASCADE to drop dependent constraints)
ALTER TABLE users DROP COLUMN IF EXISTS kanidm_user_id CASCADE;

-- Remove kanidm_synced_at column
ALTER TABLE users DROP COLUMN IF EXISTS kanidm_synced_at CASCADE;

-- Update auth_method to 'password' for any kanidm/dual users
UPDATE users SET auth_method = 'password' WHERE auth_method IN ('kanidm', 'dual');

-- Add new auth_method constraint (only password allowed now)
ALTER TABLE users ADD CONSTRAINT users_auth_method_check
  CHECK (auth_method IN ('password'));

-- =============================================================================
-- STEP 5: Drop kanidm_tenant_groups table
-- =============================================================================

DROP TABLE IF EXISTS kanidm_tenant_groups;

-- =============================================================================
-- STEP 6: Recreate session stats view without Kanidm references
-- =============================================================================

CREATE OR REPLACE VIEW v_session_stats AS
SELECT
  auth_method,
  COUNT(*) AS total_sessions,
  COUNT(*) FILTER (WHERE revoked = FALSE) AS active_sessions,
  COUNT(*) FILTER (WHERE access_token_expires_at > NOW()) AS valid_sessions,
  AVG(EXTRACT(EPOCH FROM (NOW() - created_at)) / 3600)::INTEGER AS avg_age_hours,
  MAX(last_used_at) AS most_recent_use
FROM sessions
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
GROUP BY auth_method
ORDER BY total_sessions DESC;

COMMENT ON VIEW v_session_stats IS
  'Session statistics by authentication method (last 7 days)';

-- =============================================================================
-- VERIFICATION
-- =============================================================================

-- Verify columns are removed
-- SELECT column_name FROM information_schema.columns
-- WHERE table_name = 'users' AND column_name LIKE 'kanidm%';

-- Verify table is dropped
-- SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'kanidm_tenant_groups');
