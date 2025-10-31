-- Migration: Create Casbin RBAC Tables
-- Description: Tables for storing Casbin policies and role assignments
-- Author: System
-- Date: 2025-01-10
-- Reference: https://github.com/casbin-rs/sqlx-adapter

-- =============================================================================
-- TABLE: casbin_rule
-- =============================================================================

-- Casbin uses this table to store all policies and role assignments
-- The table structure follows the Casbin adapter specification

CREATE TABLE casbin_rule (
    id SERIAL PRIMARY KEY,

    -- Policy type (p = policy, g = grouping/role)
    ptype VARCHAR(12) NOT NULL,

    -- Subject (user_id or role name)
    v0 VARCHAR(128) NOT NULL,

    -- Tenant ID (for multi-tenancy)
    v1 VARCHAR(128) NOT NULL,

    -- Resource (e.g., "/api/v1/products")
    v2 VARCHAR(128) NOT NULL,

    -- Action (e.g., "GET", "POST", "DELETE")
    v3 VARCHAR(128) NOT NULL,

    -- Additional fields (for future extensibility)
    v4 VARCHAR(128) DEFAULT '',
    v5 VARCHAR(128) DEFAULT '',

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint to prevent duplicate rules
    CONSTRAINT casbin_rule_unique UNIQUE (ptype, v0, v1, v2, v3, v4, v5)
);

-- Indexes for efficient policy lookups
CREATE INDEX idx_casbin_rule_ptype ON casbin_rule(ptype);
CREATE INDEX idx_casbin_rule_v0 ON casbin_rule(v0); -- Subject lookup
CREATE INDEX idx_casbin_rule_v1 ON casbin_rule(v1); -- Tenant lookup
CREATE INDEX idx_casbin_rule_composite ON casbin_rule(ptype, v0, v1); -- Common query pattern

-- Comments
COMMENT ON TABLE casbin_rule IS 'Casbin RBAC rules storage (policies and role assignments)';
COMMENT ON COLUMN casbin_rule.ptype IS 'Policy type: p=policy, g=grouping/role';
COMMENT ON COLUMN casbin_rule.v0 IS 'Subject: user_id or role name';
COMMENT ON COLUMN casbin_rule.v1 IS 'Tenant ID for multi-tenant isolation';
COMMENT ON COLUMN casbin_rule.v2 IS 'Resource path (e.g., /api/v1/products)';
COMMENT ON COLUMN casbin_rule.v3 IS 'Action/HTTP method (e.g., GET, POST)';

-- =============================================================================
-- SEED DATA: Default Policies (Optional - for development)
-- =============================================================================

-- Example policies for common roles
-- In production, these should be managed through an admin interface

-- Super Admin: Full access to everything
-- INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
-- ('p', 'super_admin', '*', '/*', '*');

-- Admin: Full access within their tenant
-- INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
-- ('p', 'admin', '{tenant_id}', '/api/v1/*', '*');

-- Manager: Can read and update, but not delete
-- INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
-- ('p', 'manager', '{tenant_id}', '/api/v1/products', 'GET'),
-- ('p', 'manager', '{tenant_id}', '/api/v1/products', 'POST'),
-- ('p', 'manager', '{tenant_id}', '/api/v1/products', 'PUT'),
-- ('p', 'manager', '{tenant_id}', '/api/v1/inventory', 'GET'),
-- ('p', 'manager', '{tenant_id}', '/api/v1/inventory', 'POST'),
-- ('p', 'manager', '{tenant_id}', '/api/v1/orders', 'GET');

-- User: Read-only access
-- INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
-- ('p', 'user', '{tenant_id}', '/api/v1/products', 'GET'),
-- ('p', 'user', '{tenant_id}', '/api/v1/inventory', 'GET'),
-- ('p', 'user', '{tenant_id}', '/api/v1/orders', 'GET');

-- Viewer: Very limited read access
-- INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
-- ('p', 'viewer', '{tenant_id}', '/api/v1/dashboard', 'GET'),
-- ('p', 'viewer', '{tenant_id}', '/api/v1/reports', 'GET');

-- =============================================================================
-- HELPER VIEWS (Optional - for easier policy management)
-- =============================================================================

-- View to see all policies in a more readable format
CREATE OR REPLACE VIEW casbin_policies AS
SELECT
    id,
    ptype AS policy_type,
    v0 AS subject,
    v1 AS tenant_id,
    v2 AS resource,
    v3 AS action,
    created_at
FROM casbin_rule
WHERE ptype = 'p';

COMMENT ON VIEW casbin_policies IS 'Human-readable view of Casbin policies';

-- View to see all role assignments
CREATE OR REPLACE VIEW casbin_role_assignments AS
SELECT
    id,
    v0 AS user_id,
    v1 AS role,
    v2 AS tenant_id,
    created_at
FROM casbin_rule
WHERE ptype = 'g';

COMMENT ON VIEW casbin_role_assignments IS 'Human-readable view of Casbin role assignments';
