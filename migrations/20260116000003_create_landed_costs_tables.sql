-- Migration: Create landed costs tables
-- Description: Creates tables for landed cost allocation to incoming inventory
-- Dependencies: goods_receipts table, goods_receipt_items table, inventory_valuations table
-- Created: 2026-01-16

-- ==================================
-- PREREQUISITE: Add unique constraint to goods_receipt_items
-- ==================================
-- This is required for the composite foreign key in landed_cost_allocations
ALTER TABLE goods_receipt_items
    ADD CONSTRAINT goods_receipt_items_tenant_item_unique
    UNIQUE (tenant_id, receipt_item_id);

-- ==================================
-- LANDED_COST_DOCUMENTS TABLE
-- ==================================
-- Header table for landed cost documents.
-- A landed cost document groups additional costs (freight, customs, handling, etc.)
-- that need to be allocated to receipt lines.

CREATE TABLE landed_cost_documents (
    -- Primary key using UUID v7
    document_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Document identifiers
    document_number VARCHAR(50) NOT NULL,  -- Auto-generated: LC-2026-00001
    reference_number VARCHAR(100),          -- External reference (vendor invoice, etc.)

    -- Status workflow: draft -> posted -> cancelled
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'posted', 'cancelled')),

    -- Associated receipt (the GRN this landed cost applies to)
    receipt_id UUID NOT NULL,

    -- Totals (calculated from cost lines)
    total_cost_amount BIGINT NOT NULL DEFAULT 0,  -- Total additional costs in cents
    currency_code VARCHAR(3) NOT NULL DEFAULT 'VND',

    -- Allocation method for distributing costs
    allocation_method VARCHAR(20) NOT NULL DEFAULT 'by_value'
        CHECK (allocation_method IN ('by_value', 'by_quantity', 'equal')),

    -- Dates
    document_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMPTZ,  -- When the document was posted

    -- Notes
    notes TEXT,

    -- Audit fields
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT landed_cost_documents_number_unique_per_tenant
        UNIQUE (tenant_id, document_number) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT landed_cost_documents_tenant_receipt_fk
        FOREIGN KEY (tenant_id, receipt_id)
        REFERENCES goods_receipts (tenant_id, receipt_id),
    CONSTRAINT landed_cost_documents_positive_total
        CHECK (total_cost_amount >= 0)
);

-- Add composite unique for foreign key references
ALTER TABLE landed_cost_documents
    ADD CONSTRAINT landed_cost_documents_tenant_document_unique
    UNIQUE (tenant_id, document_id);

-- ==================================
-- LANDED_COST_LINES TABLE
-- ==================================
-- Individual cost lines within a landed cost document.
-- Each line represents a specific type of cost (freight, customs, etc.)

CREATE TABLE landed_cost_lines (
    -- Primary key using UUID v7
    line_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Parent document
    document_id UUID NOT NULL,

    -- Cost type/category
    cost_type VARCHAR(50) NOT NULL,  -- 'freight', 'customs', 'handling', 'insurance', 'other'
    description VARCHAR(255),

    -- Cost amount in cents
    amount BIGINT NOT NULL CHECK (amount >= 0),

    -- Optional: vendor/supplier for this cost
    vendor_reference VARCHAR(100),

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT landed_cost_lines_tenant_document_fk
        FOREIGN KEY (tenant_id, document_id)
        REFERENCES landed_cost_documents (tenant_id, document_id)
);

-- ==================================
-- LANDED_COST_ALLOCATIONS TABLE
-- ==================================
-- Tracks how landed costs are allocated to individual receipt items.
-- Created when a landed cost document is posted.

CREATE TABLE landed_cost_allocations (
    -- Primary key using UUID v7
    allocation_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Parent document
    document_id UUID NOT NULL,

    -- The receipt item receiving the allocation
    receipt_item_id UUID NOT NULL,

    -- Allocation details
    allocated_amount BIGINT NOT NULL CHECK (allocated_amount >= 0),  -- Amount allocated in cents

    -- Pre-allocation values (for audit/rollback)
    original_unit_cost BIGINT NOT NULL,  -- Unit cost before allocation
    new_unit_cost BIGINT NOT NULL,       -- Unit cost after allocation

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT landed_cost_allocations_tenant_document_fk
        FOREIGN KEY (tenant_id, document_id)
        REFERENCES landed_cost_documents (tenant_id, document_id),
    CONSTRAINT landed_cost_allocations_receipt_item_fk
        FOREIGN KEY (tenant_id, receipt_item_id)
        REFERENCES goods_receipt_items (tenant_id, receipt_item_id),

    -- Prevent duplicate allocations for the same receipt item in a document
    CONSTRAINT landed_cost_allocations_unique_per_item
        UNIQUE (tenant_id, document_id, receipt_item_id)
);

-- ==================================
-- SEQUENCES
-- ==================================
-- Sequence for document number generation

CREATE SEQUENCE IF NOT EXISTS landed_cost_document_number_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- ==================================
-- INDEXES
-- ==================================

-- Landed cost documents indexes
CREATE INDEX idx_landed_cost_documents_tenant_status
    ON landed_cost_documents(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_landed_cost_documents_tenant_receipt
    ON landed_cost_documents(tenant_id, receipt_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_landed_cost_documents_tenant_date
    ON landed_cost_documents(tenant_id, document_date DESC)
    WHERE deleted_at IS NULL;

-- Landed cost lines indexes
CREATE INDEX idx_landed_cost_lines_tenant_document
    ON landed_cost_lines(tenant_id, document_id);

CREATE INDEX idx_landed_cost_lines_cost_type
    ON landed_cost_lines(tenant_id, cost_type);

-- Landed cost allocations indexes
CREATE INDEX idx_landed_cost_allocations_tenant_document
    ON landed_cost_allocations(tenant_id, document_id);

CREATE INDEX idx_landed_cost_allocations_receipt_item
    ON landed_cost_allocations(receipt_item_id);

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_landed_cost_documents_updated_at
    BEFORE UPDATE ON landed_cost_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_landed_cost_lines_updated_at
    BEFORE UPDATE ON landed_cost_lines
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS
-- ==================================

COMMENT ON TABLE landed_cost_documents IS 'Landed cost documents for allocating additional costs to incoming inventory';
COMMENT ON TABLE landed_cost_lines IS 'Individual cost lines within a landed cost document';
COMMENT ON TABLE landed_cost_allocations IS 'Allocation records showing how costs were distributed to receipt items';

COMMENT ON COLUMN landed_cost_documents.allocation_method IS 'Method for distributing costs: by_value (proportional to line value), by_quantity (proportional to quantity), equal (split equally)';
COMMENT ON COLUMN landed_cost_allocations.original_unit_cost IS 'Unit cost before landed cost allocation, stored for audit and potential rollback';
COMMENT ON COLUMN landed_cost_allocations.new_unit_cost IS 'Unit cost after landed cost allocation';
