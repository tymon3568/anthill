-- Migration: Add Casbin policies for stock-levels endpoint
-- Description: Grants read access to stock-levels for admin, manager, and user roles
-- The stock-levels endpoint is read-only (GET only)

-- ============================================================================
-- STOCK LEVELS POLICIES
-- ============================================================================

-- Admin: Read access to stock levels
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/stock-levels', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Read access to stock levels
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/stock-levels', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Read access to stock levels
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/stock-levels', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read access to stock levels
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/stock-levels', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;
