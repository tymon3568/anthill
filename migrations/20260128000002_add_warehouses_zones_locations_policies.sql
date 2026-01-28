-- Migration: Add Casbin policies for warehouses zones and locations endpoints
-- Description: Grants CRUD access to warehouses, zones, and locations for owner, admin, manager roles
-- Related Bug: Zones/Locations API 403 Forbidden (BUGS_FIXED.md #5)

-- ============================================================================
-- WAREHOUSES CRUD POLICIES (POST, PUT, DELETE)
-- ============================================================================

-- Owner: Full CRUD access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Admin: Full CRUD access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full CRUD access to warehouses
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- ============================================================================
-- ZONES POLICIES (GET, POST, PUT, DELETE)
-- Path: /api/v1/inventory/warehouses/{id}/zones
-- ============================================================================

-- Owner: Full CRUD access to zones
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Admin: Full CRUD access to zones
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full CRUD access to zones
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Read access to zones
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read access to zones
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses/*/zones/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- ============================================================================
-- LOCATIONS POLICIES (GET, POST, PUT, DELETE)
-- Path: /api/v1/inventory/warehouses/{id}/locations
-- ============================================================================

-- Owner: Full CRUD access to locations
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Admin: Full CRUD access to locations
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full CRUD access to locations
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'POST', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'PUT', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'DELETE', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Read access to locations
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read access to locations
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses/*/locations/*', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- ============================================================================
-- WAREHOUSE TREE POLICIES (GET)
-- Path: /api/v1/inventory/warehouses/tree
-- ============================================================================

-- All roles: Read access to warehouse tree
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/warehouses/tree', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/warehouses/tree', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/warehouses/tree', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/warehouses/tree', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.tenant_id::text, '/api/v1/inventory/warehouses/tree', 'GET', '', ''
FROM tenants t WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;
