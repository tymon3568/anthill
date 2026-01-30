-- Migration: Create adjustment_documents and adjustment_lines tables
-- Description: Creates tables for stock adjustment documents with header/lines pattern
-- Similar to scrap_documents/scrap_lines for consistency
-- Dependencies: tenants, warehouses, products, users tables
-- Created: 2026-01-29

-- ==================================
-- ADJUSTMENT_DOCUMENTS TABLE (Header)
-- ==================================
-- This table stores the header information for stock adjustments.
-- Each adjustment document can have multiple line items.

CREATE TABLE adjustment_documents (
    -- Composite primary key for multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    adjustment_id UUID NOT NULL DEFAULT uuid_generate_v7(),

    -- Human-readable reference (e.g., ADJ-2026-0001)
    reference TEXT,

    -- Workflow status
    status TEXT NOT NULL DEFAULT 'draft',

    -- Warehouse where adjustment applies
    warehouse_id UUID NOT NULL,

    -- Optional notes
    notes TEXT,

    -- Audit fields
    created_by UUID,
    posted_by UUID,
    posted_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Primary key
    PRIMARY KEY (tenant_id, adjustment_id),

    -- Status constraint
    CONSTRAINT chk_adjustment_documents_status
        CHECK (status IN ('draft', 'posted', 'cancelled')),

    -- Posted fields constraint - if posted, must have posted_at and posted_by
    CONSTRAINT chk_adjustment_documents_posted_fields
        CHECK (status != 'posted' OR (posted_at IS NOT NULL AND posted_by IS NOT NULL)),

    -- Cancelled fields constraint - if cancelled, must have cancelled_at and cancelled_by
    CONSTRAINT chk_adjustment_documents_cancelled_fields
        CHECK (status != 'cancelled' OR (cancelled_at IS NOT NULL AND cancelled_by IS NOT NULL)),

    -- Foreign keys
    CONSTRAINT fk_adjustment_documents_warehouse
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses(tenant_id, warehouse_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- ADJUSTMENT_LINES TABLE (Line Items)
-- ==================================
-- This table stores line items for each adjustment document.

CREATE TABLE adjustment_lines (
    -- Composite primary key for multi-tenancy
    tenant_id UUID NOT NULL,
    adjustment_line_id UUID NOT NULL DEFAULT uuid_generate_v7(),

    -- Reference to parent document
    adjustment_id UUID NOT NULL,

    -- Product being adjusted
    product_id UUID NOT NULL,
    variant_id UUID,

    -- Adjustment details
    adjustment_type TEXT NOT NULL DEFAULT 'decrease',  -- 'increase' or 'decrease'
    qty BIGINT NOT NULL,  -- Always positive, direction determined by adjustment_type

    -- Reason for adjustment
    reason_code TEXT NOT NULL,
    reason_notes TEXT,

    -- Location within warehouse (optional, for zone/bin tracking)
    location_id UUID,

    -- Lot/Serial tracking (optional)
    lot_id UUID,
    serial_id UUID,

    -- Stock move created when posted
    posted_stock_move_id UUID,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Primary key
    PRIMARY KEY (tenant_id, adjustment_line_id),

    -- Constraints
    CONSTRAINT chk_adjustment_lines_qty_positive
        CHECK (qty > 0),

    CONSTRAINT chk_adjustment_lines_type
        CHECK (adjustment_type IN ('increase', 'decrease')),

    CONSTRAINT chk_adjustment_lines_reason_code
        CHECK (reason_code IN (
            'damaged',           -- Product damaged
            'lost',              -- Product lost/missing
            'found',             -- Product found (for increases)
            'count_correction',  -- Physical count correction
            'system_correction', -- System error correction
            'expired',           -- Product expired
            'theft',             -- Suspected theft
            'promotion',         -- Used for promotion/sample
            'return_to_stock',   -- Returned to stock
            'other'              -- Other reason (use notes)
        )),

    -- Foreign keys
    CONSTRAINT fk_adjustment_lines_document
        FOREIGN KEY (tenant_id, adjustment_id)
        REFERENCES adjustment_documents(tenant_id, adjustment_id)
        ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,

    CONSTRAINT fk_adjustment_lines_product
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products(tenant_id, product_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,

    CONSTRAINT fk_adjustment_lines_location
        FOREIGN KEY (tenant_id, location_id)
        REFERENCES warehouse_locations(tenant_id, location_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- adjustment_documents indexes
CREATE INDEX idx_adjustment_documents_tenant_status
    ON adjustment_documents(tenant_id, status);

CREATE INDEX idx_adjustment_documents_tenant_warehouse
    ON adjustment_documents(tenant_id, warehouse_id);

CREATE INDEX idx_adjustment_documents_tenant_created_at
    ON adjustment_documents(tenant_id, created_at DESC);

CREATE INDEX idx_adjustment_documents_tenant_posted_at
    ON adjustment_documents(tenant_id, posted_at DESC)
    WHERE posted_at IS NOT NULL;

CREATE INDEX idx_adjustment_documents_tenant_reference
    ON adjustment_documents(tenant_id, reference)
    WHERE reference IS NOT NULL;

-- adjustment_lines indexes
CREATE INDEX idx_adjustment_lines_tenant_adjustment
    ON adjustment_lines(tenant_id, adjustment_id);

CREATE INDEX idx_adjustment_lines_tenant_product
    ON adjustment_lines(tenant_id, product_id);

CREATE INDEX idx_adjustment_lines_tenant_location
    ON adjustment_lines(tenant_id, location_id)
    WHERE location_id IS NOT NULL;

CREATE INDEX idx_adjustment_lines_tenant_lot
    ON adjustment_lines(tenant_id, lot_id)
    WHERE lot_id IS NOT NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_adjustment_documents_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_adjustment_documents_updated_at
    BEFORE UPDATE ON adjustment_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_adjustment_documents_updated_at();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE adjustment_documents IS 'Stock adjustment documents - header table for inventory adjustments';
COMMENT ON COLUMN adjustment_documents.adjustment_id IS 'UUID v7 primary key';
COMMENT ON COLUMN adjustment_documents.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN adjustment_documents.reference IS 'Human-readable reference number (e.g., ADJ-2026-0001)';
COMMENT ON COLUMN adjustment_documents.status IS 'Workflow status: draft, posted, cancelled';
COMMENT ON COLUMN adjustment_documents.warehouse_id IS 'Warehouse where adjustment applies';
COMMENT ON COLUMN adjustment_documents.notes IS 'Optional notes for the adjustment';

COMMENT ON TABLE adjustment_lines IS 'Stock adjustment line items - details of products being adjusted';
COMMENT ON COLUMN adjustment_lines.adjustment_line_id IS 'UUID v7 primary key';
COMMENT ON COLUMN adjustment_lines.adjustment_type IS 'Type of adjustment: increase or decrease';
COMMENT ON COLUMN adjustment_lines.qty IS 'Quantity to adjust (always positive)';
COMMENT ON COLUMN adjustment_lines.reason_code IS 'Reason code for the adjustment';
COMMENT ON COLUMN adjustment_lines.posted_stock_move_id IS 'Reference to stock_move created when posted';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration supports:
-- 1. Header/Lines pattern for stock adjustments
-- 2. Workflow states: draft -> posted or draft -> cancelled
-- 3. Multi-tenant isolation with composite keys
-- 4. Audit trail with created_by, posted_by, cancelled_by
-- 5. Integration with stock_moves for inventory tracking
-- 6. Support for lot/serial tracking
-- 7. Multiple reason codes for different adjustment scenarios
