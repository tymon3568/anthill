-- Migration: Add Casbin policies for transfer action endpoints (confirm, receive, cancel)
-- Description: The existing /api/v1/inventory/transfers/* policy doesn't match nested paths like /{id}/confirm
-- These explicit policies are needed for the transfer workflow actions

-- ============================================================================
-- TRANSFER ACTION POLICIES (confirm, receive, cancel)
-- ============================================================================

-- Owner: Full access to transfer actions
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/transfers/*/confirm', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/transfers/*/receive', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'owner', t.tenant_id::text, '/api/v1/inventory/transfers/*/cancel', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Admin: Full access to transfer actions
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/transfers/*/confirm', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/transfers/*/receive', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'admin', t.tenant_id::text, '/api/v1/inventory/transfers/*/cancel', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Manager: Full access to transfer actions
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/transfers/*/confirm', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/transfers/*/receive', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'manager', t.tenant_id::text, '/api/v1/inventory/transfers/*/cancel', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- User: Allow confirm and receive (but not cancel - requires manager approval)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/transfers/*/confirm', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
SELECT 'p', 'user', t.tenant_id::text, '/api/v1/inventory/transfers/*/receive', 'POST', '', ''
FROM tenants t
WHERE t.deleted_at IS NULL
ON CONFLICT (ptype, v0, v1, v2, v3, v4, v5) DO NOTHING;

-- Viewer: No access to transfer actions (read-only role)
