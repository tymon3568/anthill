-- Migration: Create goods_receipt_items table
-- Description: Creates the goods_receipt_items table for storing line items in Goods Receipt Notes (GRN)
-- Dependencies: goods_receipts table (20250110000028), products table (20250110000017), unit_of_measures table (20250110000018)
-- Created: 2025-11-17

-- Add unique constraints required for composite foreign keys
ALTER TABLE goods_receipts ADD CONSTRAINT goods_receipts_tenant_receipt_unique UNIQUE (tenant_id, receipt_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE products ADD CONSTRAINT products_tenant_product_unique UNIQUE (tenant_id, product_id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE unit_of_measures ADD CONSTRAINT unit_of_measures_tenant_uom_unique UNIQUE (tenant_id, uom_id) DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- GOODS_RECEIPT_ITEMS TABLE (GRN Line Items)
-- ==================================
-- This table stores individual line items for each Goods Receipt Note
-- Each item represents a product received in a specific quantity and cost

CREATE TABLE goods_receipt_items (
    -- Primary key using UUID v7 (timestamp-based)
    receipt_item_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Parent receipt relationship
    receipt_id UUID NOT NULL,

    -- Product relationship
    product_id UUID NOT NULL,

    -- Quantities (in smallest unit: pieces, grams, etc.)
    expected_quantity BIGINT NOT NULL DEFAULT 0,  -- Expected quantity from purchase order
    received_quantity BIGINT NOT NULL DEFAULT 0,  -- Actual quantity received

    -- Cost information (stored in smallest currency unit: cents/xu)
    unit_cost BIGINT,  -- Cost per unit in cents
    line_total BIGINT DEFAULT 0,  -- Calculated: received_quantity * unit_cost

    -- Unit of measure
    uom_id UUID,  -- References unit_of_measures table

    -- Lot/Serial tracking
    lot_number VARCHAR(100),  -- Lot number for batch tracking
    serial_numbers JSONB,     -- Array of serial numbers if tracking_method = 'serial'
    expiry_date TIMESTAMPTZ,  -- Expiry date for perishable goods

    -- Item details
    notes TEXT,  -- Additional notes for this line item

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT goods_receipt_items_positive_quantities
        CHECK (expected_quantity >= 0 AND received_quantity >= 0),
    CONSTRAINT goods_receipt_items_positive_cost
        CHECK (unit_cost IS NULL OR unit_cost >= 0),
    CONSTRAINT goods_receipt_items_positive_total
        CHECK (line_total >= 0),
    CONSTRAINT goods_receipt_items_tenant_receipt_fk
        FOREIGN KEY (tenant_id, receipt_id)
        REFERENCES goods_receipts (tenant_id, receipt_id),
    CONSTRAINT goods_receipt_items_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id),
    CONSTRAINT goods_receipt_items_tenant_uom_fk
        FOREIGN KEY (tenant_id, uom_id)
        REFERENCES unit_of_measures (tenant_id, uom_id)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_goods_receipt_items_tenant_receipt
    ON goods_receipt_items(tenant_id, receipt_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_goods_receipt_items_tenant_product
    ON goods_receipt_items(tenant_id, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_goods_receipt_items_tenant_uom
    ON goods_receipt_items(tenant_id, uom_id)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_goods_receipt_items_tenant_lot
    ON goods_receipt_items(tenant_id, lot_number)
    WHERE deleted_at IS NULL AND lot_number IS NOT NULL;

CREATE INDEX idx_goods_receipt_items_tenant_expiry
    ON goods_receipt_items(tenant_id, expiry_date)
    WHERE deleted_at IS NULL AND expiry_date IS NOT NULL;

-- Composite indexes for common queries
CREATE INDEX idx_goods_receipt_items_tenant_receipt_product
    ON goods_receipt_items(tenant_id, receipt_id, product_id)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_goods_receipt_items_updated_at
    BEFORE UPDATE ON goods_receipt_items
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auto-calculate line_total when quantities or unit_cost change
CREATE OR REPLACE FUNCTION calculate_goods_receipt_item_total()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate line_total = received_quantity * unit_cost
    NEW.line_total := COALESCE(NEW.received_quantity, 0) * COALESCE(NEW.unit_cost, 0);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER calculate_goods_receipt_item_total_trigger
    BEFORE INSERT OR UPDATE OF received_quantity, unit_cost
    ON goods_receipt_items
    FOR EACH ROW
    EXECUTE FUNCTION calculate_goods_receipt_item_total();

-- ==================================
-- ROW LEVEL SECURITY (Future)
-- ==================================
-- Note: We use application-level filtering instead of RLS
-- All queries must include: WHERE tenant_id = $1 AND deleted_at IS NULL

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE goods_receipt_items IS 'Line items for Goods Receipt Notes (GRN) - individual products received';
COMMENT ON COLUMN goods_receipt_items.receipt_item_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN goods_receipt_items.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN goods_receipt_items.receipt_id IS 'Reference to parent goods_receipts record';
COMMENT ON COLUMN goods_receipt_items.product_id IS 'Product being received';
COMMENT ON COLUMN goods_receipt_items.expected_quantity IS 'Expected quantity from purchase order/supplier';
COMMENT ON COLUMN goods_receipt_items.received_quantity IS 'Actual quantity received and accepted';
COMMENT ON COLUMN goods_receipt_items.unit_cost IS 'Cost per unit in smallest currency unit (cents/xu)';
COMMENT ON COLUMN goods_receipt_items.line_total IS 'Total cost for this line item (calculated)';
COMMENT ON COLUMN goods_receipt_items.uom_id IS 'Unit of measure for quantities';
COMMENT ON COLUMN goods_receipt_items.lot_number IS 'Lot/batch number for inventory tracking';
COMMENT ON COLUMN goods_receipt_items.serial_numbers IS 'JSON array of serial numbers if applicable';
COMMENT ON COLUMN goods_receipt_items.expiry_date IS 'Expiry date for perishable items';
COMMENT ON COLUMN goods_receipt_items.notes IS 'Additional notes for this receipt item';

COMMENT ON FUNCTION calculate_goods_receipt_item_total() IS 'Auto-calculates line_total from received_quantity * unit_cost';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Detailed GRN line item management
-- 2. Quantity and cost tracking per product
-- 3. Lot and serial number tracking
-- 4. Expiry date management
-- 5. Receipt validation and reconciliation

-- Future migrations may add:
-- - Integration with purchase order lines
-- - Quality control inspections per item
-- - Automated cost averaging
-- - Receipt item approval workflows
