-- Migration: Add AuthZ Versioning (Hybrid: Tenant + User)
-- Description: Adds authz_version columns to tenants and users to support immediate-effect authorization changes
-- Date: 2026-01-02
--
-- Hybrid versioning strategy:
-- - tenants.authz_version bumps when role/policy definitions change (tenant-wide invalidation)
-- - users.authz_version bumps when a specific user's effective access/security state changes (user-level invalidation)
--
-- Notes:
-- - Versions start at 1 for all existing rows.
-- - BIGINT is used to avoid wrap risk and allow monotonic increments over time.

-- =============================================================================
-- TENANTS: add authz_version
-- =============================================================================

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS authz_version BIGINT NOT NULL DEFAULT 1;

COMMENT ON COLUMN tenants.authz_version IS
    'Authorization version for tenant-wide invalidation (bump on role/policy changes)';

-- =============================================================================
-- USERS: add authz_version
-- =============================================================================

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS authz_version BIGINT NOT NULL DEFAULT 1;

COMMENT ON COLUMN users.authz_version IS
    'Authorization version for user-level invalidation (bump on role/status/security changes)';

-- =============================================================================
-- INDEXES
-- =============================================================================
-- Tenants are looked up by tenant_id (PK), so an additional index on authz_version is generally unnecessary.
-- For users, version checks can be performed by user_id (PK) or (tenant_id, user_id).
-- A partial composite index can help if queries frequently filter on tenant_id and join on user_id for active users.

CREATE INDEX IF NOT EXISTS idx_users_authz_version_active
    ON users(tenant_id, user_id, authz_version)
    WHERE deleted_at IS NULL;

-- =============================================================================
-- BACKFILL
-- =============================================================================
-- Defaults handle existing rows automatically due to NOT NULL DEFAULT 1.
-- No explicit UPDATE is required.
