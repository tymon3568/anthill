-- Migration: Create delivery_orders table
-- Description: Creates the delivery_orders table for managing outbound shipments from warehouses
-- Dependencies: warehouses table (20250110000023), tenants table
-- Created: 2025-11-19

-- ==================================
-- DELIVERY_ORDERS TABLE (Outbound Shipments)
-- ==================================
-- This table manages Delivery Orders (DO) which record the shipment of goods out of warehouses
-- DOs are created when goods are shipped to customers and are processed out of inventory

CREATE TABLE delivery_orders (
    -- Primary key using UUID v7 (timestamp-based)
    delivery_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Delivery identifiers
    delivery_number VARCHAR(50) NOT NULL,  -- Auto-generated: DO-2025-00001
    reference_number VARCHAR(100),         -- External reference (order number, etc.)

    -- Warehouse and customer relationships
    warehouse_id UUID NOT NULL,
    order_id UUID,      -- References future orders table (nullable for now)
    customer_id UUID,   -- References future customers table (nullable for now)

    -- Delivery status
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'confirmed', 'partially_picked', 'picked', 'partially_shipped', 'shipped', 'cancelled')),

    -- Dates
    delivery_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),     -- When the DO was created
    expected_ship_date TIMESTAMPTZ,                       -- Expected shipping date
    actual_ship_date TIMESTAMPTZ,                         -- Actual shipping date

    -- Shipping details
    shipping_method VARCHAR(100),                         -- Shipping method (ground, air, sea, etc.)
    carrier VARCHAR(100),                                 -- Shipping carrier (FedEx, UPS, etc.)
    tracking_number VARCHAR(100),                         -- Tracking number from carrier
    shipping_cost BIGINT,                                 -- Shipping cost in smallest currency unit

    -- Delivery details
    notes TEXT,                                           -- Additional notes
    created_by UUID NOT NULL,                             -- User who created the DO
    updated_by UUID,                                      -- User who last updated the DO

    -- Summary fields (calculated from delivery lines)
    total_quantity BIGINT DEFAULT 0,                      -- Total quantity shipped
    total_value BIGINT DEFAULT 0,                         -- Total value in cents
    currency_code VARCHAR(3) DEFAULT 'VND',               -- ISO 4217 currency code

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT delivery_orders_number_unique_per_tenant
        UNIQUE (tenant_id, delivery_number),
    CONSTRAINT delivery_orders_tenant_delivery_unique
        UNIQUE (tenant_id, delivery_id),
    CONSTRAINT delivery_orders_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT delivery_orders_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT delivery_orders_positive_totals
        CHECK (total_quantity >= 0 AND total_value >= 0),
    CONSTRAINT delivery_orders_positive_shipping_cost
        CHECK (shipping_cost IS NULL OR shipping_cost >= 0),
    CONSTRAINT delivery_orders_ship_dates
        CHECK (actual_ship_date IS NULL OR expected_ship_date IS NULL OR actual_ship_date >= expected_ship_date)
);



-- ==================================
-- SEQUENCE FOR DELIVERY NUMBER GENERATION
-- ==================================
-- Global sequence for delivery numbers (DO-YYYY-XXXXX)
-- Note: For production multi-tenant systems, consider per-tenant sequences

CREATE SEQUENCE IF NOT EXISTS delivery_order_number_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- ==================================
-- FUNCTION FOR DELIVERY NUMBER GENERATION
-- ==================================

CREATE OR REPLACE FUNCTION generate_delivery_number()
RETURNS TEXT AS $$
DECLARE
    current_year TEXT := EXTRACT(YEAR FROM NOW())::TEXT;
    next_seq TEXT := LPAD(nextval('delivery_order_number_seq')::TEXT, 5, '0');
BEGIN
    RETURN 'DO-' || current_year || '-' || next_seq;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_delivery_orders_tenant_number
    ON delivery_orders(tenant_id, delivery_number)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_delivery_orders_tenant_status
    ON delivery_orders(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_delivery_orders_tenant_warehouse
    ON delivery_orders(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_delivery_orders_tenant_order
    ON delivery_orders(tenant_id, order_id)
    WHERE deleted_at IS NULL AND order_id IS NOT NULL;

CREATE INDEX idx_delivery_orders_tenant_customer
    ON delivery_orders(tenant_id, customer_id)
    WHERE deleted_at IS NULL AND customer_id IS NOT NULL;

CREATE INDEX idx_delivery_orders_tenant_date
    ON delivery_orders(tenant_id, delivery_date)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_delivery_orders_tenant_created_by
    ON delivery_orders(tenant_id, created_by)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_delivery_orders_tenant_active
    ON delivery_orders(tenant_id, delivery_id)
    WHERE deleted_at IS NULL AND status IN ('confirmed', 'partially_shipped', 'shipped');

CREATE INDEX idx_delivery_orders_tenant_tracking
    ON delivery_orders(tenant_id, tracking_number)
    WHERE deleted_at IS NULL AND tracking_number IS NOT NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_delivery_orders_updated_at
    BEFORE UPDATE ON delivery_orders
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

COMMENT ON TABLE delivery_orders IS 'Delivery Orders (DO) table for recording warehouse outbound shipments';
COMMENT ON COLUMN delivery_orders.delivery_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN delivery_orders.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN delivery_orders.delivery_number IS 'Auto-generated delivery number (DO-YYYY-XXXXX)';
COMMENT ON COLUMN delivery_orders.reference_number IS 'External reference number (sales order, etc.)';
COMMENT ON COLUMN delivery_orders.warehouse_id IS 'Warehouse where goods are shipped from';
COMMENT ON COLUMN delivery_orders.order_id IS 'Reference to sales order (nullable for future orders table)';
COMMENT ON COLUMN delivery_orders.customer_id IS 'Customer receiving the goods (nullable for future customers table)';
COMMENT ON COLUMN delivery_orders.status IS 'Delivery status: draft/confirmed/partially_picked/picked/partially_shipped/shipped/cancelled';
COMMENT ON COLUMN delivery_orders.delivery_date IS 'Date when the DO was created';
COMMENT ON COLUMN delivery_orders.expected_ship_date IS 'Expected shipping date';
COMMENT ON COLUMN delivery_orders.actual_ship_date IS 'Actual shipping date';
COMMENT ON COLUMN delivery_orders.shipping_method IS 'Shipping method (ground, air, sea, etc.)';
COMMENT ON COLUMN delivery_orders.carrier IS 'Shipping carrier name';
COMMENT ON COLUMN delivery_orders.tracking_number IS 'Carrier tracking number';
COMMENT ON COLUMN delivery_orders.shipping_cost IS 'Shipping cost in smallest currency unit';
COMMENT ON COLUMN delivery_orders.notes IS 'Additional notes about the delivery';
COMMENT ON COLUMN delivery_orders.created_by IS 'User ID who created the DO';
COMMENT ON COLUMN delivery_orders.updated_by IS 'User ID who last updated the DO';
COMMENT ON COLUMN delivery_orders.total_quantity IS 'Total quantity of all items in the delivery';
COMMENT ON COLUMN delivery_orders.total_value IS 'Total value of the delivery in smallest currency unit (cents/xu)';
COMMENT ON COLUMN delivery_orders.currency_code IS 'ISO 4217 currency code (VND, USD, etc.)';

COMMENT ON FUNCTION generate_delivery_number() IS 'Generates auto-incrementing delivery numbers in format DO-YYYY-XXXXX';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Delivery order management (DO processing)
-- 2. Warehouse shipping operations
-- 3. Customer delivery tracking
-- 4. Inventory shipment validation
-- 5. Shipment-based stock movements

-- Future migrations will add:
-- - delivery_order_lines table (individual items in deliveries)
-- - Integration with sales orders
-- - Delivery approval workflows
-- - Automated delivery number generation per tenant
-- - Shipping carrier integrations
