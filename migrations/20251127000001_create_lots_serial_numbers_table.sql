-- Migration: Create lots_serial_numbers table
-- Description: Creates the lots_serial_numbers table for tracking individual product units (serial numbers) or batches (lot numbers) with all fixes applied
-- Dependencies: products table (20250110000017), tenants table, warehouse_locations table (20250110000023)
-- Created: 2025-11-27

-- Define ENUM types for better type safety
CREATE TYPE lot_serial_tracking_type AS ENUM ('lot', 'serial');
CREATE TYPE lot_serial_status AS ENUM ('active', 'expired', 'quarantined', 'disposed', 'reserved');

-- ==================================
-- LOTS_SERIAL_NUMBERS TABLE (Lot and Serial Number Tracking)
-- ==================================
-- This table enables traceability by tracking individual product units (serial numbers) or batches (lot numbers)
-- Used for quality control, recalls, and inventory management requiring granular tracking

CREATE TABLE lots_serial_numbers (
    -- Primary key using UUID v7 (timestamp-based)
    lot_serial_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Product relationship
    product_id UUID NOT NULL,

    -- Tracking type and identifiers (using ENUMs for type safety)
    tracking_type lot_serial_tracking_type NOT NULL,
    lot_number VARCHAR(100),     -- For lot tracking (batch numbers)
    serial_number VARCHAR(100),  -- For serial tracking (individual units)

    -- Tracking details
    expiry_date TIMESTAMPTZ,     -- Expiry date for perishable items
    manufacture_date TIMESTAMPTZ,-- Date of manufacture
    batch_reference VARCHAR(100),-- Additional batch reference

    -- Status and lifecycle (using ENUM for type safety)
    status lot_serial_status NOT NULL DEFAULT 'active',

    -- Quantity tracking (for lots)
    initial_quantity BIGINT,     -- Initial quantity in the lot
    remaining_quantity BIGINT,   -- Remaining quantity available

    -- Location and ownership
    warehouse_id UUID,           -- Current warehouse location
    location_id UUID,            -- Specific location within warehouse

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT lots_serial_numbers_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id),
    CONSTRAINT lots_serial_numbers_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT lots_serial_numbers_tenant_location_fk
        FOREIGN KEY (tenant_id, location_id)
        REFERENCES warehouse_locations (tenant_id, location_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT lots_serial_numbers_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT lots_serial_numbers_tenant_updated_by_fk
        FOREIGN KEY (tenant_id, updated_by)
        REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT lots_serial_numbers_tracking_validation
        CHECK (
            (tracking_type = 'lot' AND lot_number IS NOT NULL AND serial_number IS NULL) OR
            (tracking_type = 'serial' AND serial_number IS NOT NULL AND lot_number IS NULL)
        ),
    CONSTRAINT lots_serial_numbers_quantity_validation
        CHECK (
            (tracking_type = 'lot' AND initial_quantity IS NOT NULL AND initial_quantity >= 0 AND remaining_quantity IS NOT NULL AND remaining_quantity >= 0 AND remaining_quantity <= initial_quantity) OR
            (tracking_type = 'serial' AND initial_quantity IS NULL AND remaining_quantity IS NULL)
        )
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_lots_serial_numbers_tenant_product
    ON lots_serial_numbers(tenant_id, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_lots_serial_numbers_tenant_tracking
    ON lots_serial_numbers(tenant_id, tracking_type, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_lots_serial_numbers_tenant_lot
    ON lots_serial_numbers(tenant_id, lot_number)
    WHERE deleted_at IS NULL AND lot_number IS NOT NULL;

CREATE INDEX idx_lots_serial_numbers_tenant_serial
    ON lots_serial_numbers(tenant_id, serial_number)
    WHERE deleted_at IS NULL AND serial_number IS NOT NULL;

CREATE INDEX idx_lots_serial_numbers_tenant_status
    ON lots_serial_numbers(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_lots_serial_numbers_tenant_warehouse
    ON lots_serial_numbers(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL AND warehouse_id IS NOT NULL;

CREATE INDEX idx_lots_serial_numbers_tenant_expiry
    ON lots_serial_numbers(tenant_id, expiry_date)
    WHERE deleted_at IS NULL AND expiry_date IS NOT NULL;

-- Partial unique indexes (for soft delete support)
CREATE UNIQUE INDEX idx_lots_serial_numbers_unique_lot_per_tenant
    ON lots_serial_numbers(tenant_id, product_id, lot_number)
    WHERE deleted_at IS NULL AND lot_number IS NOT NULL;

CREATE UNIQUE INDEX idx_lots_serial_numbers_unique_serial_per_tenant
    ON lots_serial_numbers(tenant_id, product_id, serial_number)
    WHERE deleted_at IS NULL AND serial_number IS NOT NULL;

-- Query optimization indexes
CREATE INDEX idx_lots_serial_numbers_tenant_active
    ON lots_serial_numbers(tenant_id, product_id, status)
    WHERE deleted_at IS NULL AND status = 'active';

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_lots_serial_numbers_updated_at
    BEFORE UPDATE ON lots_serial_numbers
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

COMMENT ON TABLE lots_serial_numbers IS 'Lot and serial number tracking table for granular inventory traceability';
COMMENT ON COLUMN lots_serial_numbers.lot_serial_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN lots_serial_numbers.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN lots_serial_numbers.product_id IS 'Product this lot/serial belongs to';
COMMENT ON COLUMN lots_serial_numbers.tracking_type IS 'Type of tracking: lot (batch) or serial (individual unit)';
COMMENT ON COLUMN lots_serial_numbers.lot_number IS 'Lot/batch number for batch tracking';
COMMENT ON COLUMN lots_serial_numbers.serial_number IS 'Serial number for individual unit tracking';
COMMENT ON COLUMN lots_serial_numbers.expiry_date IS 'Expiry date for perishable items';
COMMENT ON COLUMN lots_serial_numbers.manufacture_date IS 'Date of manufacture';
COMMENT ON COLUMN lots_serial_numbers.batch_reference IS 'Additional batch reference information';
COMMENT ON COLUMN lots_serial_numbers.status IS 'Status: active/expired/quarantined/disposed/reserved';
COMMENT ON COLUMN lots_serial_numbers.initial_quantity IS 'Initial quantity in the lot (for lot tracking)';
COMMENT ON COLUMN lots_serial_numbers.remaining_quantity IS 'Remaining quantity available (for lot tracking)';
COMMENT ON COLUMN lots_serial_numbers.warehouse_id IS 'Current warehouse location';
COMMENT ON COLUMN lots_serial_numbers.location_id IS 'Specific location within warehouse';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Lot and serial number tracking with ENUM types for type safety
-- 2. Product traceability and recalls with proper FK constraints
-- 3. Quality control and expiry management
-- 4. Inventory accuracy with granular tracking
-- 5. Batch and serial number validation with soft delete support

-- Key improvements from PR review:
-- - ENUM types for tracking_type and status instead of VARCHAR+CHECK
-- - Added FK constraint for location_id to warehouse_locations
-- - Partial unique indexes instead of constraints for soft delete compatibility
-- - Non-negative quantity validation
-- - Removed redundant unique constraint on (tenant_id, lot_serial_id)

-- Future migrations will add:
-- - Integration with stock movements
-- - Expiry alerts and notifications
-- - Quality control workflows
-- - Serial number generation functions
-- - Batch splitting and merging operations
