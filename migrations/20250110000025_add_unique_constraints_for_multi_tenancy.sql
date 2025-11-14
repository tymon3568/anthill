-- Migration: Add UNIQUE constraints for multi-tenancy composite FKs
-- Description: Adds UNIQUE constraints on (tenant_id, id) for tables referenced by stock_adjustments
-- Dependencies: All referenced tables must exist
-- Created: 2025-10-29

-- ==================================
-- UNIQUE CONSTRAINTS FOR MULTI-TENANCY
-- ==================================
-- These constraints enable composite foreign keys that enforce tenant isolation
-- and prevent cross-tenant data references in stock_adjustments table.

-- stock_moves: Ensure (tenant_id, move_id) is unique for FK reference
-- Note: move_id is already PK, but we add explicit constraint for clarity
ALTER TABLE stock_moves
ADD CONSTRAINT stock_moves_tenant_move_unique
UNIQUE (tenant_id, move_id) DEFERRABLE INITIALLY DEFERRED;

-- products: Ensure (tenant_id, product_id) is unique for FK reference
-- Note: product_id is already PK, but we add explicit constraint for clarity
ALTER TABLE products
ADD CONSTRAINT products_tenant_product_unique
UNIQUE (tenant_id, product_id) DEFERRABLE INITIALLY DEFERRED;

-- warehouse_locations: Ensure (tenant_id, location_id) is unique for FK reference
-- Note: location_id is already PK, but we add explicit constraint for clarity
ALTER TABLE warehouse_locations
ADD CONSTRAINT warehouse_locations_tenant_location_unique
UNIQUE (tenant_id, location_id) DEFERRABLE INITIALLY DEFERRED;

-- users: Ensure (tenant_id, user_id) is unique for FK reference
-- Note: user_id is already PK, but we add explicit constraint for clarity
ALTER TABLE users
ADD CONSTRAINT users_tenant_user_unique
UNIQUE (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON CONSTRAINT stock_moves_tenant_move_unique ON stock_moves IS 'Enables composite FK from stock_adjustments for tenant isolation';
COMMENT ON CONSTRAINT products_tenant_product_unique ON products IS 'Enables composite FK from stock_adjustments for tenant isolation';
COMMENT ON CONSTRAINT warehouse_locations_tenant_location_unique ON warehouse_locations IS 'Enables composite FK from stock_adjustments for tenant isolation';
COMMENT ON CONSTRAINT users_tenant_user_unique ON users IS 'Enables composite FK from stock_adjustments for tenant isolation';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration is prerequisite for stock_adjustments composite FKs
-- Ensures data integrity and prevents cross-tenant references
-- All constraints are DEFERRABLE INITIALLY DEFERRED to allow bulk operations
