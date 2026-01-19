-- Migration: Copy Casbin policies for existing tenants
-- Description: Copies all role policies from 'default_tenant' to all existing tenants
-- that don't have their own policies yet. This fixes the 403 Forbidden issue for
-- users who registered before the policy copy feature was added.
--
-- Root Cause: When users register and create a new tenant, they are assigned the
-- 'owner' role for their tenant. However, Casbin policies (the 'p' rules that define
-- what each role can do) only existed for 'default_tenant'. The Casbin matcher
-- requires r.dom == p.dom, so requests from other tenants were denied with 403.
--
-- Fix: This migration copies all policies from 'default_tenant' to each existing
-- tenant that doesn't have policies yet.
--
-- Author: Claude
-- Date: 2026-01-19

-- Copy policies for all existing tenants that don't have their own policies
-- We copy from 'default_tenant' (v1 column) to each tenant's ID
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3)
SELECT
    p.ptype,
    p.v0,           -- role name (owner, admin, manager, user)
    t.tenant_id::text,  -- target tenant ID
    p.v2,           -- resource path
    p.v3            -- action (GET, POST, etc.)
FROM casbin_rule p
CROSS JOIN tenants t
WHERE
    p.ptype = 'p'                           -- Only permission policies, not grouping policies
    AND p.v1 = 'default_tenant'             -- Source from default_tenant
    AND t.tenant_id::text != 'default_tenant'  -- Don't copy to default_tenant itself
    AND t.deleted_at IS NULL                -- Only active tenants
    AND NOT EXISTS (
        -- Check if this tenant already has this specific policy
        SELECT 1 FROM casbin_rule existing
        WHERE existing.ptype = 'p'
            AND existing.v0 = p.v0          -- Same role
            AND existing.v1 = t.tenant_id::text  -- Same target tenant
            AND existing.v2 = p.v2          -- Same resource
            AND existing.v3 = p.v3          -- Same action
    );

-- Log how many policies were copied (for debugging)
-- This is a PostgreSQL-specific way to see the result
DO $$
DECLARE
    tenant_count INTEGER;
    policy_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO tenant_count FROM tenants WHERE deleted_at IS NULL AND tenant_id::text != 'default_tenant';
    SELECT COUNT(*) INTO policy_count FROM casbin_rule WHERE ptype = 'p' AND v1 != 'default_tenant';

    RAISE NOTICE 'Casbin policy migration complete: % tenants, % non-default policies', tenant_count, policy_count;
END $$;
