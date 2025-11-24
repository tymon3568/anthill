-- Migration: Create stock_transfers table
-- Description: Creates the stock_transfers table for managing internal stock transfers between warehouses
-- Dependencies: warehouses table (20250110000023), tenants table
-- Created: 2025-11-22

-- ==================================
-- STOCK_TRANSFERS TABLE (Internal Warehouse Transfers)
-- ==================================
-- This table manages Stock Transfers (ST) which record the movement of goods between warehouses
-- within the same tenant. Transfers are used for inventory redistribution and replenishment.

CREATE TABLE stock_transfers (
    -- Primary key using UUID v7 (timestamp-based)
    transfer_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Transfer identifiers
    transfer_number VARCHAR(50) NOT NULL,  -- Auto-generated: ST-2025-00001
    reference_number VARCHAR(100),         -- External reference (optional)

    -- Warehouse relationships (both required for transfers)
    source_warehouse_id UUID NOT NULL,
    destination_warehouse_id UUID NOT NULL,

    -- Transfer status
    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'confirmed', 'partially_picked', 'picked', 'partially_shipped', 'shipped', 'received', 'cancelled')),

    -- Transfer type and priority
    transfer_type VARCHAR(20) NOT NULL DEFAULT 'manual'
        CHECK (transfer_type IN ('manual', 'auto_replenishment', 'emergency', 'consolidation')),
    priority VARCHAR(10) NOT NULL DEFAULT 'normal'
        CHECK (priority IN ('low', 'normal', 'high', 'urgent')),

    -- Dates
    transfer_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),     -- When the transfer was created
    expected_ship_date TIMESTAMPTZ,                       -- Expected shipping date
    actual_ship_date TIMESTAMPTZ,                         -- Actual shipping date
    expected_receive_date TIMESTAMPTZ,                    -- Expected receive date
    actual_receive_date TIMESTAMPTZ,                      -- Actual receive date

    -- Shipping details
    shipping_method VARCHAR(100),                         -- Shipping method (truck, internal, etc.)
    carrier VARCHAR(100),                                 -- Shipping carrier (internal fleet, etc.)
    tracking_number VARCHAR(100),                         -- Tracking number if applicable
    shipping_cost BIGINT,                                 -- Shipping cost in smallest currency unit

    -- Transfer details
    notes TEXT,                                           -- Additional notes
    reason TEXT,                                          -- Reason for transfer
    created_by UUID NOT NULL,                             -- User who created the transfer
    updated_by UUID,                                      -- User who last updated the transfer
    approved_by UUID,                                     -- User who approved the transfer
    approved_at TIMESTAMPTZ,                              -- When transfer was approved

    -- Summary fields (calculated from transfer lines)
    total_quantity BIGINT DEFAULT 0,                      -- Total quantity transferred
    total_value BIGINT DEFAULT 0,                         -- Total value in cents
    currency_code VARCHAR(3) DEFAULT 'VND',               -- ISO 4217 currency code

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT stock_transfers_number_unique_per_tenant
        UNIQUE (tenant_id, transfer_number) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_transfers_different_warehouses
        CHECK (source_warehouse_id != destination_warehouse_id),
    CONSTRAINT stock_transfers_tenant_source_warehouse_fk
        FOREIGN KEY (tenant_id, source_warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT stock_transfers_tenant_destination_warehouse_fk
        FOREIGN KEY (tenant_id, destination_warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT stock_transfers_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT stock_transfers_tenant_updated_by_fk
        FOREIGN KEY (tenant_id, updated_by)
        REFERENCES users (tenant_id, user_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_transfers_tenant_approved_by_fk
        FOREIGN KEY (tenant_id, approved_by)
        REFERENCES users (tenant_id, user_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_transfers_positive_totals
        CHECK (total_quantity >= 0 AND total_value >= 0),
    CONSTRAINT stock_transfers_positive_shipping_cost
        CHECK (shipping_cost IS NULL OR shipping_cost >= 0),
    CONSTRAINT stock_transfers_approval_dates
        CHECK (approved_at IS NULL OR approved_by IS NOT NULL),
    CONSTRAINT stock_transfers_receive_dates
        CHECK (actual_receive_date IS NULL OR expected_receive_date IS NULL OR actual_receive_date >= expected_receive_date),
    CONSTRAINT stock_transfers_ship_dates
        CHECK (actual_ship_date IS NULL OR expected_ship_date IS NULL OR actual_ship_date >= expected_ship_date)
);

-- Add unique constraint for composite foreign keys
ALTER TABLE stock_transfers ADD CONSTRAINT stock_transfers_tenant_transfer_unique UNIQUE (tenant_id, transfer_id);

-- ==================================
-- SEQUENCE FOR TRANSFER NUMBER GENERATION
-- ==================================
-- Global sequence for transfer numbers (ST-YYYY-XXXXX)
-- Note: For production multi-tenant systems, consider per-tenant sequences

CREATE SEQUENCE IF NOT EXISTS stock_transfer_number_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

-- ==================================
-- FUNCTION FOR TRANSFER NUMBER GENERATION
-- ==================================

CREATE OR REPLACE FUNCTION generate_stock_transfer_number()
RETURNS TEXT AS $$
DECLARE
    current_year TEXT := EXTRACT(YEAR FROM NOW())::TEXT;
    next_seq TEXT := LPAD(nextval('stock_transfer_number_seq')::TEXT, 5, '0');
BEGIN
    RETURN 'ST-' || current_year || '-' || next_seq;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_stock_transfers_tenant_number
    ON stock_transfers(tenant_id, transfer_number)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_status
    ON stock_transfers(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_source_warehouse
    ON stock_transfers(tenant_id, source_warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_destination_warehouse
    ON stock_transfers(tenant_id, destination_warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_type
    ON stock_transfers(tenant_id, transfer_type)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_priority
    ON stock_transfers(tenant_id, priority)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_date
    ON stock_transfers(tenant_id, transfer_date)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_transfers_tenant_created_by
    ON stock_transfers(tenant_id, created_by)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_stock_transfers_tenant_active
    ON stock_transfers(tenant_id, transfer_id)
    WHERE deleted_at IS NULL AND status IN ('confirmed', 'partially_shipped', 'shipped', 'received');

CREATE INDEX idx_stock_transfers_tenant_pending
    ON stock_transfers(tenant_id, transfer_id)
    WHERE deleted_at IS NULL AND status IN ('draft', 'confirmed', 'partially_picked', 'picked');

CREATE INDEX idx_stock_transfers_tenant_tracking
    ON stock_transfers(tenant_id, tracking_number)
    WHERE deleted_at IS NULL AND tracking_number IS NOT NULL;

CREATE INDEX idx_stock_transfers_tenant_dates
    ON stock_transfers(tenant_id, expected_ship_date, expected_receive_date)
    WHERE deleted_at IS NULL;

-- Composite indexes for complex queries
CREATE INDEX idx_stock_transfers_tenant_warehouses_status
    ON stock_transfers(tenant_id, source_warehouse_id, destination_warehouse_id, status)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_stock_transfers_updated_at
    BEFORE UPDATE ON stock_transfers
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

COMMENT ON TABLE stock_transfers IS 'Stock Transfers (ST) table for recording internal warehouse-to-warehouse transfers';
COMMENT ON COLUMN stock_transfers.transfer_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN stock_transfers.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN stock_transfers.transfer_number IS 'Auto-generated transfer number (ST-YYYY-XXXXX)';
COMMENT ON COLUMN stock_transfers.reference_number IS 'External reference number (optional)';
COMMENT ON COLUMN stock_transfers.source_warehouse_id IS 'Source warehouse where goods are transferred from';
COMMENT ON COLUMN stock_transfers.destination_warehouse_id IS 'Destination warehouse where goods are transferred to';
COMMENT ON COLUMN stock_transfers.status IS 'Transfer status: draft/confirmed/partially_picked/picked/partially_shipped/shipped/received/cancelled';
COMMENT ON COLUMN stock_transfers.transfer_type IS 'Transfer type: manual/auto_replenishment/emergency/consolidation';
COMMENT ON COLUMN stock_transfers.priority IS 'Transfer priority: low/normal/high/urgent';
COMMENT ON COLUMN stock_transfers.transfer_date IS 'Date when the transfer was created';
COMMENT ON COLUMN stock_transfers.expected_ship_date IS 'Expected shipping date';
COMMENT ON COLUMN stock_transfers.actual_ship_date IS 'Actual shipping date';
COMMENT ON COLUMN stock_transfers.expected_receive_date IS 'Expected receive date';
COMMENT ON COLUMN stock_transfers.actual_receive_date IS 'Actual receive date';
COMMENT ON COLUMN stock_transfers.shipping_method IS 'Shipping method (truck, internal, etc.)';
COMMENT ON COLUMN stock_transfers.carrier IS 'Shipping carrier name';
COMMENT ON COLUMN stock_transfers.tracking_number IS 'Tracking number if applicable';
COMMENT ON COLUMN stock_transfers.shipping_cost IS 'Shipping cost in smallest currency unit';
COMMENT ON COLUMN stock_transfers.notes IS 'Additional notes about the transfer';
COMMENT ON COLUMN stock_transfers.reason IS 'Reason for the transfer';
COMMENT ON COLUMN stock_transfers.created_by IS 'User ID who created the transfer';
COMMENT ON COLUMN stock_transfers.updated_by IS 'User ID who last updated the transfer';
COMMENT ON COLUMN stock_transfers.approved_by IS 'User ID who approved the transfer';
COMMENT ON COLUMN stock_transfers.approved_at IS 'When transfer was approved';
COMMENT ON COLUMN stock_transfers.total_quantity IS 'Total quantity of all items in the transfer';
COMMENT ON COLUMN stock_transfers.total_value IS 'Total value of the transfer in smallest currency unit (cents/xu)';
COMMENT ON COLUMN stock_transfers.currency_code IS 'ISO 4217 currency code (VND, USD, etc.)';

COMMENT ON FUNCTION generate_stock_transfer_number() IS 'Generates auto-incrementing transfer numbers in format ST-YYYY-XXXXX';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Internal warehouse transfer management
-- 2. Inventory redistribution between locations
-- 3. Transfer approval and tracking workflows
-- 4. Multi-echelon inventory management
-- 5. Transfer-based stock movements

-- Key design decisions:
-- - Separate from delivery orders (internal vs external transfers)
-- - Approval workflow for controlled transfers
-- - Both source and destination warehouses required
-- - Transfer types for different business scenarios
-- - Priority levels for operational planning

-- Future migrations will add:
-- - stock_transfer_items table (individual items in transfers)
-- - Transfer approval workflows
-- - Automated transfer triggers
-- - Transfer analytics and reporting
-- - Integration with replenishment systems
