-- Migration: Fix Casbin Views
-- Description: Drop and recreate casbin views to avoid column mismatch
-- Author: System
-- Date: 2025-01-10

-- Drop existing views
DROP VIEW IF EXISTS casbin_policies CASCADE;
DROP VIEW IF EXISTS casbin_role_assignments CASCADE;

-- Recreate views with correct structure
CREATE OR REPLACE VIEW casbin_policies AS
SELECT
    id,
    v0 AS role,
    v1 AS tenant_id,
    v2 AS resource,
    v3 AS action
FROM casbin_rule
WHERE ptype = 'p'
ORDER BY v0, v2, v3;

CREATE OR REPLACE VIEW casbin_role_assignments AS
SELECT
    id,
    v0 AS user_id,
    v1 AS role,
    v2 AS tenant_id
FROM casbin_rule
WHERE ptype = 'g'
ORDER BY v2, v1, v0;
