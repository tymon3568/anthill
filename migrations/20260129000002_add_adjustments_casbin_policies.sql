-- Migration: Add Casbin policies for stock adjustments endpoint
-- Description: Grants appropriate access to adjustments for all roles
-- Stock adjustments allow increasing/decreasing inventory with reason codes

-- ============================================================================
-- ADJUSTMENTS POLICIES
-- ============================================================================

-- Admin: Full access to adjustments
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/adjustments', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/adjustments', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/adjustments/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/adjustments/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/adjustments/summary', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full access to adjustments
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/adjustments', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/adjustments', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/adjustments/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/adjustments/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/adjustments/summary', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Create and view adjustments (no posting/canceling)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/adjustments', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/adjustments', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/adjustments/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/adjustments/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/adjustments/summary', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read-only access to adjustments
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/adjustments', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/adjustments/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/adjustments/summary', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Owner: Full access to adjustments (owner role from owner_policies)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.slug, '/api/v1/inventory/adjustments', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.slug, '/api/v1/inventory/adjustments', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.slug, '/api/v1/inventory/adjustments/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.slug, '/api/v1/inventory/adjustments/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.slug, '/api/v1/inventory/adjustments/summary', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;
