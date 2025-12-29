-- ============================================================================
-- Migration: Add Cycle Count Columns to stock_takes Table
-- Task ID: V1_MVP/04_Inventory_Service/4.14_Cycle_Counting_Scrap/task_04.14.01_implement_cycle_counting.md
-- Description: Extends stock_takes table to support cycle counting workflow
-- ============================================================================

-- ----------------------------------------------------------------------------
-- Add new columns to stock_takes for cycle counting support
-- ----------------------------------------------------------------------------

-- Count type to distinguish full counts from cycle counts
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS count_type TEXT NOT NULL DEFAULT 'full'
    CHECK (count_type IN ('full', 'cycle', 'spot'));

COMMENT ON COLUMN stock_takes.count_type IS 'Type of count: full (complete inventory), cycle (scheduled partial), spot (ad-hoc verification)';

-- Schedule ID for schedule-triggered cycle counts
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS schedule_id UUID NULL;

COMMENT ON COLUMN stock_takes.schedule_id IS 'Reference to cycle_count_schedules if count was triggered by a schedule';

-- Location scope (if specified, only this location subtree is counted)
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS location_id UUID NULL;

COMMENT ON COLUMN stock_takes.location_id IS 'Location root for scoped counts; NULL means entire warehouse';

-- As-of timestamp for snapshot semantics
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS as_of TIMESTAMPTZ NULL;

COMMENT ON COLUMN stock_takes.as_of IS 'Snapshot reference timestamp for expected quantities';

-- Generated adjustment ID (after reconciliation)
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS adjustment_id UUID NULL;

COMMENT ON COLUMN stock_takes.adjustment_id IS 'Reference to stock adjustment generated during reconciliation';

-- User who closed/reconciled the session
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS closed_by UUID NULL;

COMMENT ON COLUMN stock_takes.closed_by IS 'User who closed/reconciled the stock take session';

-- Updated by user (for tracking who made changes)
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS updated_by UUID NULL;

COMMENT ON COLUMN stock_takes.updated_by IS 'User who last updated the stock take';

-- Deleted by user (for soft delete audit)
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS deleted_by UUID NULL;

COMMENT ON COLUMN stock_takes.deleted_by IS 'User who soft-deleted the stock take';

-- Total items counted and total variance (for summary statistics)
ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS total_items_counted INTEGER NOT NULL DEFAULT 0;

COMMENT ON COLUMN stock_takes.total_items_counted IS 'Count of lines that have been counted';

ALTER TABLE stock_takes
ADD COLUMN IF NOT EXISTS total_variance BIGINT NOT NULL DEFAULT 0;

COMMENT ON COLUMN stock_takes.total_variance IS 'Sum of all variance quantities (positive and negative)';

-- ----------------------------------------------------------------------------
-- Add foreign key for location_id if storage_locations table exists
-- ----------------------------------------------------------------------------
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'storage_locations') THEN
        ALTER TABLE stock_takes
        ADD CONSTRAINT fk_stock_takes_location
            FOREIGN KEY (tenant_id, location_id)
            REFERENCES storage_locations(tenant_id, location_id)
            ON DELETE RESTRICT
            DEFERRABLE INITIALLY DEFERRED;
    END IF;
EXCEPTION
    WHEN duplicate_object THEN
        -- Constraint already exists, ignore
        NULL;
END $$;

-- ----------------------------------------------------------------------------
-- Add new indexes for cycle counting queries
-- ----------------------------------------------------------------------------

-- Index for filtering by count_type
CREATE INDEX IF NOT EXISTS idx_stock_takes_tenant_count_type
    ON stock_takes(tenant_id, count_type)
    WHERE deleted_at IS NULL;

-- Index for schedule-based lookups
CREATE INDEX IF NOT EXISTS idx_stock_takes_tenant_schedule
    ON stock_takes(tenant_id, schedule_id)
    WHERE schedule_id IS NOT NULL AND deleted_at IS NULL;

-- Index for location-scoped counts
CREATE INDEX IF NOT EXISTS idx_stock_takes_tenant_location
    ON stock_takes(tenant_id, location_id)
    WHERE location_id IS NOT NULL AND deleted_at IS NULL;

-- Index for as_of timestamp queries
CREATE INDEX IF NOT EXISTS idx_stock_takes_tenant_as_of
    ON stock_takes(tenant_id, as_of)
    WHERE as_of IS NOT NULL AND deleted_at IS NULL;

-- ----------------------------------------------------------------------------
-- Add columns to stock_take_lines for cycle counting
-- ----------------------------------------------------------------------------

-- Line status to track counting progress
ALTER TABLE stock_take_lines
ADD COLUMN IF NOT EXISTS line_status TEXT NOT NULL DEFAULT 'open'
    CHECK (line_status IN ('open', 'counted', 'skipped'));

COMMENT ON COLUMN stock_take_lines.line_status IS 'Line status: open (not counted), counted (count submitted), skipped (intentionally skipped)';

-- Optional variant ID for product variants
ALTER TABLE stock_take_lines
ADD COLUMN IF NOT EXISTS variant_id UUID NULL;

COMMENT ON COLUMN stock_take_lines.variant_id IS 'Product variant ID if variant-level tracking is enabled';

-- Lot/Serial tracking
ALTER TABLE stock_take_lines
ADD COLUMN IF NOT EXISTS lot_id UUID NULL;

COMMENT ON COLUMN stock_take_lines.lot_id IS 'Lot ID for lot-tracked products';

ALTER TABLE stock_take_lines
ADD COLUMN IF NOT EXISTS serial_id UUID NULL;

COMMENT ON COLUMN stock_take_lines.serial_id IS 'Serial ID for serial-tracked products';

-- Location ID for location-level counting
ALTER TABLE stock_take_lines
ADD COLUMN IF NOT EXISTS location_id UUID NULL;

COMMENT ON COLUMN stock_take_lines.location_id IS 'Specific storage location being counted';

-- Updated at timestamp
ALTER TABLE stock_take_lines
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

COMMENT ON COLUMN stock_take_lines.updated_at IS 'Last update timestamp';

-- ----------------------------------------------------------------------------
-- Add trigger to update updated_at on stock_take_lines
-- ----------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION update_stock_take_lines_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_stock_take_lines_updated_at ON stock_take_lines;
CREATE TRIGGER trg_stock_take_lines_updated_at
    BEFORE UPDATE ON stock_take_lines
    FOR EACH ROW
    EXECUTE FUNCTION update_stock_take_lines_updated_at();

-- ----------------------------------------------------------------------------
-- Add indexes for stock_take_lines cycle counting queries
-- ----------------------------------------------------------------------------

-- Index for line status filtering
CREATE INDEX IF NOT EXISTS idx_stock_take_lines_tenant_status
    ON stock_take_lines(tenant_id, stock_take_id, line_status)
    WHERE deleted_at IS NULL;

-- Index for location-level queries
CREATE INDEX IF NOT EXISTS idx_stock_take_lines_tenant_location
    ON stock_take_lines(tenant_id, location_id)
    WHERE location_id IS NOT NULL AND deleted_at IS NULL;

-- Index for lot tracking
CREATE INDEX IF NOT EXISTS idx_stock_take_lines_tenant_lot
    ON stock_take_lines(tenant_id, lot_id)
    WHERE lot_id IS NOT NULL AND deleted_at IS NULL;

-- ----------------------------------------------------------------------------
-- Migration Metadata
-- ----------------------------------------------------------------------------

-- This migration extends the existing stock_takes infrastructure to support:
-- 1. Cycle counting (partial, scheduled inventory counts)
-- 2. Spot checks (ad-hoc verification counts)
-- 3. Location-scoped counting
-- 4. Snapshot semantics with as_of timestamp
-- 5. Line-level status tracking (open/counted/skipped)
-- 6. Lot and serial number tracking at line level

-- Key design decisions:
-- - Reuse existing stock_takes table instead of separate cycle_counts table
-- - Add count_type to distinguish between full/cycle/spot counts
-- - as_of timestamp enables snapshot-based expected quantity calculation
-- - line_status enables partial counting workflow
-- - Maintain backward compatibility with existing stock take functionality
