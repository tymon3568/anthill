-- Migration: Unify location tables - Drop storage_locations
-- Description: Drop the legacy storage_locations table after migration is complete
-- Dependencies: 20260128300002_unify_location_tables_migrate_data.sql
-- Part of: Module 4.5 - Location Architecture Fix
-- Created: 2026-01-28

-- ==================================
-- VERIFY NO ORPHAN REFERENCES
-- ==================================

DO $$
DECLARE
    orphan_count INTEGER;
BEGIN
    -- Check inventory_levels
    SELECT COUNT(*) INTO orphan_count
    FROM inventory_levels il
    WHERE il.location_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM warehouse_locations wl
        WHERE wl.location_id = il.location_id
    );

    IF orphan_count > 0 THEN
        RAISE EXCEPTION 'Found % orphan location_id references in inventory_levels', orphan_count;
    END IF;

    -- Check lots_serial_numbers
    SELECT COUNT(*) INTO orphan_count
    FROM lots_serial_numbers lsn
    WHERE lsn.location_id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM warehouse_locations wl
        WHERE wl.location_id = lsn.location_id
    );

    IF orphan_count > 0 THEN
        RAISE EXCEPTION 'Found % orphan location_id references in lots_serial_numbers', orphan_count;
    END IF;

    RAISE NOTICE 'No orphan references found - safe to drop storage_locations';
END $$;

-- ==================================
-- DROP STORAGE_LOCATIONS TABLE
-- ==================================

-- Drop the legacy storage_locations table
-- All data should have been migrated to warehouse_locations
DROP TABLE IF EXISTS storage_locations CASCADE;

-- Drop the migration mapping table if it exists
DROP TABLE IF EXISTS _migration_location_mapping;

-- ==================================
-- UPDATE COMMENTS
-- ==================================

COMMENT ON TABLE warehouse_locations IS 'Unified storage locations within warehouse zones. Replaces the legacy storage_locations table. Supports hierarchical structure (zone -> aisle -> rack -> level -> position) and capacity tracking.';

-- ==================================
-- MIGRATION METADATA
-- ==================================
-- This migration completes the location table unification:
-- 1. Verified no orphan FK references exist
-- 2. Dropped legacy storage_locations table
-- 3. warehouse_locations is now the single source of truth for all storage locations
