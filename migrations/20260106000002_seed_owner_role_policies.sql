-- Migration: Seed owner role policies
-- Description: Creates default policies for the 'owner' role (tenant creator)
-- Owner has all permissions of admin plus tenant-level management
-- Task: task_03.03.06_register_bootstrap_owner_and_default_role.md
-- Author: Claude
-- Date: 2026-01-06

-- ============================================================================
-- OWNER ROLE: Full access + Tenant management
-- ============================================================================
-- Owner is the tenant creator and has superset of admin permissions.
-- Owner can manage tenant settings, billing, and perform destructive operations.

-- Users management (full access including dangerous operations)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/users', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/users', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/users/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/users/*', 'PATCH'),
('p', 'owner', 'default_tenant', '/api/v1/users/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/users/*/suspend', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/users/*/activate', 'POST');

-- Admin endpoints (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/admin/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/*', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/admin/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/admin/*', 'PATCH'),
('p', 'owner', 'default_tenant', '/api/v1/admin/*', 'DELETE');

-- Roles and policies management (explicit paths needed - keyMatch2 wildcard only matches single segments)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/admin/roles', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/roles', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/admin/roles/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/roles/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/admin/roles/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/admin/policies', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/policies', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/admin/policies/*', 'DELETE');

-- Tenant management (owner-only, NOT under /admin so explicitly defined)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/tenant', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/tenant', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/settings', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/settings', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/billing', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/billing', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/plan', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/plan', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/danger/*', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/export', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/tenant/delete', 'POST');

-- Invitations management (explicit paths needed - keyMatch2 wildcard only matches single segments)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/admin/users/invite', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/admin/invitations', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/invitations/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/invitations/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/admin/invitations/*/resend', 'POST');

-- Products management (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/inventory/products', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/products', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/products/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/products/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/products/*', 'DELETE');

-- Warehouses management (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/inventory/warehouses', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/warehouses', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/warehouses/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/warehouses/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/warehouses/*', 'DELETE');

-- Stock operations (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/inventory/stock', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/receipts', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/receipts', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/receipts/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/receipts/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/receipts/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/deliveries', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/deliveries', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/deliveries/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/deliveries/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/deliveries/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/transfers', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/transfers', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/transfers/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/transfers/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/inventory/transfers/*', 'DELETE');

-- Orders (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/orders', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/orders', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/orders/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/orders/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/orders/*', 'DELETE');

-- Integrations (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/integrations', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/integrations', 'POST'),
('p', 'owner', 'default_tenant', '/api/v1/integrations/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/integrations/*', 'PUT'),
('p', 'owner', 'default_tenant', '/api/v1/integrations/*', 'DELETE'),
('p', 'owner', 'default_tenant', '/api/v1/integrations/*/sync', 'POST');

-- Audit logs (explicit paths needed - keyMatch2 wildcard only matches single segments)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/admin/audit-logs', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/admin/audit-logs/*', 'GET');

-- Analytics and reports (full access)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', 'default_tenant', '/api/v1/analytics/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/reports/*', 'GET'),
('p', 'owner', 'default_tenant', '/api/v1/reports/*', 'POST');

-- ============================================================================
-- NOTES:
-- ============================================================================
-- 1. Owner role is assigned automatically when a user creates a new tenant
-- 2. There should be exactly ONE owner per tenant (enforced by application logic)
-- 3. Owner cannot be demoted unless ownership is transferred first
-- 4. Owner has access to tenant-level dangerous operations (export, delete)
-- 5. When a new tenant is created, copy these policies with their tenant_id
--    using the SQL function or application logic
-- 6. Owner policies include /api/v1/admin/* wildcard for future admin endpoints
