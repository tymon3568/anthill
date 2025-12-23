-- Migration: Fix soft delete unique constraints
-- Description: Replaces unconditional unique constraints with partial unique indexes to support soft delete reuse
-- Dependencies: inventory_levels, products, warehouses, warehouse_zones, warehouse_locations
-- Created: 2025-12-16

-- ==================================
-- INVENTORY LEVELS
-- ==================================
-- Drop unconditional constraint created in 20251124000002_add_warehouse_to_inventory_levels.sql
-- Note: Updated to drop 'inventory_levels_unique_product_per_location' which replaced the per-warehouse constraint
ALTER TABLE inventory_levels
DROP CONSTRAINT IF EXISTS inventory_levels_unique_product_per_location;

-- Drop non-unique index if it exists (replaced by location-aware index in 20251209000002)
DROP INDEX IF EXISTS idx_inventory_levels_tenant_warehouse_location_product;

-- Create UNIQUE index filtering out deleted records (includes location_id for per-location tracking)
CREATE UNIQUE INDEX idx_inventory_levels_tenant_warehouse_location_product_unique
ON inventory_levels(tenant_id, warehouse_id, location_id, product_id)
WHERE deleted_at IS NULL;


-- ==================================
-- PRODUCTS
-- ==================================
-- Drop unconditional constraint
ALTER TABLE products
DROP CONSTRAINT IF EXISTS products_sku_unique_per_tenant;

-- Drop existing index if it exists (it might conflict or be redundant)
DROP INDEX IF EXISTS idx_products_tenant_sku;

-- Create UNIQUE index filtering out deleted records
CREATE UNIQUE INDEX idx_products_sku_unique_active
ON products(tenant_id, sku)
WHERE deleted_at IS NULL;


-- ==================================
-- WAREHOUSES
-- ==================================
-- Drop unconditional constraint
ALTER TABLE warehouses
DROP CONSTRAINT IF EXISTS warehouses_code_unique_per_tenant;

-- Drop existing index if it exists (it might conflict or be redundant)
DROP INDEX IF EXISTS idx_warehouses_tenant_code;

-- Create UNIQUE index filtering out deleted records
CREATE UNIQUE INDEX idx_warehouses_code_unique_active
ON warehouses(tenant_id, warehouse_code)
WHERE deleted_at IS NULL;


-- ==================================
-- WAREHOUSE ZONES
-- ==================================
-- Drop unconditional constraint
ALTER TABLE warehouse_zones
DROP CONSTRAINT IF EXISTS warehouse_zones_code_unique_per_warehouse;

-- Drop existing index if it exists (it might conflict or be redundant)
DROP INDEX IF EXISTS idx_warehouse_zones_warehouse_code;

-- Create UNIQUE index filtering out deleted records
CREATE UNIQUE INDEX idx_warehouse_zones_code_unique_active
ON warehouse_zones(tenant_id, warehouse_id, zone_code)
WHERE deleted_at IS NULL;


-- ==================================
-- WAREHOUSE LOCATIONS
-- ==================================
-- Drop unconditional constraint
ALTER TABLE warehouse_locations
DROP CONSTRAINT IF EXISTS warehouse_locations_code_unique_per_warehouse;

-- Drop existing index if it exists (it might conflict or be redundant)
DROP INDEX IF EXISTS idx_warehouse_locations_warehouse_code;

-- Create UNIQUE index filtering out deleted records
CREATE UNIQUE INDEX idx_warehouse_locations_code_unique_active
ON warehouse_locations(tenant_id, warehouse_id, location_code)
WHERE deleted_at IS NULL;


-- ==================================
-- MIGRATION METADATA
-- ==================================
-- This migration fixes the issue where soft-deleted records prevented reuse of
-- unique identifiers (SKU, Warehouse Code, Location Code, etc.).
--
-- By replacing unconditional UNIQUE constraints with partial UNIQUE indexes
-- (WHERE deleted_at IS NULL), we enforce uniqueness only among active records.
