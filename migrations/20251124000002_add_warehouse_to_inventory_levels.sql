-- Migration: Add warehouse_id to inventory_levels table
-- Description: Adds warehouse support to inventory levels for multi-warehouse stock tracking
-- Dependencies: warehouses table, inventory_levels table
-- Created: 2025-11-24

-- ==================================
-- ADD WAREHOUSE SUPPORT TO INVENTORY LEVELS
-- ==================================

-- Add warehouse_id column
ALTER TABLE inventory_levels
ADD COLUMN warehouse_id UUID NOT NULL REFERENCES warehouses(warehouse_id);

-- Update unique constraint to include warehouse
ALTER TABLE inventory_levels
DROP CONSTRAINT inventory_levels_unique_product_per_tenant;

ALTER TABLE inventory_levels
ADD CONSTRAINT inventory_levels_unique_product_per_warehouse
    UNIQUE (tenant_id, warehouse_id, product_id);

-- ==================================
-- UPDATE INDEXES
-- ==================================

-- Drop old index
DROP INDEX idx_inventory_levels_tenant_product;

-- Add new composite index
CREATE INDEX idx_inventory_levels_tenant_warehouse_product
    ON inventory_levels(tenant_id, warehouse_id, product_id)
    WHERE deleted_at IS NULL;

-- Update available quantity index to include warehouse
DROP INDEX idx_inventory_levels_tenant_available;
CREATE INDEX idx_inventory_levels_tenant_warehouse_available
    ON inventory_levels(tenant_id, warehouse_id, available_quantity)
    WHERE deleted_at IS NULL;

-- ==================================
-- MIGRATION NOTES
-- ==================================

-- This migration enables per-warehouse inventory tracking:
-- - Each product can have different stock levels per warehouse
-- - Transfers between warehouses will update inventory accordingly
-- - Existing data will need warehouse_id populated (manual step required)

-- WARNING: Existing inventory_levels rows need warehouse_id values
-- before this migration can be applied in production.
-- Consider running a data migration script to assign default warehouse.

COMMENT ON COLUMN inventory_levels.warehouse_id IS 'Reference to warehouse for multi-warehouse inventory tracking';
