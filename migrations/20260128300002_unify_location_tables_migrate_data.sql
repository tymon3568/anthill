-- Migration: Unify location tables - Migrate data and update FKs
-- Description: Migrate data from storage_locations to warehouse_locations, update FKs
-- Dependencies: 20260128300001_unify_location_tables_add_columns.sql
-- Part of: Module 4.5 - Location Architecture Fix
-- Created: 2026-01-28

-- ==================================
-- CREATE MIGRATION MAPPING TABLE
-- ==================================
-- This table maps old storage_locations.location_id to new warehouse_locations.location_id
-- Used for updating FKs in other tables

CREATE TABLE IF NOT EXISTS _migration_location_mapping (
    old_location_id UUID PRIMARY KEY,
    new_location_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    warehouse_id UUID NOT NULL,
    location_code VARCHAR(100) NOT NULL,
    migrated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ==================================
-- MIGRATE DATA FROM storage_locations TO warehouse_locations
-- ==================================

-- Insert storage_locations into warehouse_locations, matching zones by code
-- Generate new UUID for each migrated location and store mapping
WITH migrated AS (
    INSERT INTO warehouse_locations (
        location_id,
        tenant_id,
        warehouse_id,
        zone_id,
        location_code,
        location_name,
        description,
        location_type,
        coordinates,
        dimensions,
        capacity_info,
        location_attributes,
        is_active,
        created_at,
        updated_at,
        deleted_at,
        -- New columns from storage_locations
        aisle,
        rack,
        level,
        position,
        capacity,
        current_stock,
        is_quarantine,
        is_picking_location,
        length_cm,
        width_cm,
        height_cm,
        weight_limit_kg,
        created_by,
        updated_by
    )
    SELECT
        uuid_generate_v7() AS location_id,  -- Generate new location_id
        sl.tenant_id,
        sl.warehouse_id,
        wz.zone_id,  -- Match zone by name/code
        sl.location_code,
        COALESCE(sl.location_code, 'Migrated Location') AS location_name,
        NULL AS description,
        sl.location_type,
        -- Build coordinates JSONB from hierarchical structure
        jsonb_build_object(
            'aisle', sl.aisle,
            'rack', sl.rack,
            'level', sl.level,
            'position', sl.position
        ) AS coordinates,
        -- Build dimensions JSONB
        CASE
            WHEN sl.length_cm IS NOT NULL AND sl.width_cm IS NOT NULL AND sl.height_cm IS NOT NULL
            THEN jsonb_build_object(
                'length_mm', sl.length_cm * 10,
                'width_mm', sl.width_cm * 10,
                'height_mm', sl.height_cm * 10
            )
            ELSE NULL
        END AS dimensions,
        -- Build capacity_info JSONB
        CASE
            WHEN sl.capacity IS NOT NULL OR sl.weight_limit_kg IS NOT NULL
            THEN jsonb_build_object(
                'max_weight_kg', sl.weight_limit_kg,
                'max_units', sl.capacity
            )
            ELSE NULL
        END AS capacity_info,
        -- Build location_attributes JSONB
        jsonb_build_object(
            'is_quarantine', sl.is_quarantine,
            'is_picking', sl.is_picking_location
        ) AS location_attributes,
        sl.is_active,
        sl.created_at,
        sl.updated_at,
        sl.deleted_at,
        -- Direct column mappings
        sl.aisle,
        sl.rack,
        sl.level,
        sl.position,
        sl.capacity,
        sl.current_stock,
        sl.is_quarantine,
        sl.is_picking_location,
        sl.length_cm,
        sl.width_cm,
        sl.height_cm,
        sl.weight_limit_kg,
        sl.created_by,
        sl.updated_by
    FROM storage_locations sl
    LEFT JOIN warehouse_zones wz ON wz.tenant_id = sl.tenant_id
        AND wz.warehouse_id = sl.warehouse_id
        AND wz.zone_code = sl.zone
    -- Only migrate locations that don't already exist in warehouse_locations
    WHERE NOT EXISTS (
        SELECT 1 FROM warehouse_locations wl
        WHERE wl.tenant_id = sl.tenant_id
        AND wl.warehouse_id = sl.warehouse_id
        AND wl.location_code = sl.location_code
    )
    RETURNING location_id, tenant_id, warehouse_id, location_code
)
INSERT INTO _migration_location_mapping (old_location_id, new_location_id, tenant_id, warehouse_id, location_code)
SELECT
    sl.location_id AS old_location_id,
    m.location_id AS new_location_id,
    m.tenant_id,
    m.warehouse_id,
    m.location_code
FROM storage_locations sl
JOIN migrated m ON m.tenant_id = sl.tenant_id
    AND m.warehouse_id = sl.warehouse_id
    AND m.location_code = sl.location_code;

-- ==================================
-- UPDATE inventory_levels.location_id FK
-- ==================================

-- First, drop the existing FK constraint to storage_locations
ALTER TABLE inventory_levels
DROP CONSTRAINT IF EXISTS inventory_levels_location_id_fkey;

-- Update location_id values using the mapping table
UPDATE inventory_levels il
SET location_id = mlm.new_location_id
FROM _migration_location_mapping mlm
WHERE il.location_id = mlm.old_location_id
  AND il.location_id IS NOT NULL;

-- Add new FK constraint to warehouse_locations
-- Use composite FK for multi-tenant safety
ALTER TABLE inventory_levels
ADD CONSTRAINT inventory_levels_location_id_fkey
FOREIGN KEY (tenant_id, location_id)
REFERENCES warehouse_locations(tenant_id, location_id);

-- ==================================
-- UPDATE lots_serial_numbers.location_id FK
-- ==================================

-- lots_serial_numbers also has location_id that may reference storage_locations
-- Check if there's an existing constraint
ALTER TABLE lots_serial_numbers
DROP CONSTRAINT IF EXISTS lots_serial_numbers_location_id_fkey;

-- Update location_id values using the mapping table
UPDATE lots_serial_numbers lsn
SET location_id = mlm.new_location_id
FROM _migration_location_mapping mlm
WHERE lsn.location_id = mlm.old_location_id
  AND lsn.location_id IS NOT NULL;

-- The lots_serial_numbers.location_id FK already references warehouse_locations
-- based on the ERD, so we just need to ensure consistency
-- If it was referencing storage_locations, update to warehouse_locations
ALTER TABLE lots_serial_numbers
ADD CONSTRAINT lots_serial_numbers_tenant_location_fk
FOREIGN KEY (tenant_id, location_id)
REFERENCES warehouse_locations(tenant_id, location_id);

-- ==================================
-- VERIFICATION QUERIES
-- ==================================
-- Run these to verify migration success before proceeding

-- Check for orphan references in inventory_levels
DO $$
DECLARE
    orphan_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO orphan_count
    FROM inventory_levels il
    WHERE il.location_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM warehouse_locations wl
        WHERE wl.location_id = il.location_id
    );

    IF orphan_count > 0 THEN
        RAISE WARNING 'Found % orphan location_id references in inventory_levels', orphan_count;
    ELSE
        RAISE NOTICE 'No orphan location_id references found in inventory_levels';
    END IF;
END $$;

-- Check for orphan references in lots_serial_numbers
DO $$
DECLARE
    orphan_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO orphan_count
    FROM lots_serial_numbers lsn
    WHERE lsn.location_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM warehouse_locations wl
        WHERE wl.location_id = lsn.location_id
    );

    IF orphan_count > 0 THEN
        RAISE WARNING 'Found % orphan location_id references in lots_serial_numbers', orphan_count;
    ELSE
        RAISE NOTICE 'No orphan location_id references found in lots_serial_numbers';
    END IF;
END $$;

-- ==================================
-- LOG MIGRATION STATS
-- ==================================

DO $$
DECLARE
    migrated_count INTEGER;
    total_storage_locations INTEGER;
    total_warehouse_locations INTEGER;
BEGIN
    SELECT COUNT(*) INTO migrated_count FROM _migration_location_mapping;
    SELECT COUNT(*) INTO total_storage_locations FROM storage_locations WHERE deleted_at IS NULL;
    SELECT COUNT(*) INTO total_warehouse_locations FROM warehouse_locations WHERE deleted_at IS NULL;

    RAISE NOTICE 'Migration complete: % locations migrated', migrated_count;
    RAISE NOTICE 'Total storage_locations: %', total_storage_locations;
    RAISE NOTICE 'Total warehouse_locations after migration: %', total_warehouse_locations;
END $$;

-- ==================================
-- MIGRATION METADATA
-- ==================================
-- This migration:
-- 1. Creates a mapping table for old->new location_id
-- 2. Migrates data from storage_locations to warehouse_locations
-- 3. Updates inventory_levels.location_id FK to reference warehouse_locations
-- 4. Updates lots_serial_numbers.location_id FK if needed
--
-- Next migration will:
-- - Drop storage_locations table (after Rust code is updated)
-- - Drop _migration_location_mapping table
