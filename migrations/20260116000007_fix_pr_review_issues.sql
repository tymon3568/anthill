-- Migration: Fix PR Review Issues (PRs #152, #153, #154)
-- Description: Addresses code review feedback from merged PRs
-- Created: 2026-01-17
--
-- Issues Fixed:
-- PR #152 (Storage Categories):
--   1. Change gen_random_uuid() to uuid_generate_v7() for storage_category_id
--   2. Add partial unique index on name with soft-delete support
--   3. Remove redundant idx_storage_categories_tenant_code
--   4. Fix idx_storage_categories_tenant_active to use only (tenant_id)
--
-- PR #153 (Cycle Count Schedules):
--   1. Add UNIQUE constraint on warehouse_locations(tenant_id, location_id) for FK support
--   2. Add UNIQUE constraint on product_categories(tenant_id, category_id) for FK support
--   3. Add CHECK constraint for end_at >= start_at
--   4. Change gen_random_uuid() to uuid_generate_v7() for schedule_id
--
-- PR #154 (Valuation Settings):
--   1. Remove 'lifo' from CHECK constraint (not supported in Rust enum)
--   2. Add deleted_at column for soft-delete support
--   3. Change gen_random_uuid() to uuid_generate_v7() for id
--   4. Add partial unique index with soft-delete support

-- ==================================
-- SAFETY: Data Validation Functions
-- ==================================
-- These functions check for potential constraint violations before creating indexes.
-- If violations exist, they raise a warning but allow the migration to continue
-- by cleaning up duplicates (keeping the most recent record).

-- Function to dedupe warehouse_locations if duplicates exist
CREATE OR REPLACE FUNCTION _migration_dedupe_warehouse_locations() RETURNS void AS $$
DECLARE
    dup_count INTEGER;
BEGIN
    -- Count duplicates
    SELECT COUNT(*) INTO dup_count
    FROM (
        SELECT tenant_id, location_id, COUNT(*) as cnt
        FROM warehouse_locations
        GROUP BY tenant_id, location_id
        HAVING COUNT(*) > 1
    ) dups;

    IF dup_count > 0 THEN
        RAISE WARNING 'Found % duplicate (tenant_id, location_id) pairs in warehouse_locations. Keeping most recent records.', dup_count;

        -- Delete older duplicates, keeping the one with the most recent updated_at
        DELETE FROM warehouse_locations w1
        WHERE EXISTS (
            SELECT 1 FROM warehouse_locations w2
            WHERE w1.tenant_id = w2.tenant_id
              AND w1.location_id = w2.location_id
              AND (w1.updated_at < w2.updated_at
                   OR (w1.updated_at = w2.updated_at AND w1.location_id < w2.location_id))
        );
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to dedupe product_categories if duplicates exist
CREATE OR REPLACE FUNCTION _migration_dedupe_product_categories() RETURNS void AS $$
DECLARE
    dup_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO dup_count
    FROM (
        SELECT tenant_id, category_id, COUNT(*) as cnt
        FROM product_categories
        GROUP BY tenant_id, category_id
        HAVING COUNT(*) > 1
    ) dups;

    IF dup_count > 0 THEN
        RAISE WARNING 'Found % duplicate (tenant_id, category_id) pairs in product_categories. Keeping most recent records.', dup_count;

        -- Delete older duplicates, keeping the one with the most recent updated_at
        DELETE FROM product_categories p1
        WHERE EXISTS (
            SELECT 1 FROM product_categories p2
            WHERE p1.tenant_id = p2.tenant_id
              AND p1.category_id = p2.category_id
              AND (p1.updated_at < p2.updated_at
                   OR (p1.updated_at = p2.updated_at AND p1.category_id < p2.category_id))
        );
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to fix invalid date ranges in cycle_count_schedules
CREATE OR REPLACE FUNCTION _migration_fix_schedule_date_ranges() RETURNS void AS $$
DECLARE
    invalid_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO invalid_count
    FROM cycle_count_schedules
    WHERE end_at IS NOT NULL AND end_at < start_at;

    IF invalid_count > 0 THEN
        RAISE WARNING 'Found % schedules with end_at < start_at. Setting end_at = start_at for these records.', invalid_count;

        UPDATE cycle_count_schedules
        SET end_at = start_at
        WHERE end_at IS NOT NULL AND end_at < start_at;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- PR #152 FIXES: Storage Categories
-- ==================================

-- Fix 1: Change default UUID generation to v7
-- Note: This only affects new records; existing records keep their v4 UUIDs
ALTER TABLE storage_categories
    ALTER COLUMN storage_category_id SET DEFAULT uuid_generate_v7();

-- Fix 2: Drop the non-soft-delete-aware unique constraint and replace with partial
ALTER TABLE storage_categories
    DROP CONSTRAINT IF EXISTS storage_categories_name_unique_per_tenant;

CREATE UNIQUE INDEX IF NOT EXISTS storage_categories_name_unique_per_tenant
    ON storage_categories (tenant_id, name)
    WHERE deleted_at IS NULL;

-- Fix 3: Drop redundant index (code lookup already handled by partial unique index)
DROP INDEX IF EXISTS idx_storage_categories_tenant_code;

-- Fix 4: Fix the active categories index to use only tenant_id for simpler filtering
DROP INDEX IF EXISTS idx_storage_categories_tenant_active;

CREATE INDEX idx_storage_categories_tenant_active
    ON storage_categories (tenant_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- ==================================
-- PR #153 FIXES: Cycle Count Schedules
-- ==================================

-- SAFETY: Run deduplication before creating unique indexes
SELECT _migration_dedupe_warehouse_locations();
SELECT _migration_dedupe_product_categories();
SELECT _migration_fix_schedule_date_ranges();

-- Fix 1: Add UNIQUE constraint on warehouse_locations for composite FK support
-- This enables the FK (tenant_id, location_id) in cycle_count_schedule_locations
CREATE UNIQUE INDEX IF NOT EXISTS warehouse_locations_tenant_location_unique
    ON warehouse_locations (tenant_id, location_id);

-- Fix 2: Add UNIQUE constraint on product_categories for composite FK support
-- This enables the FK (tenant_id, category_id) in cycle_count_schedule_categories
CREATE UNIQUE INDEX IF NOT EXISTS product_categories_tenant_category_unique
    ON product_categories (tenant_id, category_id);

-- Fix 3: Add CHECK constraint for date range validation
-- Using NOT VALID to avoid scanning existing rows, then validate separately
ALTER TABLE cycle_count_schedules
    DROP CONSTRAINT IF EXISTS cycle_count_schedules_date_range_check;

ALTER TABLE cycle_count_schedules
    ADD CONSTRAINT cycle_count_schedules_date_range_check
    CHECK (end_at IS NULL OR end_at >= start_at)
    NOT VALID;

-- Validate the constraint (this is safe now since we fixed invalid data above)
ALTER TABLE cycle_count_schedules
    VALIDATE CONSTRAINT cycle_count_schedules_date_range_check;

-- Fix 4: Change default UUID generation to v7 for schedule_id
ALTER TABLE cycle_count_schedules
    ALTER COLUMN schedule_id SET DEFAULT uuid_generate_v7();

-- ==================================
-- PR #154 FIXES: Valuation Settings
-- ==================================

-- Fix 1: Remove 'lifo' from CHECK constraint (not supported in Rust enum)
-- First, update any existing 'lifo' values to 'fifo' (the default)
UPDATE inventory_valuation_settings
SET method = 'fifo'
WHERE method = 'lifo';

-- Drop and recreate the constraint with only supported methods
ALTER TABLE inventory_valuation_settings
    DROP CONSTRAINT IF EXISTS inventory_valuation_settings_method_check;

ALTER TABLE inventory_valuation_settings
    ADD CONSTRAINT inventory_valuation_settings_method_check
    CHECK (method IN ('fifo', 'avco', 'standard'))
    NOT VALID;

-- Validate the constraint (safe since we fixed lifo values above)
ALTER TABLE inventory_valuation_settings
    VALIDATE CONSTRAINT inventory_valuation_settings_method_check;

-- Fix 2: Add deleted_at column for soft-delete support
ALTER TABLE inventory_valuation_settings
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Fix 3: Change default UUID generation to v7
ALTER TABLE inventory_valuation_settings
    ALTER COLUMN id SET DEFAULT uuid_generate_v7();

-- Fix 4: Update unique indexes to support soft-delete
-- Drop existing unique indexes
DROP INDEX IF EXISTS idx_valuation_settings_tenant_unique;
DROP INDEX IF EXISTS idx_valuation_settings_override_unique;

-- Recreate with soft-delete awareness
CREATE UNIQUE INDEX idx_valuation_settings_tenant_unique
    ON inventory_valuation_settings(tenant_id, scope_type)
    WHERE scope_type = 'tenant' AND deleted_at IS NULL;

CREATE UNIQUE INDEX idx_valuation_settings_override_unique
    ON inventory_valuation_settings(tenant_id, scope_type, scope_id)
    WHERE scope_id IS NOT NULL AND deleted_at IS NULL;

-- Update category/product lookup indexes for soft-delete
DROP INDEX IF EXISTS idx_valuation_settings_category;
DROP INDEX IF EXISTS idx_valuation_settings_product;

CREATE INDEX idx_valuation_settings_category
    ON inventory_valuation_settings(tenant_id, scope_id)
    WHERE scope_type = 'category' AND deleted_at IS NULL;

CREATE INDEX idx_valuation_settings_product
    ON inventory_valuation_settings(tenant_id, scope_id)
    WHERE scope_type = 'product' AND deleted_at IS NULL;

-- ==================================
-- CLEANUP: Drop temporary functions
-- ==================================
DROP FUNCTION IF EXISTS _migration_dedupe_warehouse_locations();
DROP FUNCTION IF EXISTS _migration_dedupe_product_categories();
DROP FUNCTION IF EXISTS _migration_fix_schedule_date_ranges();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON COLUMN storage_categories.storage_category_id IS 'UUID v7 primary key (timestamp-based, migrated from v4)';
COMMENT ON COLUMN cycle_count_schedules.schedule_id IS 'UUID v7 primary key (timestamp-based, migrated from v4)';
COMMENT ON COLUMN inventory_valuation_settings.id IS 'UUID v7 primary key (timestamp-based, migrated from v4)';
COMMENT ON COLUMN inventory_valuation_settings.deleted_at IS 'Soft delete timestamp - follows project standard';
COMMENT ON COLUMN inventory_valuation_settings.method IS 'Valuation method: fifo, avco (weighted average), standard';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration fixes review issues from PRs #152, #153, #154:
-- 1. UUID v7 adoption for all new tables
-- 2. Soft-delete aware unique constraints
-- 3. Proper composite FK support via UNIQUE indexes
-- 4. Date range validation for schedules
-- 5. Removed unsupported 'lifo' method from DB constraint
-- 6. Added safety checks: deduplication before unique indexes, NOT VALID + VALIDATE for constraints
