-- Migration: Add audit columns to casbin_rule table
-- Purpose: Enable policy change tracking per AUTHORIZATION_RBAC_STRATEGY.md requirements
-- - Track who created/modified policies
-- - Support policy rollback scenarios
-- - Enable audit trail for compliance

-- Add audit columns to casbin_rule table
ALTER TABLE casbin_rule
    ADD COLUMN IF NOT EXISTS created_by UUID,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

-- Add foreign key constraint for created_by (optional, may fail if users don't exist)
-- Using DO block to gracefully handle cases where the constraint already exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_casbin_rule_created_by'
        AND table_name = 'casbin_rule'
    ) THEN
        -- Note: We don't add a hard FK constraint because:
        -- 1. Seed policies are created without a user context
        -- 2. Policy migrations run before users exist
        -- Instead, we document this as a soft reference
        COMMENT ON COLUMN casbin_rule.created_by IS 'UUID of the user who created this policy (NULL for seed/migration policies)';
    END IF;
END $$;

-- Add comment for updated_at
COMMENT ON COLUMN casbin_rule.updated_at IS 'Timestamp of last policy modification';

-- Create index for querying policies by creator (useful for audit)
CREATE INDEX IF NOT EXISTS idx_casbin_rule_created_by
    ON casbin_rule(created_by)
    WHERE created_by IS NOT NULL;

-- Create index for querying recently modified policies
CREATE INDEX IF NOT EXISTS idx_casbin_rule_updated_at
    ON casbin_rule(updated_at DESC);

-- ============================================================================
-- Tenant AuthZ Version Table
-- Per AUTHORIZATION_RBAC_STRATEGY.md: "Per-tenant policy_version integer"
-- ============================================================================

-- Create tenant_authz_versions table for fast policy invalidation
CREATE TABLE IF NOT EXISTS tenant_authz_versions (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    policy_version BIGINT NOT NULL DEFAULT 1,
    last_policy_change_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_policy_change_by UUID,
    last_policy_change_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add comment for documentation
COMMENT ON TABLE tenant_authz_versions IS 'Per-tenant authorization version for fast policy cache invalidation. Version is bumped on any policy or role change.';
COMMENT ON COLUMN tenant_authz_versions.policy_version IS 'Incrementing version number. Cache keys include this value for instant invalidation.';
COMMENT ON COLUMN tenant_authz_versions.last_policy_change_reason IS 'Human-readable reason for the last version bump (e.g., "add_role", "remove_user_role")';

-- Create index for fast lookups
CREATE INDEX IF NOT EXISTS idx_tenant_authz_versions_policy_version
    ON tenant_authz_versions(tenant_id, policy_version);

-- Initialize authz versions for all existing tenants
INSERT INTO tenant_authz_versions (tenant_id, policy_version, last_policy_change_reason)
SELECT tenant_id, 1, 'initial_setup'
FROM tenants
WHERE deleted_at IS NULL
ON CONFLICT (tenant_id) DO NOTHING;

-- ============================================================================
-- User AuthZ Version Table
-- For per-user immediate revocation (e.g., password reset, suspend)
-- ============================================================================

-- Create user_authz_versions table for user-level fast revocation
CREATE TABLE IF NOT EXISTS user_authz_versions (
    user_id UUID PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
    authz_version BIGINT NOT NULL DEFAULT 1,
    last_authz_change_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_authz_change_by UUID,
    last_authz_change_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add comment for documentation
COMMENT ON TABLE user_authz_versions IS 'Per-user authorization version for immediate permission revocation. Bumped on role changes, suspension, password reset, etc.';

-- Create index for fast lookups
CREATE INDEX IF NOT EXISTS idx_user_authz_versions_authz_version
    ON user_authz_versions(user_id, authz_version);

-- Initialize authz versions for all existing users
INSERT INTO user_authz_versions (user_id, authz_version, last_authz_change_reason)
SELECT user_id, 1, 'initial_setup'
FROM users
WHERE deleted_at IS NULL
ON CONFLICT (user_id) DO NOTHING;

-- ============================================================================
-- Trigger to auto-update tenant authz version on casbin_rule changes
-- ============================================================================

CREATE OR REPLACE FUNCTION bump_tenant_authz_version_on_casbin_change()
RETURNS TRIGGER AS $$
DECLARE
    affected_tenant_id UUID;
BEGIN
    -- Extract tenant_id from v1 column (domain in Casbin model)
    IF TG_OP = 'DELETE' THEN
        -- Try to cast v1 to UUID, skip if it's not a valid UUID (e.g., 'default_tenant')
        BEGIN
            affected_tenant_id := OLD.v1::UUID;
        EXCEPTION WHEN invalid_text_representation THEN
            -- v1 is not a valid UUID, skip trigger
            RETURN OLD;
        END;
    ELSE
        BEGIN
            affected_tenant_id := NEW.v1::UUID;
        EXCEPTION WHEN invalid_text_representation THEN
            -- v1 is not a valid UUID, skip trigger
            RETURN NEW;
        END;
    END IF;

    -- Bump the tenant's policy version
    INSERT INTO tenant_authz_versions (tenant_id, policy_version, last_policy_change_at, last_policy_change_reason)
    VALUES (affected_tenant_id, 1, NOW(), TG_OP || '_policy')
    ON CONFLICT (tenant_id) DO UPDATE
    SET
        policy_version = tenant_authz_versions.policy_version + 1,
        last_policy_change_at = NOW(),
        last_policy_change_reason = TG_OP || '_policy',
        updated_at = NOW();

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on casbin_rule table
DROP TRIGGER IF EXISTS trg_bump_tenant_authz_version ON casbin_rule;
CREATE TRIGGER trg_bump_tenant_authz_version
    AFTER INSERT OR UPDATE OR DELETE ON casbin_rule
    FOR EACH ROW
    EXECUTE FUNCTION bump_tenant_authz_version_on_casbin_change();

-- Add comment for trigger documentation
COMMENT ON FUNCTION bump_tenant_authz_version_on_casbin_change() IS 'Automatically bumps tenant policy_version when casbin_rule is modified. Enables instant cache invalidation.';
