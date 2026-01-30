-- Migration: Add Casbin policies for product images endpoints
-- Date: 2026-01-30
-- Description: Authorization rules for product image management
-- TaskID: 08.10.06.02

-- ============================================================================
-- Product Images Policies
-- ============================================================================
-- Endpoint pattern: /api/v1/inventory/products/{id}/images/*
-- All inventory roles (owner, admin, inventory_manager, warehouse_staff) can manage images

-- Owner role - full access
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'owner', '*', '/api/v1/inventory/products/*/images', 'GET'),
('p', 'owner', '*', '/api/v1/inventory/products/*/images', 'POST'),
('p', 'owner', '*', '/api/v1/inventory/products/*/images/*', 'GET'),
('p', 'owner', '*', '/api/v1/inventory/products/*/images/*', 'PUT'),
('p', 'owner', '*', '/api/v1/inventory/products/*/images/*', 'DELETE'),
('p', 'owner', '*', '/api/v1/inventory/products/*/images/reorder', 'PUT'),
('p', 'owner', '*', '/api/v1/inventory/products/*/images/*/primary', 'PUT')
ON CONFLICT DO NOTHING;

-- Admin role - full access
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'admin', '*', '/api/v1/inventory/products/*/images', 'GET'),
('p', 'admin', '*', '/api/v1/inventory/products/*/images', 'POST'),
('p', 'admin', '*', '/api/v1/inventory/products/*/images/*', 'GET'),
('p', 'admin', '*', '/api/v1/inventory/products/*/images/*', 'PUT'),
('p', 'admin', '*', '/api/v1/inventory/products/*/images/*', 'DELETE'),
('p', 'admin', '*', '/api/v1/inventory/products/*/images/reorder', 'PUT'),
('p', 'admin', '*', '/api/v1/inventory/products/*/images/*/primary', 'PUT')
ON CONFLICT DO NOTHING;

-- Inventory Manager role - full access
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images', 'GET'),
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images', 'POST'),
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images/*', 'GET'),
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images/*', 'PUT'),
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images/*', 'DELETE'),
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images/reorder', 'PUT'),
('p', 'inventory_manager', '*', '/api/v1/inventory/products/*/images/*/primary', 'PUT')
ON CONFLICT DO NOTHING;

-- Warehouse Staff role - read-only access (can view but not modify)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'warehouse_staff', '*', '/api/v1/inventory/products/*/images', 'GET'),
('p', 'warehouse_staff', '*', '/api/v1/inventory/products/*/images/*', 'GET')
ON CONFLICT DO NOTHING;

-- Sales Staff role - read-only access
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'sales_staff', '*', '/api/v1/inventory/products/*/images', 'GET'),
('p', 'sales_staff', '*', '/api/v1/inventory/products/*/images/*', 'GET')
ON CONFLICT DO NOTHING;

-- Member role - read-only access (for customer-facing apps)
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES
('p', 'member', '*', '/api/v1/inventory/products/*/images', 'GET'),
('p', 'member', '*', '/api/v1/inventory/products/*/images/*', 'GET')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- DOWN (for rollback)
-- ============================================================================
-- To rollback, run these commands manually:
--
-- DELETE FROM casbin_rule WHERE v2 LIKE '/api/v1/inventory/products/*/images%';
