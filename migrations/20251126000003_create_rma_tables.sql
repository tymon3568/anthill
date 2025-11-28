-- Migration: Create RMA Tables
-- Description: Creates tables for Returned Merchandise Authorization (RMA) management
-- Created: 2025-11-26
-- Purpose: Support customer returns and merchandise processing workflow

-- Create sequence for RMA number generation
CREATE SEQUENCE IF NOT EXISTS rma_number_seq START 1;

-- Function to generate RMA numbers (RMA-YYYY-XXXXX)
CREATE OR REPLACE FUNCTION generate_rma_number()
RETURNS TEXT AS $$
DECLARE
    current_year TEXT := TO_CHAR(NOW(), 'YYYY');
    next_number INTEGER;
BEGIN
    -- Get next sequence value
    SELECT nextval('rma_number_seq') INTO next_number;

    -- Return formatted RMA number
    RETURN 'RMA-' || current_year || '-' || LPAD(next_number::TEXT, 5, '0');
END;
$$ LANGUAGE plpgsql;

-- Create rma_requests table
CREATE TABLE rma_requests (
    rma_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rma_number VARCHAR(20) UNIQUE NOT NULL DEFAULT generate_rma_number(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    customer_id UUID NOT NULL, -- References external customer system
    original_delivery_id UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'received', 'processed', 'rejected')),
    return_reason TEXT,
    notes TEXT,
    total_items INTEGER NOT NULL DEFAULT 0,
    total_value BIGINT NOT NULL DEFAULT 0, -- In cents/xu
    currency_code VARCHAR(3) NOT NULL DEFAULT 'VND',
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Foreign key constraint for original_delivery_id (assuming delivery_orders has tenant_id)
    FOREIGN KEY (tenant_id, original_delivery_id) REFERENCES delivery_orders(tenant_id, delivery_id),

    -- Soft delete constraint
    CHECK (deleted_at IS NULL OR deleted_at > created_at)
);

-- Create rma_items table
CREATE TABLE rma_items (
    rma_item_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    rma_id UUID NOT NULL,
    product_id UUID NOT NULL,
    variant_id UUID, -- Optional for product variants
    quantity_returned BIGINT NOT NULL CHECK (quantity_returned > 0),
    condition VARCHAR(20) NOT NULL DEFAULT 'new' CHECK (condition IN ('new', 'used', 'damaged', 'defective')),
    action VARCHAR(20) NOT NULL DEFAULT 'restock' CHECK (action IN ('restock', 'scrap', 'refund', 'exchange')),
    unit_cost BIGINT, -- Cost per unit in cents/xu
    line_total BIGINT, -- Calculated: quantity_returned * unit_cost
    notes TEXT,
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Composite foreign key to rma_requests
    FOREIGN KEY (tenant_id, rma_id) REFERENCES rma_requests(tenant_id, rma_id),

    -- Composite foreign key to products
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id),

    -- Optional foreign key to product_variants
    FOREIGN KEY (tenant_id, variant_id) REFERENCES product_variants(tenant_id, variant_id),

    -- Soft delete constraint
    CHECK (deleted_at IS NULL OR deleted_at > created_at)
);

-- Indexes for performance
CREATE INDEX idx_rma_requests_tenant_status ON rma_requests(tenant_id, status) WHERE deleted_at IS NULL;
CREATE INDEX idx_rma_requests_tenant_customer ON rma_requests(tenant_id, customer_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_rma_requests_tenant_delivery ON rma_requests(tenant_id, original_delivery_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_rma_requests_created_at ON rma_requests(created_at) WHERE deleted_at IS NULL;

CREATE INDEX idx_rma_items_tenant_rma ON rma_items(tenant_id, rma_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_rma_items_tenant_product ON rma_items(tenant_id, product_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_rma_items_tenant_variant ON rma_items(tenant_id, variant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_rma_items_condition_action ON rma_items(condition, action) WHERE deleted_at IS NULL;

-- Triggers for updated_at
CREATE TRIGGER update_rma_requests_updated_at
    BEFORE UPDATE ON rma_requests
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rma_items_updated_at
    BEFORE UPDATE ON rma_items
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to auto-calculate line_total in rma_items
CREATE OR REPLACE FUNCTION calculate_rma_item_total()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.unit_cost IS NOT NULL AND NEW.quantity_returned IS NOT NULL THEN
        NEW.line_total := NEW.unit_cost * NEW.quantity_returned;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER calculate_rma_item_total_trigger
    BEFORE INSERT OR UPDATE OF unit_cost, quantity_returned ON rma_items
    FOR EACH ROW
    EXECUTE FUNCTION calculate_rma_item_total();

-- Comments for documentation
COMMENT ON TABLE rma_requests IS 'Returned Merchandise Authorization requests - manages customer returns workflow';
COMMENT ON TABLE rma_items IS 'RMA line items - details of returned products and processing actions';

COMMENT ON COLUMN rma_requests.rma_number IS 'Auto-generated RMA number in format RMA-YYYY-XXXXX';
COMMENT ON COLUMN rma_requests.total_value IS 'Total value of returned items in cents/xu';
COMMENT ON COLUMN rma_requests.currency_code IS 'Currency code (e.g., VND, USD)';

COMMENT ON COLUMN rma_items.quantity_returned IS 'Number of units returned';
COMMENT ON COLUMN rma_items.condition IS 'Condition of returned item: new, used, damaged, defective';
COMMENT ON COLUMN rma_items.action IS 'Processing action: restock, scrap, refund, exchange';
COMMENT ON COLUMN rma_items.unit_cost IS 'Cost per unit in cents/xu';
COMMENT ON COLUMN rma_items.line_total IS 'Calculated total for this line item';
