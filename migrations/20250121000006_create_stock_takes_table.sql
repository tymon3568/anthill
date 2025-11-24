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
    status VARCHAR(20) NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Draft', 'Scheduled', 'InProgress', 'Completed', 'Cancelled')),

    -- Dates
    scheduled_date TIMESTAMPTZ,                          -- When the stock take is scheduled
    started_at TIMESTAMPTZ,                              -- When counting actually started
    completed_at TIMESTAMPTZ,                            -- When counting was completed

    -- User assignments
    created_by UUID NOT NULL,                            -- User who created the stock take
    assigned_to UUID,                                    -- User assigned to perform the count

    -- Stock take details
    notes TEXT,                                          -- Additional notes
    reason TEXT,                                         -- Reason for stock take

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT stock_takes_number_unique_per_tenant
        UNIQUE (tenant_id, stock_take_number),
    CONSTRAINT stock_takes_tenant_id_unique
        UNIQUE (tenant_id, stock_take_id),
    CONSTRAINT stock_takes_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT stock_takes_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT stock_takes_tenant_assigned_to_fk
        FOREIGN KEY (tenant_id, assigned_to)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT stock_takes_completion_dates
        CHECK (completed_at IS NULL OR (started_at IS NOT NULL AND completed_at >= started_at))
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



CREATE INDEX idx_stock_takes_tenant_scheduled_date
    ON stock_takes(tenant_id, scheduled_date)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_created_by
    ON stock_takes(tenant_id, created_by)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_takes_tenant_assigned_to
    ON stock_takes(tenant_id, assigned_to)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_stock_takes_tenant_active
    ON stock_takes(tenant_id, stock_take_id)
    WHERE deleted_at IS NULL AND status IN ('Scheduled', 'InProgress');

CREATE INDEX idx_stock_takes_tenant_completed
    ON stock_takes(tenant_id, stock_take_id)
    WHERE deleted_at IS NULL AND status = 'Completed';

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
COMMENT ON COLUMN stock_takes.status IS 'Stock take status: Draft/Scheduled/InProgress/Completed/Cancelled';
COMMENT ON COLUMN stock_takes.scheduled_date IS 'Date when the stock take is scheduled';
COMMENT ON COLUMN stock_takes.started_at IS 'Timestamp when counting started';
COMMENT ON COLUMN stock_takes.completed_at IS 'Timestamp when counting completed';
COMMENT ON COLUMN stock_takes.created_by IS 'User ID who created the stock take';
COMMENT ON COLUMN stock_takes.assigned_to IS 'User ID assigned to perform the count';
COMMENT ON COLUMN stock_takes.notes IS 'Additional notes about the stock take';
COMMENT ON COLUMN stock_takes.reason IS 'Reason for performing the stock take';

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
-- - Basic status tracking for operational workflow

-- Future migrations will add:
-- - stock_take_lines table (individual item counts in stock takes)
-- - Integration with inventory adjustments
-- - Stock take analytics and reporting
