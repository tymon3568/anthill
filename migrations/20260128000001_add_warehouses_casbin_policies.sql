-- Migration: Add Casbin policies for warehouses endpoint (all tenants)
-- Description: Grants read access to warehouses for owner, admin, manager, user, and viewer roles
-- This fixes 403 Forbidden error when accessing warehouses from non-default tenants
-- Related Bug: Warehouses API 403 Forbidden (BUGS_FIXED.md #3)

-- ============================================================================
-- WAREHOUSES POLICIES (Dynamic - All Tenants)
-- ============================================================================

-- Owner: Read access to warehouses (highest privilege role)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Admin: Read access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Read access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Read access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- ============================================================================
-- WAREHOUSES BY ID POLICIES (Dynamic - All Tenants)
-- ============================================================================

-- Owner: Read access to individual warehouse
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Admin: Read access to individual warehouse
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Read access to individual warehouse
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Read access to individual warehouse
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read access to individual warehouse
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;
