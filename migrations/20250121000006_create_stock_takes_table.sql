-- Migration: Create stock_takes table
-- Description: Creates the stock_takes table for managing physical inventory counting sessions
-- Dependencies: warehouses table (20250110000023), tenants table
-- Created: 2025-11-22

-- ==================================
-- SEQUENCE FOR STOCK TAKE NUMBER GENERATION
-- ==================================
-- Global sequence for stock take numbers (STK-YYYY-XXXXX)
-- Note: For production multi-tenant systems, consider per-tenant sequences

CREATE SEQUENCE IF NOT EXISTS stock_take_number_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- ==================================
-- FUNCTION FOR STOCK TAKE NUMBER GENERATION
-- ==================================

CREATE OR REPLACE FUNCTION generate_stock_take_number()
RETURNS TEXT AS $$
DECLARE
    current_year TEXT := EXTRACT(YEAR FROM NOW())::TEXT;
    next_seq TEXT := LPAD(nextval('stock_take_number_seq')::TEXT, 5, '0');
BEGIN
    RETURN 'STK-' || current_year || '-' || next_seq;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- STOCK_TAKES TABLE (Physical Inventory Counting)
-- ==================================
-- This table manages Stock Takes (STK) which record physical inventory counting sessions
-- within a warehouse. Stock takes are used for inventory accuracy verification and reconciliation.

CREATE TABLE stock_takes (
    -- Primary key using UUID v7 (timestamp-based)
    stock_take_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Stock take identifiers
    stock_take_number VARCHAR(50) NOT NULL DEFAULT generate_stock_take_number(),  -- Auto-generated: STK-2025-00001
    reference_number VARCHAR(100),         -- External reference (optional)

    -- Warehouse relationship (single warehouse for stock take)
    warehouse_id UUID NOT NULL,

    -- Stock take status
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'scheduled', 'in_progress', 'completed', 'cancelled')),

    -- Count type and priority
    count_type VARCHAR(20) NOT NULL DEFAULT 'full'
        CHECK (count_type IN ('full', 'partial', 'cycle', 'spot')),
    priority VARCHAR(10) NOT NULL DEFAULT 'normal'
        CHECK (priority IN ('low', 'normal', 'high', 'urgent')),

    -- Dates
    scheduled_date TIMESTAMPTZ,                          -- When the stock take is scheduled
    started_at TIMESTAMPTZ,                              -- When counting actually started
    completed_at TIMESTAMPTZ,                            -- When counting was completed

    -- User assignments
    initiated_by UUID NOT NULL,                          -- User who initiated the stock take
    assigned_to UUID,                                    -- User assigned to perform the count
    approved_by UUID,                                    -- User who approved the results
    approved_at TIMESTAMPTZ,                             -- When results were approved

    -- Stock take details
    notes TEXT,                                          -- Additional notes
    reason TEXT,                                         -- Reason for stock take
    variance_threshold DECIMAL(5,2),                     -- Acceptable variance percentage (e.g., 2.5 for 2.5%)

    -- Summary fields (calculated from stock take lines)
    total_items_counted INTEGER NOT NULL DEFAULT 0,      -- Number of items counted
    total_variance BIGINT NOT NULL DEFAULT 0,            -- Total variance in base units

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT stock_takes_number_unique_per_tenant
        UNIQUE (tenant_id, stock_take_number) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_takes_tenant_id_unique
        UNIQUE (tenant_id, stock_take_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_takes_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT stock_takes_tenant_initiated_by_fk
        FOREIGN KEY (tenant_id, initiated_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT stock_takes_tenant_assigned_to_fk
        FOREIGN KEY (tenant_id, assigned_to)
        REFERENCES users (tenant_id, user_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_takes_tenant_approved_by_fk
        FOREIGN KEY (tenant_id, approved_by)
        REFERENCES users (tenant_id, user_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_takes_positive_items_counted
        CHECK (total_items_counted >= 0),
    CONSTRAINT stock_takes_variance_threshold_range
        CHECK (variance_threshold IS NULL OR (variance_threshold >= 0 AND variance_threshold <= 100)),
    CONSTRAINT stock_takes_completion_dates
        CHECK (completed_at IS NULL OR (started_at IS NOT NULL AND completed_at >= started_at)),
    CONSTRAINT stock_takes_approval_dates
        CHECK (approved_at IS NULL OR approved_by IS NOT NULL)
);



-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_stock_takes_tenant_number
    ON stock_takes(tenant_id, stock_take_number)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_status
    ON stock_takes(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_warehouse
    ON stock_takes(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_type
    ON stock_takes(tenant_id, count_type)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_priority
    ON stock_takes(tenant_id, priority)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_scheduled_date
    ON stock_takes(tenant_id, scheduled_date)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_initiated_by
    ON stock_takes(tenant_id, initiated_by)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_assigned_to
    ON stock_takes(tenant_id, assigned_to)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_stock_takes_tenant_active
    ON stock_takes(tenant_id, stock_take_id)
    WHERE deleted_at IS NULL AND status IN ('scheduled', 'in_progress');

CREATE INDEX idx_stock_takes_tenant_completed
    ON stock_takes(tenant_id, stock_take_id)
    WHERE deleted_at IS NULL AND status = 'completed';

CREATE INDEX idx_stock_takes_tenant_dates
    ON stock_takes(tenant_id, scheduled_date, started_at, completed_at)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_stock_takes_updated_at
    BEFORE UPDATE ON stock_takes
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

COMMENT ON TABLE stock_takes IS 'Stock Takes (STK) table for recording physical inventory counting sessions';
COMMENT ON COLUMN stock_takes.stock_take_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN stock_takes.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN stock_takes.stock_take_number IS 'Auto-generated stock take number (STK-YYYY-XXXXX)';
COMMENT ON COLUMN stock_takes.reference_number IS 'External reference number (optional)';
COMMENT ON COLUMN stock_takes.warehouse_id IS 'Warehouse where the stock take is performed';
COMMENT ON COLUMN stock_takes.status IS 'Stock take status: draft/scheduled/in_progress/completed/cancelled';
COMMENT ON COLUMN stock_takes.count_type IS 'Type of count: full/partial/cycle/spot';
COMMENT ON COLUMN stock_takes.priority IS 'Stock take priority: low/normal/high/urgent';
COMMENT ON COLUMN stock_takes.scheduled_date IS 'Date when the stock take is scheduled';
COMMENT ON COLUMN stock_takes.started_at IS 'Timestamp when counting started';
COMMENT ON COLUMN stock_takes.completed_at IS 'Timestamp when counting completed';
COMMENT ON COLUMN stock_takes.initiated_by IS 'User ID who initiated the stock take';
COMMENT ON COLUMN stock_takes.assigned_to IS 'User ID assigned to perform the count';
COMMENT ON COLUMN stock_takes.approved_by IS 'User ID who approved the results';
COMMENT ON COLUMN stock_takes.approved_at IS 'Timestamp when results were approved';
COMMENT ON COLUMN stock_takes.notes IS 'Additional notes about the stock take';
COMMENT ON COLUMN stock_takes.reason IS 'Reason for performing the stock take';
COMMENT ON COLUMN stock_takes.variance_threshold IS 'Acceptable variance percentage (0-100)';
COMMENT ON COLUMN stock_takes.total_items_counted IS 'Number of items that were counted';
COMMENT ON COLUMN stock_takes.total_variance IS 'Total variance in base units';

COMMENT ON FUNCTION generate_stock_take_number() IS 'Generates auto-incrementing stock take numbers in format STK-YYYY-XXXXX';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Physical inventory counting session management
-- 2. Stock take scheduling and assignment
-- 3. Inventory accuracy verification
-- 4. Variance analysis and reconciliation
-- 5. Audit trail for inventory counts

-- Key design decisions:
-- - Single warehouse per stock take (unlike transfers which have source/destination)
-- - User assignment for counting responsibility
-- - Approval workflow for count results
-- - Variance tracking for accuracy analysis
-- - Different count types for operational flexibility

-- Future migrations will add:
-- - stock_take_items table (individual item counts in stock takes)
-- - Stock take approval workflows
-- - Automated variance calculations
-- - Integration with inventory adjustments
-- - Stock take analytics and reporting
