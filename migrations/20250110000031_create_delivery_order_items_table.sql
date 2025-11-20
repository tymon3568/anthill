-- Migration: Create delivery_order_items table
-- Description: Creates the delivery_order_items table for line items in Delivery Orders
-- Dependencies: delivery_orders (20250110000030), products (20250110000017), unit_of_measures (20250110000018)
-- Created: 2025-11-20

-- ==================================
-- DELIVERY_ORDER_ITEMS TABLE (Delivery Line Items)
-- ==================================
-- This table stores individual line items for each Delivery Order (DO)
-- Each item represents a product being shipped in a delivery

CREATE TABLE delivery_order_items (
    -- Primary key using UUID v7 (timestamp-based)
    delivery_item_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Relationships
    delivery_id UUID NOT NULL,
    product_id UUID NOT NULL,
    uom_id UUID,  -- Unit of measure for this line item

    -- Quantities (in smallest unit of measure)
    ordered_quantity BIGINT NOT NULL,    -- Quantity ordered for delivery
    picked_quantity BIGINT DEFAULT 0,    -- Quantity picked from warehouse
    delivered_quantity BIGINT DEFAULT 0, -- Quantity actually delivered

    -- Pricing (stored in smallest currency unit: cents/xu)
    unit_price BIGINT,                   -- Price per unit
    line_total BIGINT,                   -- Total for this line (calculated)

    -- Additional details
    notes TEXT,                          -- Line-specific notes
    batch_number VARCHAR(100),           -- Batch/lot number if applicable
    expiry_date DATE,                    -- Expiry date if applicable

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT delivery_order_items_tenant_delivery_fk
        FOREIGN KEY (tenant_id, delivery_id)
        REFERENCES delivery_orders (tenant_id, delivery_id),
    CONSTRAINT delivery_order_items_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id),
    CONSTRAINT delivery_order_items_tenant_uom_fk
        FOREIGN KEY (tenant_id, uom_id)
        REFERENCES unit_of_measures (tenant_id, uom_id),
    CONSTRAINT delivery_order_items_positive_quantities
        CHECK (ordered_quantity > 0 AND picked_quantity >= 0 AND delivered_quantity >= 0),
    CONSTRAINT delivery_order_items_picked_not_exceed_ordered
        CHECK (picked_quantity <= ordered_quantity),
    CONSTRAINT delivery_order_items_delivered_not_exceed_picked
        CHECK (delivered_quantity <= picked_quantity),
    CONSTRAINT delivery_order_items_positive_prices
        CHECK (unit_price IS NULL OR unit_price >= 0),
    CONSTRAINT delivery_order_items_positive_line_total
        CHECK (line_total IS NULL OR line_total >= 0)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_delivery_order_items_tenant_delivery
    ON delivery_order_items(tenant_id, delivery_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_delivery_order_items_tenant_product
    ON delivery_order_items(tenant_id, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_delivery_order_items_tenant_uom
    ON delivery_order_items(tenant_id, uom_id)
    WHERE deleted_at IS NULL AND uom_id IS NOT NULL;

-- Composite indexes for common queries
CREATE INDEX idx_delivery_order_items_tenant_delivery_product
    ON delivery_order_items(tenant_id, delivery_id, product_id)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_delivery_order_items_tenant_batch
    ON delivery_order_items(tenant_id, batch_number)
    WHERE deleted_at IS NULL AND batch_number IS NOT NULL;

CREATE INDEX idx_delivery_order_items_tenant_expiry
    ON delivery_order_items(tenant_id, expiry_date)
    WHERE deleted_at IS NULL AND expiry_date IS NOT NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_delivery_order_items_updated_at
    BEFORE UPDATE ON delivery_order_items
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

COMMENT ON TABLE delivery_order_items IS 'Line items for Delivery Orders - individual products in each delivery';
COMMENT ON COLUMN delivery_order_items.delivery_item_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN delivery_order_items.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN delivery_order_items.delivery_id IS 'Reference to parent delivery order';
COMMENT ON COLUMN delivery_order_items.product_id IS 'Product being delivered';
COMMENT ON COLUMN delivery_order_items.uom_id IS 'Unit of measure for quantities';
COMMENT ON COLUMN delivery_order_items.ordered_quantity IS 'Quantity ordered for delivery';
COMMENT ON COLUMN delivery_order_items.picked_quantity IS 'Quantity picked from warehouse inventory';
COMMENT ON COLUMN delivery_order_items.delivered_quantity IS 'Quantity actually delivered to customer';
COMMENT ON COLUMN delivery_order_items.unit_price IS 'Price per unit in smallest currency unit';
COMMENT ON COLUMN delivery_order_items.line_total IS 'Total value for this line item';
COMMENT ON COLUMN delivery_order_items.batch_number IS 'Batch or lot number for traceability';
COMMENT ON COLUMN delivery_order_items.expiry_date IS 'Expiry date for perishable items';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Detailed delivery line item management
-- 2. Quantity tracking through delivery process
-- 3. Line-level pricing and totals
-- 4. Batch/lot tracking for deliveries
-- 5. Integration with inventory picking

-- Future migrations may add:
-- - Triggers to update delivery_order totals
-- - Integration with stock moves
-- - Delivery item approval workflows
-- - Automated quantity validation
