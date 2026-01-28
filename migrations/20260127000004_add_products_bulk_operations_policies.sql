-- Add Casbin policies for product bulk operations
-- These policies allow admin and owner roles to perform bulk activate, deactivate, and delete operations

-- Owner role: Full bulk operations access
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/inventory/products/bulk/*', 'POST')
ON CONFLICT DO NOTHING;

-- Admin role: Full bulk operations access
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/inventory/products/bulk/*', 'POST')
ON CONFLICT DO NOTHING;

-- Manager role: Activate and deactivate only (no bulk delete)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/inventory/products/bulk/activate', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/products/bulk/deactivate', 'POST')
ON CONFLICT DO NOTHING;
