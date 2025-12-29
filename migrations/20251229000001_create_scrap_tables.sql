-- ============================================================================
-- Migration: Create Scrap Management Tables
-- Task ID: V1_MVP/04_Inventory_Service/4.14_Cycle_Counting_Scrap/task_04.14.02_implement_scrap_management.md
-- Description: Creates tables for scrap document workflow (draft → posted)
-- ============================================================================

-- ----------------------------------------------------------------------------
-- Table: scrap_documents
-- Purpose: Scrap document header - represents a scrap request/execution
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scrap_documents (
    -- Primary identification
    tenant_id UUID NOT NULL,
    scrap_id UUID NOT NULL,

    -- Document reference
    reference TEXT NULL,

    -- Status: 'draft', 'posted', 'cancelled'
    status TEXT NOT NULL DEFAULT 'draft',

    -- Scrap location (where scrapped items go)
    scrap_location_id UUID NOT NULL,

    -- Notes
    notes TEXT NULL,

    -- Audit fields
    created_by UUID NULL,
    posted_by UUID NULL,
    posted_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    PRIMARY KEY (tenant_id, scrap_id),

    -- Foreign keys
    CONSTRAINT fk_scrap_documents_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants(tenant_id)
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED,

    CONSTRAINT fk_scrap_documents_scrap_location
        FOREIGN KEY (tenant_id, scrap_location_id)
        REFERENCES warehouses(tenant_id, warehouse_id)
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,

    -- Status validation
    CONSTRAINT chk_scrap_documents_status
        CHECK (status IN ('draft', 'posted', 'cancelled')),

    -- Posted fields consistency
    CONSTRAINT chk_scrap_documents_posted_fields
        CHECK (
            (status != 'posted') OR
            (posted_at IS NOT NULL AND posted_by IS NOT NULL)
        )
);

-- Indexes for common access patterns
CREATE INDEX IF NOT EXISTS idx_scrap_documents_tenant_status
    ON scrap_documents(tenant_id, status);

CREATE INDEX IF NOT EXISTS idx_scrap_documents_tenant_posted_at
    ON scrap_documents(tenant_id, posted_at)
    WHERE posted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_scrap_documents_tenant_created_at
    ON scrap_documents(tenant_id, created_at);

CREATE INDEX IF NOT EXISTS idx_scrap_documents_tenant_reference
    ON scrap_documents(tenant_id, reference)
    WHERE reference IS NOT NULL;

-- ----------------------------------------------------------------------------
-- Table: scrap_lines
-- Purpose: Individual items being scrapped within a document
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scrap_lines (
    -- Primary identification
    tenant_id UUID NOT NULL,
    scrap_line_id UUID NOT NULL,

    -- Parent document
    scrap_id UUID NOT NULL,

    -- Product being scrapped
    product_id UUID NOT NULL,
    variant_id UUID NULL,

    -- Source location (where items are taken from)
    source_location_id UUID NOT NULL,

    -- Lot/Serial tracking (optional)
    lot_id UUID NULL,
    serial_id UUID NULL,

    -- Quantity (must be > 0)
    qty BIGINT NOT NULL,

    -- Reason for scrapping
    reason_code TEXT NULL, -- 'damaged', 'expired', 'lost', 'quality_fail', 'obsolete', 'other'
    reason TEXT NULL,

    -- Link to stock move created on posting
    posted_stock_move_id UUID NULL,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    PRIMARY KEY (tenant_id, scrap_line_id),

    -- Foreign keys
    CONSTRAINT fk_scrap_lines_document
        FOREIGN KEY (tenant_id, scrap_id)
        REFERENCES scrap_documents(tenant_id, scrap_id)
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED,

    CONSTRAINT fk_scrap_lines_product
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products(tenant_id, product_id)
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,

    CONSTRAINT fk_scrap_lines_source_location
        FOREIGN KEY (tenant_id, source_location_id)
        REFERENCES warehouses(tenant_id, warehouse_id)
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,

    CONSTRAINT fk_scrap_lines_lot
        FOREIGN KEY (tenant_id, lot_id)
        REFERENCES lots_serial_numbers(tenant_id, lot_serial_id)
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,

    -- Quantity validation
    CONSTRAINT chk_scrap_lines_qty_positive
        CHECK (qty > 0),

    -- Reason code validation
    CONSTRAINT chk_scrap_lines_reason_code
        CHECK (
            reason_code IS NULL OR
            reason_code IN ('damaged', 'expired', 'lost', 'quality_fail', 'obsolete', 'other')
        )
);

-- Indexes for common access patterns
CREATE INDEX IF NOT EXISTS idx_scrap_lines_tenant_scrap
    ON scrap_lines(tenant_id, scrap_id);

CREATE INDEX IF NOT EXISTS idx_scrap_lines_tenant_product
    ON scrap_lines(tenant_id, product_id);

CREATE INDEX IF NOT EXISTS idx_scrap_lines_tenant_source_location
    ON scrap_lines(tenant_id, source_location_id);

CREATE INDEX IF NOT EXISTS idx_scrap_lines_tenant_lot
    ON scrap_lines(tenant_id, lot_id)
    WHERE lot_id IS NOT NULL;

-- ----------------------------------------------------------------------------
-- Trigger: Auto-update updated_at on scrap_documents
-- ----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION update_scrap_documents_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_scrap_documents_updated_at ON scrap_documents;
CREATE TRIGGER trg_scrap_documents_updated_at
    BEFORE UPDATE ON scrap_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_scrap_documents_updated_at();

-- ----------------------------------------------------------------------------
-- Comments
-- ----------------------------------------------------------------------------
COMMENT ON TABLE scrap_documents IS 'Scrap document header - represents a scrap request/execution workflow';
COMMENT ON COLUMN scrap_documents.tenant_id IS 'Tenant isolation key';
COMMENT ON COLUMN scrap_documents.scrap_id IS 'Unique identifier (UUID v7)';
COMMENT ON COLUMN scrap_documents.reference IS 'Optional external reference number';
COMMENT ON COLUMN scrap_documents.status IS 'Document status: draft, posted, cancelled';
COMMENT ON COLUMN scrap_documents.scrap_location_id IS 'Destination location for scrapped items';
COMMENT ON COLUMN scrap_documents.posted_at IS 'Timestamp when document was posted';
COMMENT ON COLUMN scrap_documents.posted_by IS 'User who posted the document';

COMMENT ON TABLE scrap_lines IS 'Individual line items within a scrap document';
COMMENT ON COLUMN scrap_lines.tenant_id IS 'Tenant isolation key';
COMMENT ON COLUMN scrap_lines.scrap_line_id IS 'Unique line identifier (UUID v7)';
COMMENT ON COLUMN scrap_lines.scrap_id IS 'Parent scrap document';
COMMENT ON COLUMN scrap_lines.product_id IS 'Product being scrapped';
COMMENT ON COLUMN scrap_lines.source_location_id IS 'Location from which items are taken';
COMMENT ON COLUMN scrap_lines.qty IS 'Quantity to scrap (BIGINT for precision)';
COMMENT ON COLUMN scrap_lines.reason_code IS 'Standard reason code for scrap';
COMMENT ON COLUMN scrap_lines.posted_stock_move_id IS 'Stock move created when document is posted';
