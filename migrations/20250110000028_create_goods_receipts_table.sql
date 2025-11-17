-- Migration: Create goods_receipts table
-- Description: Creates the goods_receipts table for managing Goods Receipt Notes (GRN) in stock operations
-- Dependencies: warehouses table (20250110000023), tenants table
-- Created: 2025-11-17

-- ==================================
-- GOODS_RECEIPTS TABLE (Goods Receipt Notes)
-- ==================================
-- This table manages Goods Receipt Notes (GRN) which record the receiving of goods into warehouses
-- GRNs are created when goods arrive from suppliers and are processed into inventory

CREATE TABLE goods_receipts (
    -- Primary key using UUID v7 (timestamp-based)
    receipt_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Receipt identifiers
    receipt_number VARCHAR(50) NOT NULL,  -- Auto-generated: GRN-2025-00001
    reference_number VARCHAR(100),        -- External reference (PO number, etc.)

    -- Warehouse and supplier relationships
    warehouse_id UUID NOT NULL,
    supplier_id UUID,  -- References future suppliers table (nullable for now)

    -- Receipt status
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'confirmed', 'partially_received', 'received', 'cancelled')),

    -- Dates
    receipt_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),        -- When the GRN was created
    expected_delivery_date TIMESTAMPTZ,                     -- Expected delivery date
    actual_delivery_date TIMESTAMPTZ,                       -- Actual delivery date

    -- Receipt details
    notes TEXT,                                             -- Additional notes
    created_by UUID NOT NULL,                               -- User who created the GRN

    -- Summary fields (calculated from receipt lines)
    total_quantity BIGINT DEFAULT 0,                        -- Total quantity received
    total_value BIGINT DEFAULT 0,                           -- Total value in cents
    currency_code VARCHAR(3) DEFAULT 'VND',                 -- ISO 4217 currency code

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT goods_receipts_number_unique_per_tenant
        UNIQUE (tenant_id, receipt_number) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT goods_receipts_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT goods_receipts_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT goods_receipts_positive_totals
        CHECK (total_quantity >= 0 AND total_value >= 0),
    CONSTRAINT goods_receipts_delivery_dates
        CHECK (actual_delivery_date IS NULL OR expected_delivery_date IS NULL OR actual_delivery_date >= expected_delivery_date - INTERVAL '30 days')
);

-- ==================================
-- SEQUENCE FOR RECEIPT NUMBER GENERATION
-- ==================================
-- Global sequence for receipt numbers (GRN-YYYY-XXXXX)
-- Note: For production multi-tenant systems, consider per-tenant sequences

CREATE SEQUENCE IF NOT EXISTS goods_receipt_number_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- ==================================
-- FUNCTION FOR RECEIPT NUMBER GENERATION
-- ==================================

CREATE OR REPLACE FUNCTION generate_receipt_number()
RETURNS TEXT AS $$
DECLARE
    current_year TEXT := EXTRACT(YEAR FROM NOW())::TEXT;
    next_seq TEXT := LPAD(nextval('goods_receipt_number_seq')::TEXT, 5, '0');
BEGIN
    RETURN 'GRN-' || current_year || '-' || next_seq;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_goods_receipts_tenant_number
    ON goods_receipts(tenant_id, receipt_number)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_goods_receipts_tenant_status
    ON goods_receipts(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_goods_receipts_tenant_warehouse
    ON goods_receipts(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_goods_receipts_tenant_supplier
    ON goods_receipts(tenant_id, supplier_id)
    WHERE deleted_at IS NULL AND supplier_id IS NOT NULL;

CREATE INDEX idx_goods_receipts_tenant_date
    ON goods_receipts(tenant_id, receipt_date)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_goods_receipts_tenant_created_by
    ON goods_receipts(tenant_id, created_by)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_goods_receipts_tenant_active
    ON goods_receipts(tenant_id, receipt_id)
    WHERE deleted_at IS NULL AND status IN ('confirmed', 'partially_received', 'received');

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_goods_receipts_updated_at
    BEFORE UPDATE ON goods_receipts
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

COMMENT ON TABLE goods_receipts IS 'Goods Receipt Notes (GRN) table for recording warehouse goods receipts';
COMMENT ON COLUMN goods_receipts.receipt_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN goods_receipts.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN goods_receipts.receipt_number IS 'Auto-generated receipt number (GRN-YYYY-XXXXX)';
COMMENT ON COLUMN goods_receipts.reference_number IS 'External reference number (purchase order, etc.)';
COMMENT ON COLUMN goods_receipts.warehouse_id IS 'Warehouse where goods are received';
COMMENT ON COLUMN goods_receipts.supplier_id IS 'Supplier providing the goods (nullable for future suppliers table)';
COMMENT ON COLUMN goods_receipts.status IS 'Receipt status: draft/confirmed/partially_received/received/cancelled';
COMMENT ON COLUMN goods_receipts.receipt_date IS 'Date when the GRN was created';
COMMENT ON COLUMN goods_receipts.expected_delivery_date IS 'Expected delivery date from supplier';
COMMENT ON COLUMN goods_receipts.actual_delivery_date IS 'Actual delivery date when goods arrived';
COMMENT ON COLUMN goods_receipts.notes IS 'Additional notes about the receipt';
COMMENT ON COLUMN goods_receipts.created_by IS 'User ID who created the GRN';
COMMENT ON COLUMN goods_receipts.total_quantity IS 'Total quantity of all items in the receipt';
COMMENT ON COLUMN goods_receipts.total_value IS 'Total value of the receipt in smallest currency unit (cents/xu)';
COMMENT ON COLUMN goods_receipts.currency_code IS 'ISO 4217 currency code (VND, USD, etc.)';

COMMENT ON FUNCTION generate_receipt_number() IS 'Generates auto-incrementing receipt numbers in format GRN-YYYY-XXXXX';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Goods receipt management (GRN processing)
-- 2. Warehouse receiving operations
-- 3. Supplier delivery tracking
-- 4. Inventory receipt validation
-- 5. Receipt-based stock movements

-- Future migrations will add:
-- - goods_receipt_lines table (individual items in receipts)
-- - Integration with purchase orders
-- - Receipt approval workflows
-- - Automated receipt number generation per tenant
-- - Receipt quality control and inspection
