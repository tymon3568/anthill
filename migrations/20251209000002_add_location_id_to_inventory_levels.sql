-- Migration: Add location_id to inventory_levels table
-- Description: Adds location_id to inventory_levels for per-location stock tracking, enabling removal strategies to work with location-specific stock levels
-- Dependencies: storage_locations table (20251205000001), inventory_levels table (20250110000032)
-- Created: 2025-12-09

-- ==================================
-- ADD LOCATION SUPPORT TO INVENTORY LEVELS
-- ==================================

-- Add location_id column
ALTER TABLE inventory_levels
ADD COLUMN location_id UUID REFERENCES storage_locations(location_id);

-- Update unique constraint to include location_id
ALTER TABLE inventory_levels
DROP CONSTRAINT inventory_levels_unique_product_per_warehouse;

ALTER TABLE inventory_levels
ADD CONSTRAINT inventory_levels_unique_product_per_location
    UNIQUE (tenant_id, warehouse_id, location_id, product_id);

-- ==================================
-- UPDATE INDEXES
-- ==================================

-- Drop old index
DROP INDEX idx_inventory_levels_tenant_warehouse_product;

-- Add new composite index
CREATE INDEX idx_inventory_levels_tenant_warehouse_location_product
    ON inventory_levels(tenant_id, warehouse_id, location_id, product_id)
    WHERE deleted_at IS NULL;

-- Update available quantity index to include location
DROP INDEX idx_inventory_levels_tenant_warehouse_available;
CREATE INDEX idx_inventory_levels_tenant_warehouse_location_available
    ON inventory_levels(tenant_id, warehouse_id, location_id, available_quantity)
    WHERE deleted_at IS NULL;

-- ==================================
-- MIGRATION NOTES
-- ==================================

-- This migration enables per-location inventory tracking:
-- - Each product can have different stock levels per warehouse location
-- - Supports removal strategies that pick from specific locations
-- - Enables location-based inventory reports and analytics

-- WARNING: Existing inventory_levels rows need location_id values
-- before this migration can be applied in production.
-- Consider running a data migration script to assign default locations.

COMMENT ON COLUMN inventory_levels.location_id IS 'Reference to storage_locations for per-location inventory tracking';
