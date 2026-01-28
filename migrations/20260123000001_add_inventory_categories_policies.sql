-- Migration: Add Casbin policies for inventory categories
-- Description: Adds authorization policies for category management endpoints
-- Date: 2026-01-23

-- ============================================================================
-- OWNER ROLE: Category management (full access)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories/*', 'PATCH'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories/tree', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/categories/bulk/*', 'POST')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- ADMIN ROLE: Category management (full access)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories/*', 'PATCH'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories/*', 'DELETE'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories/tree', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/categories/bulk/*', 'POST')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- MANAGER ROLE: Category management (read + write, no delete)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/inventory/categories', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/categories', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/categories/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/categories/*', 'PUT'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/categories/*', 'PATCH'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/categories/tree', 'GET')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- USER ROLE: Category viewing (read-only)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/inventory/categories', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/categories/*', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/categories/tree', 'GET')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Copy policies to existing tenants
-- ============================================================================
-- Insert category policies for all existing tenants (except default_tenant which already has them)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3)
SELECT DISTINCT 'p', r.v0, t.tenant_id::text, p.v2, p.v3
FROM casbin_rule r
CROSS JOIN tenants t
CROSS JOIN (
    SELECT v2, v3 FROM casbin_rule
    WHERE ptype = 'p'
    AND v1 = 'default_tenant'
    AND v2 LIKE '/api/v1/inventory/categories%'
) p
WHERE r.ptype = 'g'
AND r.v2 = t.tenant_id::text
AND t.tenant_id::text != 'default_tenant'
ON CONFLICT DO NOTHING;
