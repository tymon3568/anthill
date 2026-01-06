-- Migration: Add Tenant Owner and Owner Role
-- Description: Add owner_user_id to tenants table and add 'owner' to users role constraint
-- Task: task_03.03.06_register_bootstrap_owner_and_default_role.md
-- Author: Claude
-- Date: 2026-01-06

-- =============================================================================
-- STEP 1: Add owner_user_id column to tenants table
-- =============================================================================

-- Add owner_user_id column (nullable initially to support existing data)
ALTER TABLE tenants
ADD COLUMN owner_user_id UUID REFERENCES users(user_id) ON DELETE SET NULL;

-- Create index for owner lookup
CREATE INDEX idx_tenants_owner ON tenants(owner_user_id) WHERE deleted_at IS NULL;

-- Comment
COMMENT ON COLUMN tenants.owner_user_id IS 'The user who owns this tenant (assigned on tenant creation during registration)';

-- =============================================================================
-- STEP 2: Update users role constraint to include 'owner'
-- =============================================================================

-- Drop existing constraint
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_role_check;

-- Add new constraint with 'owner' role included
-- Note: 'owner' is a system role that cannot be deleted or modified
ALTER TABLE users ADD CONSTRAINT users_role_check
    CHECK (role IN ('owner', 'super_admin', 'admin', 'manager', 'user', 'viewer'));

-- Comment update
COMMENT ON COLUMN users.role IS 'User role for RBAC. System roles (owner, super_admin, admin) are protected.';

-- =============================================================================
-- NOTE: The set_tenant_owner logic is implemented in Rust (PgTenantRepository::set_owner)
-- which is the single source of truth. No SQL function is needed.
--
-- For existing tenants without an owner, you may want to run:
--
-- UPDATE tenants t
-- SET owner_user_id = (
--     SELECT user_id FROM users u
--     WHERE u.tenant_id = t.tenant_id
--     AND u.role IN ('super_admin', 'admin')
--     AND u.deleted_at IS NULL
--     ORDER BY u.created_at ASC
--     LIMIT 1
-- )
-- WHERE t.owner_user_id IS NULL AND t.deleted_at IS NULL;
--
-- Then update those users' roles to 'owner':
-- UPDATE users
-- SET role = 'owner', updated_at = NOW()
-- WHERE user_id IN (SELECT owner_user_id FROM tenants WHERE owner_user_id IS NOT NULL);
-- =============================================================================
