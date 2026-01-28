-- Migration: Add Casbin policies for inventory variants
-- Description: Adds authorization policies for product variant management endpoints
-- Date: 2026-01-27

-- ============================================================================
-- OWNER ROLE: Variant management (full access)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants/by-sku/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants/by-barcode/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/variants/bulk/*', 'POST')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- ADMIN ROLE: Variant management (full access)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants/*', 'DELETE'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants/by-sku/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants/by-barcode/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/variants/bulk/*', 'POST')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- MANAGER ROLE: Variant management (read + write, no delete)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants/*', 'PUT'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants/by-sku/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants/by-barcode/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants/bulk/activate', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/variants/bulk/deactivate', 'POST')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- USER ROLE: Variant viewing (read-only)
-- ============================================================================
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/inventory/variants', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/variants/*', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/variants/by-sku/*', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/variants/by-barcode/*', 'GET')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Copy policies to existing tenants
-- ============================================================================
-- Insert variant policies for all existing tenants (except default_tenant which already has them)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3)
SELECT DISTINCT 'p', r.v0, t.tenant_id::text, p.v2, p.v3
FROM casbin_rule r
CROSS JOIN tenants t
CROSS JOIN (
    SELECT v2, v3 FROM casbin_rule
    WHERE ptype = 'p'
    AND v1 = 'default_tenant'
    AND v2 LIKE '/api/v1/inventory/variants%'
) p
WHERE r.ptype = 'g'
AND r.v2 = t.tenant_id::text
AND t.tenant_id::text != 'default_tenant'
ON CONFLICT DO NOTHING;
