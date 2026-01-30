-- Migration: Add Casbin policies for transfers and stock-takes endpoints
-- Description: Grants appropriate access to transfers and stock-takes for all roles
-- These endpoints are essential for Stock Movements UI functionality

-- ============================================================================
-- TRANSFERS POLICIES
-- ============================================================================

-- Admin: Full access to transfers
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/transfers', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/transfers', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/transfers/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/transfers/*', 'PUT', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/transfers/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full access to transfers
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/transfers', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/transfers', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/transfers/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/transfers/*', 'PUT', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/transfers/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Create and view transfers
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/transfers', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/transfers', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/transfers/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/transfers/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read-only access to transfers
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/transfers', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/transfers/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- ============================================================================
-- STOCK-TAKES POLICIES
-- ============================================================================

-- Admin: Full access to stock-takes
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/stock-takes', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/stock-takes', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/stock-takes/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/stock-takes/*', 'PUT', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.slug, '/api/v1/inventory/stock-takes/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full access to stock-takes
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/stock-takes', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/stock-takes', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/stock-takes/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/stock-takes/*', 'PUT', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.slug, '/api/v1/inventory/stock-takes/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Create and view stock-takes
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/stock-takes', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/stock-takes', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/stock-takes/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.slug, '/api/v1/inventory/stock-takes/*', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: Read-only access to stock-takes
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/stock-takes', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'viewer', t.slug, '/api/v1/inventory/stock-takes/*', 'GET', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;
