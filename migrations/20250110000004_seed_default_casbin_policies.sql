-- Migration: Seed default Casbin RBAC policies
-- Description: Creates default roles (admin, manager, user) with permissions for each tenant
-- Note: This creates baseline policies. Tenants will have their own isolated policies.

-- ============================================================================
-- DEFAULT ROLE POLICIES
-- ============================================================================

-- ADMIN ROLE: Full access to all resources
-- Format: (ptype, subject, domain, resource, action)

-- Users management (admin only)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/users', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/users', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/users/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/users/*', 'PATCH'),
('p', 'admin', 'default_tenant', '/api/v1/users/*', 'DELETE');

-- Products management (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/inventory/products', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/products', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/products/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/products/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/products/*', 'DELETE');

-- Warehouses management
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/inventory/warehouses', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/warehouses', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/warehouses/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/warehouses/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/warehouses/*', 'DELETE');

-- Stock operations
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/inventory/stock', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/receipts', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/receipts', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/receipts/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/receipts/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/deliveries', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/deliveries', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/deliveries/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/deliveries/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/transfers', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/transfers', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/transfers/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/inventory/transfers/*', 'PUT');

-- Orders (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/orders', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/orders', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/orders/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/orders/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/orders/*', 'DELETE');

-- Integrations (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', 'default_tenant', '/api/v1/integrations', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/integrations', 'POST'),
('p', 'admin', 'default_tenant', '/api/v1/integrations/*', 'GET'),
('p', 'admin', 'default_tenant', '/api/v1/integrations/*', 'PUT'),
('p', 'admin', 'default_tenant', '/api/v1/integrations/*', 'DELETE');

-- ============================================================================
-- MANAGER ROLE: Limited management access (no user management)
-- ============================================================================

-- Products management (CRUD)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/inventory/products', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/products', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/products/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/products/*', 'PUT');

-- Warehouses (read-only for manager)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/inventory/warehouses', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/warehouses/*', 'GET');

-- Stock operations (full CRUD)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/inventory/stock', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/receipts', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/receipts', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/receipts/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/receipts/*', 'PUT'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/deliveries', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/deliveries', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/deliveries/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/deliveries/*', 'PUT'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/transfers', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/transfers', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/transfers/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/inventory/transfers/*', 'PUT');

-- Orders (full CRUD)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/orders', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/orders', 'POST'),
('p', 'manager', 'default_tenant', '/api/v1/orders/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/orders/*', 'PUT');

-- Integrations (read + sync, no delete)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'manager', 'default_tenant', '/api/v1/integrations', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/integrations/*', 'GET'),
('p', 'manager', 'default_tenant', '/api/v1/integrations/*/sync', 'POST');

-- ============================================================================
-- USER ROLE: Read-only access
-- ============================================================================

-- Users (can only view own profile - implemented in handler logic)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/users', 'GET');

-- Products (read-only)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/inventory/products', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/products/*', 'GET');

-- Warehouses (read-only)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/inventory/warehouses', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/warehouses/*', 'GET');

-- Stock (read-only)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/inventory/stock', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/receipts', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/receipts/*', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/deliveries', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/inventory/deliveries/*', 'GET');

-- Orders (read-only)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/orders', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/orders/*', 'GET');

-- Integrations (read-only)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'user', 'default_tenant', '/api/v1/integrations', 'GET'),
('p', 'user', 'default_tenant', '/api/v1/integrations/*', 'GET');

-- ============================================================================
-- HELPER VIEWS for easy policy management
-- ============================================================================

-- View: List all role-based policies
CREATE OR REPLACE VIEW casbin_policies AS
SELECT 
    id,
    v0 AS role,
    v1 AS tenant_id,
    v2 AS resource,
    v3 AS action
FROM casbin_rule
WHERE ptype = 'p'
ORDER BY v0, v2, v3;

-- View: List all role assignments
CREATE OR REPLACE VIEW casbin_role_assignments AS
SELECT 
    id,
    v0 AS user_id,
    v1 AS role,
    v2 AS tenant_id
FROM casbin_rule
WHERE ptype = 'g'
ORDER BY v2, v1, v0;

-- ============================================================================
-- NOTES:
-- ============================================================================
-- 1. These are DEFAULT policies for 'default_tenant'
-- 2. When a new tenant is created, copy these policies with their tenant_id
-- 3. Role assignments (ptype='g') are created when users are assigned roles
-- 4. Use the helper functions in shared/auth/enforcer.rs:
--    - add_policy() to add new permissions
--    - add_role_for_user() to assign roles to users
--    - remove_policy() to revoke permissions
-- 5. Tenant isolation is enforced by the 'domain' (v1) field in Casbin model
