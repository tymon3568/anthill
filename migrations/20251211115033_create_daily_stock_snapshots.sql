-- Migration: Create daily_stock_snapshots table
-- Description: Creates a summary table for daily stock snapshots to speed up historical reports
-- Dependencies: stock_moves table
-- Created: 2025-12-11

-- ==================================
-- DAILY_STOCK_SNAPSHOTS TABLE
-- ==================================
-- This table stores daily aggregated stock data for performance optimization.
-- It summarizes opening quantity, closing quantity, and total movements per product per day.

CREATE TABLE daily_stock_snapshots (
    -- Primary key
    snapshot_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Product reference
    product_id UUID NOT NULL REFERENCES products(product_id),

    -- Snapshot date (date of the snapshot, e.g., '2025-12-11')
    snapshot_date DATE NOT NULL,

    -- Quantities (in base UOM)
    opening_quantity BIGINT NOT NULL DEFAULT 0,  -- Quantity at start of day
    closing_quantity BIGINT NOT NULL DEFAULT 0,  -- Quantity at end of day
    total_movements BIGINT NOT NULL DEFAULT 0,   -- Net movement during the day

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT daily_stock_snapshots_unique_per_tenant_product_date
        UNIQUE (tenant_id, product_id, snapshot_date),
    CONSTRAINT daily_stock_snapshots_positive_quantities
        CHECK (opening_quantity >= 0 AND closing_quantity >= 0),
    CONSTRAINT daily_stock_snapshots_closing_equals_opening_plus_movements
        CHECK (closing_quantity = opening_quantity + total_movements)
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Primary lookup index
CREATE INDEX idx_daily_stock_snapshots_tenant_product_date
    ON daily_stock_snapshots(tenant_id, product_id, snapshot_date DESC);

-- Query optimization indexes
CREATE INDEX idx_daily_stock_snapshots_tenant_date
    ON daily_stock_snapshots(tenant_id, snapshot_date DESC);



-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_daily_stock_snapshots_updated_at
    BEFORE UPDATE ON daily_stock_snapshots
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- FUNCTION to Populate Snapshots
-- ==================================
-- This function can be called daily to populate snapshots for a given date range

CREATE OR REPLACE FUNCTION populate_daily_stock_snapshots(
    p_tenant_id UUID,
    p_start_date DATE,
    p_end_date DATE DEFAULT CURRENT_DATE
)
RETURNS INTEGER AS $$
DECLARE
    v_date DATE;
    v_count INTEGER := 0;
BEGIN
    -- Loop through each date in the range
    FOR v_date IN SELECT generate_series(p_start_date, p_end_date, '1 day'::interval)::date LOOP
        -- Insert or update snapshots for each product that had movements on this date
        INSERT INTO daily_stock_snapshots (
            tenant_id,
            product_id,
            snapshot_date,
            opening_quantity,
            total_movements,
            closing_quantity
        )
        SELECT
            prev.tenant_id,
            prev.product_id,
            v_date,
            prev.closing_quantity,
            COALESCE(SUM(sm.quantity), 0),
            prev.closing_quantity + COALESCE(SUM(sm.quantity), 0)
        FROM daily_stock_snapshots prev
        LEFT JOIN stock_moves sm ON
            sm.tenant_id = prev.tenant_id
            AND sm.product_id = prev.product_id
            AND sm.move_date >= v_date::timestamptz
            AND sm.move_date < (v_date + INTERVAL '1 day')::timestamptz
        WHERE prev.tenant_id = p_tenant_id
            AND prev.snapshot_date = (v_date - 1)::date
        GROUP BY prev.tenant_id, prev.product_id, prev.closing_quantity
        ON CONFLICT (tenant_id, product_id, snapshot_date)
        DO UPDATE SET
            opening_quantity = EXCLUDED.opening_quantity,
            total_movements = EXCLUDED.total_movements,
            closing_quantity = EXCLUDED.closing_quantity,
            updated_at = NOW();

        v_count := v_count + 1;
    END LOOP;

    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE daily_stock_snapshots IS 'Daily aggregated stock snapshots for performance optimization';
COMMENT ON COLUMN daily_stock_snapshots.snapshot_id IS 'UUID v7 primary key';
COMMENT ON COLUMN daily_stock_snapshots.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN daily_stock_snapshots.product_id IS 'Reference to products table';
COMMENT ON COLUMN daily_stock_snapshots.snapshot_date IS 'Date of the stock snapshot';
COMMENT ON COLUMN daily_stock_snapshots.opening_quantity IS 'Stock quantity at the start of the day';
COMMENT ON COLUMN daily_stock_snapshots.closing_quantity IS 'Stock quantity at the end of the day';
COMMENT ON COLUMN daily_stock_snapshots.total_movements IS 'Net stock movements during the day';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates:
-- 1. Daily stock snapshots table for historical reporting performance
-- 2. Indexes for efficient queries
-- 3. Function to populate snapshots from stock_moves data

-- Usage:
-- SELECT populate_daily_stock_snapshots('tenant-uuid', '2025-01-01', '2025-12-11');
-- Or schedule daily: SELECT populate_daily_stock_snapshots('tenant-uuid', CURRENT_DATE);
