-- Migration: Create stock_take_lines table
-- Description: Creates the stock_take_lines table for recording individual product counts in stock takes
-- Dependencies: stock_takes table (20250121000006), products table, users table
-- Created: 2025-11-23

-- ==================================
-- STOCK_TAKE_LINES TABLE (Stock Take Details)
-- ==================================
-- This table records the individual product counts during a stock take session.
-- Each line represents one product's expected vs actual quantity in a specific stock take.

CREATE TABLE stock_take_lines (
    -- Primary key using UUID v7 (timestamp-based)
    line_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE RESTRICT ON UPDATE RESTRICT,

    -- Stock take relationship
    stock_take_id UUID NOT NULL,

    -- Product relationship
    product_id UUID NOT NULL,

    -- Quantity counts
    expected_quantity INTEGER NOT NULL DEFAULT 0,  -- Expected quantity from system
    actual_quantity INTEGER,                        -- Actual counted quantity (NULL if not counted yet)
    difference_quantity INTEGER GENERATED ALWAYS AS (actual_quantity - expected_quantity) STORED,  -- Auto-calculated: actual - expected

    -- Counting details
    counted_by UUID,                                -- User who performed the count
    counted_at TIMESTAMPTZ,                         -- When this line was counted
    notes TEXT,                                     -- Notes about the count (discrepancies, issues, etc.)

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT stock_take_lines_tenant_stock_take_fk
        FOREIGN KEY (tenant_id, stock_take_id)
        REFERENCES stock_takes (tenant_id, stock_take_id)
        ON DELETE RESTRICT ON UPDATE RESTRICT,
    CONSTRAINT stock_take_lines_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id)
        ON DELETE RESTRICT ON UPDATE RESTRICT,
    CONSTRAINT stock_take_lines_tenant_counted_by_fk
        FOREIGN KEY (tenant_id, counted_by)
        REFERENCES users (tenant_id, user_id)
        ON DELETE SET NULL ON UPDATE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_take_lines_positive_expected
        CHECK (expected_quantity >= 0),
    CONSTRAINT stock_take_lines_positive_actual
        CHECK (actual_quantity IS NULL OR actual_quantity >= 0),

    CONSTRAINT stock_take_lines_counted_at_required
        CHECK (
            (counted_by IS NULL AND counted_at IS NULL) OR
            (counted_by IS NOT NULL AND counted_at IS NOT NULL)
        )

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Unique constraint (partial for soft delete)
CREATE UNIQUE INDEX idx_stock_take_lines_unique_per_stock_take_product
    ON stock_take_lines(tenant_id, stock_take_id, product_id)
    WHERE deleted_at IS NULL;

-- Primary lookup indexes
CREATE INDEX idx_stock_take_lines_tenant_stock_take
    ON stock_take_lines(tenant_id, stock_take_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_take_lines_tenant_product
    ON stock_take_lines(tenant_id, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_take_lines_tenant_counted_by
    ON stock_take_lines(tenant_id, counted_by)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_stock_take_lines_tenant_counted_at
    ON stock_take_lines(tenant_id, counted_at)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_take_lines_tenant_expected_actual
    ON stock_take_lines(tenant_id, expected_quantity, actual_quantity)
    WHERE deleted_at IS NULL;

-- Composite indexes for stock take processing
CREATE INDEX idx_stock_take_lines_tenant_stock_take_counted
    ON stock_take_lines(tenant_id, stock_take_id, counted_at)
    WHERE deleted_at IS NULL AND actual_quantity IS NOT NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_stock_take_lines_updated_at
    BEFORE UPDATE ON stock_take_lines
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- ROW LEVEL SECURITY (Future)
-- ==================================
-- Note: We use application-level filtering instead of RLS
-- All queries must include: WHERE tenant_id = $1 AND deleted_at IS NULL

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE stock_take_lines IS 'Stock take lines - Individual product counts within stock take sessions';
COMMENT ON COLUMN stock_take_lines.line_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN stock_take_lines.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN stock_take_lines.stock_take_id IS 'Reference to parent stock take';
COMMENT ON COLUMN stock_take_lines.product_id IS 'Product being counted';
COMMENT ON COLUMN stock_take_lines.expected_quantity IS 'Expected quantity from inventory system';
COMMENT ON COLUMN stock_take_lines.actual_quantity IS 'Actual counted quantity (NULL if not counted)';
COMMENT ON COLUMN stock_take_lines.difference_quantity IS 'Auto-calculated difference: actual - expected';
COMMENT ON COLUMN stock_take_lines.counted_by IS 'User ID who performed the count';
COMMENT ON COLUMN stock_take_lines.counted_at IS 'Timestamp when the count was performed';
COMMENT ON COLUMN stock_take_lines.notes IS 'Additional notes about the count';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the detailed counting records for:
-- 1. Individual product counts in stock takes
-- 2. Variance analysis per product
-- 3. User accountability for counts
-- 4. Audit trail for stock take details
-- 5. Progress tracking during counting

-- Key design decisions:
-- - One line per product per stock take (partial unique index for soft delete)
-- - Difference auto-calculated via generated column
-- - Optional counting (actual_quantity can be NULL)
-- - User tracking for accountability
-- - Comprehensive indexing for reporting

-- Future enhancements will include:
-- - Triggers to update stock_takes summary fields
-- - Stock take completion validation
-- - Automated adjustment creation from variances
-- - Batch counting operations
-- - Mobile counting app integration
-- - Barcode scanning support
-- - Photo evidence attachment (future table)
