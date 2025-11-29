-- Migration: Add migration tracking columns
-- Description: Track user migration status from password to Kanidm auth
-- Author: System
-- Date: 2025-01-10
-- Phase: 4.1 - Database Schema Updates

-- =============================================================================
-- STEP 1: Add auth_method column
-- =============================================================================

-- Track which authentication method user is using
ALTER TABLE users ADD COLUMN auth_method VARCHAR(50) NOT NULL DEFAULT 'password';

-- Add constraint for valid values
ALTER TABLE users ADD CONSTRAINT users_auth_method_check
  CHECK (auth_method IN ('password', 'kanidm', 'dual'));

COMMENT ON COLUMN users.auth_method IS
  'Authentication method: password (legacy only), kanidm (OAuth2 only), dual (both password and Kanidm)';

-- =============================================================================
-- STEP 2: Add migration tracking timestamps
-- =============================================================================

-- Track when migration invitation was sent
ALTER TABLE users ADD COLUMN migration_invited_at TIMESTAMPTZ;

-- Track when user completed migration (first Kanidm login)
ALTER TABLE users ADD COLUMN migration_completed_at TIMESTAMPTZ;

COMMENT ON COLUMN users.migration_invited_at IS
  'Timestamp when user was invited to migrate to Kanidm (email sent)';

COMMENT ON COLUMN users.migration_completed_at IS
  'Timestamp when user completed Kanidm migration (first successful OAuth2 login)';

-- =============================================================================
-- STEP 3: Update existing Kanidm users
-- =============================================================================

-- Mark users who already have Kanidm accounts
UPDATE users
SET auth_method = 'kanidm',
    migration_completed_at = COALESCE(kanidm_synced_at, NOW())
WHERE kanidm_user_id IS NOT NULL
  AND password_hash IS NULL;

-- Mark users with BOTH password and Kanidm as dual auth
UPDATE users
SET auth_method = 'dual',
    migration_completed_at = COALESCE(kanidm_synced_at, NOW())
WHERE kanidm_user_id IS NOT NULL
  AND password_hash IS NOT NULL;

-- =============================================================================
-- STEP 4: Create indexes for analytics
-- =============================================================================

-- Index for filtering by auth method
CREATE INDEX idx_users_auth_method ON users(auth_method)
  WHERE deleted_at IS NULL;

-- Index for migration analytics (by tenant)
CREATE INDEX idx_users_migration_status
  ON users(tenant_id, auth_method, migration_completed_at)
  WHERE deleted_at IS NULL;

-- Index for finding non-migrated active users
CREATE INDEX idx_users_pending_migration
  ON users(tenant_id, last_login_at)
  WHERE deleted_at IS NULL
    AND auth_method = 'password'
    AND kanidm_user_id IS NULL;

-- =============================================================================
-- ANALYTICS VIEWS
-- =============================================================================

-- Create view for migration progress by tenant
CREATE OR REPLACE VIEW v_migration_progress AS
SELECT
  t.tenant_id,
  t.name AS tenant_name,
  t.slug AS tenant_slug,
  COUNT(*) AS total_users,
  COUNT(*) FILTER (WHERE u.auth_method = 'password') AS password_only,
  COUNT(*) FILTER (WHERE u.auth_method = 'kanidm') AS kanidm_only,
  COUNT(*) FILTER (WHERE u.auth_method = 'dual') AS dual_auth,
  COUNT(*) FILTER (WHERE u.kanidm_user_id IS NOT NULL) AS migrated_users,
  ROUND(
    100.0 * COUNT(*) FILTER (WHERE u.kanidm_user_id IS NOT NULL) / NULLIF(COUNT(*), 0),
    2
  ) AS migration_percent,
  MAX(u.migration_completed_at) AS last_migration_at
FROM tenants t
LEFT JOIN users u ON t.tenant_id = u.tenant_id AND u.deleted_at IS NULL
WHERE t.deleted_at IS NULL
GROUP BY t.tenant_id, t.name, t.slug
ORDER BY migration_percent DESC NULLS LAST;

COMMENT ON VIEW v_migration_progress IS
  'Migration progress summary by tenant - shows auth method distribution and completion percentage';

-- =============================================================================
-- VERIFICATION QUERIES
-- =============================================================================

-- Check auth method distribution
-- SELECT auth_method, COUNT(*)
-- FROM users
-- WHERE deleted_at IS NULL
-- GROUP BY auth_method;

-- View migration progress
-- SELECT * FROM v_migration_progress;

-- Find users who need migration
-- SELECT email, full_name, last_login_at, migration_invited_at
-- FROM users
-- WHERE auth_method = 'password'
--   AND deleted_at IS NULL
--   AND status = 'active'
-- ORDER BY last_login_at DESC NULLS LAST
-- LIMIT 100;

-- Migration completion trend (daily)
-- SELECT
--   DATE(migration_completed_at) AS migration_date,
--   COUNT(*) AS users_migrated
-- FROM users
-- WHERE migration_completed_at >= CURRENT_DATE - INTERVAL '30 days'
-- GROUP BY DATE(migration_completed_at)
-- ORDER BY migration_date DESC;
