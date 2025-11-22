-- Migration: Create stock_transfer_items table
-- Description: Creates the stock_transfer_items table for storing line items in Stock Transfers (ST)
-- Dependencies: stock_transfers table (20250121000001), products table (20250110000017), unit_of_measures table (20250110000018)
-- Created: 2025-11-22

-- ==================================
-- STOCK_TRANSFER_ITEMS TABLE (ST Line Items)
-- ==================================
-- This table stores individual line items for each Stock Transfer
-- Each item represents a product being transferred between warehouses

CREATE TABLE stock_transfer_items (
    -- Primary key using UUID v7 (timestamp-based)
    transfer_item_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Parent transfer relationship
    transfer_id UUID NOT NULL,

    -- Product relationship
    product_id UUID NOT NULL,

    -- Quantities (in smallest unit: pieces, grams, etc.)
    quantity BIGINT NOT NULL DEFAULT 0,  -- Quantity to be transferred

    -- Unit of measure
    uom_id UUID,  -- References unit_of_measures table

    -- Cost information (stored in smallest currency unit: cents/xu)
    unit_cost BIGINT,  -- Cost per unit in cents (for valuation)
    line_total BIGINT DEFAULT 0,  -- Calculated: quantity * unit_cost

    -- Item ordering and details
    line_number INTEGER NOT NULL DEFAULT 1,  -- Line number for ordering items
    notes TEXT,  -- Additional notes for this line item

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT stock_transfer_items_positive_quantity
        CHECK (quantity >= 0),
    CONSTRAINT stock_transfer_items_positive_cost
        CHECK (unit_cost IS NULL OR unit_cost >= 0),
    CONSTRAINT stock_transfer_items_positive_total
        CHECK (line_total >= 0),
    CONSTRAINT stock_transfer_items_tenant_transfer_fk
        FOREIGN KEY (tenant_id, transfer_id)
        REFERENCES stock_transfers (tenant_id, transfer_id),
    CONSTRAINT stock_transfer_items_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id),
    CONSTRAINT stock_transfer_items_tenant_uom_fk
        FOREIGN KEY (tenant_id, uom_id)
        REFERENCES unit_of_measures (tenant_id, uom_id)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_stock_transfer_items_tenant_transfer
    ON stock_transfer_items(tenant_id, transfer_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfer_items_tenant_product
    ON stock_transfer_items(tenant_id, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfer_items_tenant_uom
    ON stock_transfer_items(tenant_id, uom_id)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_stock_transfer_items_tenant_line_number
    ON stock_transfer_items(tenant_id, transfer_id, line_number)
    WHERE deleted_at IS NULL;

-- Composite indexes for common queries
CREATE INDEX idx_stock_transfer_items_tenant_transfer_product
    ON stock_transfer_items(tenant_id, transfer_id, product_id)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_stock_transfer_items_updated_at
    BEFORE UPDATE ON stock_transfer_items
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auto-calculate line_total when quantities or unit_cost change
CREATE OR REPLACE FUNCTION calculate_stock_transfer_item_total()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate line_total = quantity * unit_cost
    NEW.line_total := COALESCE(NEW.quantity, 0) * COALESCE(NEW.unit_cost, 0);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER calculate_stock_transfer_item_total_trigger
    BEFORE INSERT OR UPDATE OF quantity, unit_cost
    ON stock_transfer_items
    FOR EACH ROW
    EXECUTE FUNCTION calculate_stock_transfer_item_total();

-- ==================================
-- ROW LEVEL SECURITY (Future)
-- ==================================
-- Note: We use application-level filtering instead of RLS
-- All queries must include: WHERE tenant_id = $1 AND deleted_at IS NULL

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE stock_transfer_items IS 'Line items for Stock Transfers (ST) - individual products being transferred';
COMMENT ON COLUMN stock_transfer_items.transfer_item_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN stock_transfer_items.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN stock_transfer_items.transfer_id IS 'Reference to parent stock_transfers record';
COMMENT ON COLUMN stock_transfer_items.product_id IS 'Product being transferred';
COMMENT ON COLUMN stock_transfer_items.quantity IS 'Quantity to be transferred in base UOM';
COMMENT ON COLUMN stock_transfer_items.uom_id IS 'Unit of measure for quantities';
COMMENT ON COLUMN stock_transfer_items.unit_cost IS 'Cost per unit in smallest currency unit (cents/xu)';
COMMENT ON COLUMN stock_transfer_items.line_total IS 'Total cost for this line item (calculated)';
COMMENT ON COLUMN stock_transfer_items.line_number IS 'Line number for ordering items in the transfer';
COMMENT ON COLUMN stock_transfer_items.notes IS 'Additional notes for this transfer item';

COMMENT ON FUNCTION calculate_stock_transfer_item_total() IS 'Auto-calculates line_total from quantity * unit_cost';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Detailed ST line item management
-- 2. Product-specific transfer quantities
-- 3. Cost tracking per transferred item
-- 4. Transfer item ordering and notes
-- 5. Transfer validation and reconciliation

-- Key design decisions:
-- - Separate from stock_transfers for normalization
-- - Cost tracking for inventory valuation
-- - Line numbers for item ordering
-- - Soft delete for audit trails

-- Future migrations may add:
-- - Integration with inventory levels
-- - Transfer item approval workflows
-- - Automated cost updates from product costs
-- - Transfer item status tracking
