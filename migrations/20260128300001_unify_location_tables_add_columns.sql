-- Migration: Unify location tables - Add columns to warehouse_locations
-- Description: Add missing columns from storage_locations to warehouse_locations
-- Dependencies: warehouse_locations (20250110000023), storage_locations (20251205000001)
-- Part of: Module 4.5 - Location Architecture Fix
-- Created: 2026-01-28

-- ==================================
-- PROBLEM STATEMENT
-- ==================================
-- Currently there are TWO separate location tables:
-- 1. storage_locations: Has zone (VARCHAR), aisle, rack, level, position, capacity, current_stock
-- 2. warehouse_locations: Has zone_id (FK to warehouse_zones), coordinates (JSONB), dimensions (JSONB)
--
-- This causes:
-- - inventory_levels.location_id references storage_locations but is ALWAYS NULL
-- - stock_moves references warehouse_locations
-- - Inconsistent location tracking across the system
--
-- SOLUTION: Merge storage_locations columns into warehouse_locations, then migrate data

-- ==================================
-- ADD MISSING COLUMNS TO warehouse_locations
-- ==================================

-- Add hierarchical structure columns (from storage_locations)
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS aisle VARCHAR(50);
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS rack VARCHAR(50);
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS level INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS position INTEGER;

-- Add capacity and stock tracking columns
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS capacity BIGINT;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS current_stock BIGINT DEFAULT 0;

-- Add location flags
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS is_quarantine BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS is_picking_location BOOLEAN NOT NULL DEFAULT true;

-- Add dimension columns (separate from JSONB for easier querying)
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS length_cm INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS width_cm INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS height_cm INTEGER;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS weight_limit_kg INTEGER;

-- Add audit columns
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS created_by UUID;
ALTER TABLE warehouse_locations ADD COLUMN IF NOT EXISTS updated_by UUID;

-- ==================================
-- ADD CONSTRAINTS
-- ==================================

-- Add capacity check constraint
ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_capacity_check
    CHECK (capacity IS NULL OR capacity > 0);

-- Add current_stock check constraint
ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_current_stock_check
    CHECK (current_stock IS NULL OR current_stock >= 0);

-- Add stock within capacity constraint
ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_stock_within_capacity_check
    CHECK (capacity IS NULL OR current_stock IS NULL OR current_stock <= capacity);

-- Add dimensions check constraint
ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_dimensions_check
    CHECK (
        (length_cm IS NULL AND width_cm IS NULL AND height_cm IS NULL) OR
        (length_cm > 0 AND width_cm > 0 AND height_cm > 0)
    );

-- ==================================
-- ADD TENANT-USER FK CONSTRAINTS FOR AUDIT COLUMNS
-- ==================================

-- Add FK for created_by (deferrable to handle circular references)
ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_tenant_created_by_fk
    FOREIGN KEY (tenant_id, created_by)
    REFERENCES users (tenant_id, user_id)
    DEFERRABLE INITIALLY DEFERRED;

-- Add FK for updated_by (deferrable to handle circular references)
ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_tenant_updated_by_fk
    FOREIGN KEY (tenant_id, updated_by)
    REFERENCES users (tenant_id, user_id)
    DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- ADD UNIQUE CONSTRAINT FOR TENANT-LOCATION COMPOSITE
-- ==================================

-- Add composite unique constraint for FK referenceability
-- This allows other tables to reference (tenant_id, location_id)
-- Use DO block to handle case where constraint already exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'warehouse_locations_tenant_location_unique'
    ) THEN
        ALTER TABLE warehouse_locations ADD CONSTRAINT warehouse_locations_tenant_location_unique
            UNIQUE (tenant_id, location_id);
    END IF;
END $$;

-- ==================================
-- ADD ADDITIONAL INDEXES FOR NEW COLUMNS
-- ==================================

-- Hierarchical navigation index
CREATE INDEX IF NOT EXISTS idx_warehouse_locations_hierarchy
    ON warehouse_locations(tenant_id, warehouse_id, aisle, rack, level, position)
    WHERE deleted_at IS NULL;

-- Capacity and stock management index
CREATE INDEX IF NOT EXISTS idx_warehouse_locations_capacity
    ON warehouse_locations(tenant_id, warehouse_id, capacity, current_stock)
    WHERE deleted_at IS NULL AND is_active = true;

-- Picking optimization index
CREATE INDEX IF NOT EXISTS idx_warehouse_locations_picking
    ON warehouse_locations(tenant_id, warehouse_id, location_type, current_stock)
    WHERE deleted_at IS NULL AND is_active = true AND is_picking_location = true;

-- Quarantine locations index
CREATE INDEX IF NOT EXISTS idx_warehouse_locations_quarantine
    ON warehouse_locations(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL AND is_active = true AND is_quarantine = true;

-- ==================================
-- UPDATE TABLE COMMENT
-- ==================================

COMMENT ON TABLE warehouse_locations IS 'Unified storage locations within warehouse zones (merged from storage_locations + warehouse_locations)';
COMMENT ON COLUMN warehouse_locations.aisle IS 'Aisle identifier within zone (e.g., "01", "02")';
COMMENT ON COLUMN warehouse_locations.rack IS 'Rack identifier within aisle (e.g., "R01", "R02")';
COMMENT ON COLUMN warehouse_locations.level IS 'Level/shelf number on rack (1, 2, 3...)';
COMMENT ON COLUMN warehouse_locations.position IS 'Position number on level (1, 2, 3...)';
COMMENT ON COLUMN warehouse_locations.capacity IS 'Maximum capacity in base units';
COMMENT ON COLUMN warehouse_locations.current_stock IS 'Current stock quantity at this location';
COMMENT ON COLUMN warehouse_locations.is_quarantine IS 'Whether this location is for quarantined stock';
COMMENT ON COLUMN warehouse_locations.is_picking_location IS 'Whether this location is used for picking operations';
COMMENT ON COLUMN warehouse_locations.length_cm IS 'Location length in centimeters';
COMMENT ON COLUMN warehouse_locations.width_cm IS 'Location width in centimeters';
COMMENT ON COLUMN warehouse_locations.height_cm IS 'Location height in centimeters';
COMMENT ON COLUMN warehouse_locations.weight_limit_kg IS 'Maximum weight limit in kilograms';
COMMENT ON COLUMN warehouse_locations.created_by IS 'User who created this location';
COMMENT ON COLUMN warehouse_locations.updated_by IS 'User who last updated this location';

-- ==================================
-- MIGRATION METADATA
-- ==================================
-- This migration adds columns from storage_locations to warehouse_locations
-- Next migration will:
-- 1. Migrate data from storage_locations to warehouse_locations
-- 2. Update inventory_levels FK to reference warehouse_locations
-- 3. Drop storage_locations table
