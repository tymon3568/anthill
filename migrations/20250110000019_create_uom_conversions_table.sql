-- Migration: Create uom_conversions table (UoM Conversion Factors)
-- Description: Creates the unit of measure conversions table for product-specific UoM relationships
-- Dependencies: Phase 2 database setup, products table, unit_of_measures table
-- Created: 2025-10-29

-- ==================================
-- UOM_CONVERSIONS TABLE (UoM Conversion Factors)
-- ==================================
-- This table defines conversion factors between different units of measure
-- for specific products (e.g., 1 Box = 12 Pieces for Product X)
-- Enables flexible unit conversions in inventory and sales transactions

CREATE TABLE uom_conversions (
    -- Primary key using UUID v7 (timestamp-based)
    conversion_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE RESTRICT ON UPDATE RESTRICT,

    -- Product-specific conversion
    product_id UUID NOT NULL REFERENCES products(product_id) ON DELETE RESTRICT ON UPDATE RESTRICT,

    -- Unit conversion relationship
    from_uom_id UUID NOT NULL REFERENCES unit_of_measures(uom_id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    to_uom_id UUID NOT NULL REFERENCES unit_of_measures(uom_id) ON DELETE RESTRICT ON UPDATE RESTRICT,

    -- Conversion factor (e.g., 1 Box = 12 Pieces → factor = 12)
    conversion_factor DECIMAL(20,6) NOT NULL
        CHECK (conversion_factor > 0),

    -- Conversion lifecycle
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT uom_conversions_different_uoms
        CHECK (from_uom_id != to_uom_id),
    CONSTRAINT uom_conversions_unique_per_product_uom_pair
        UNIQUE (tenant_id, product_id, from_uom_id, to_uom_id)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_uom_conversions_tenant_product
    ON uom_conversions(tenant_id, product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_uom_conversions_tenant_from_uom
    ON uom_conversions(tenant_id, from_uom_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_uom_conversions_tenant_to_uom
    ON uom_conversions(tenant_id, to_uom_id)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_uom_conversions_tenant_active
    ON uom_conversions(tenant_id, conversion_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_uom_conversions_updated_at
    BEFORE UPDATE ON uom_conversions
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

COMMENT ON TABLE uom_conversions IS 'Unit of Measure conversion factors for product-specific UoM relationships';
COMMENT ON COLUMN uom_conversions.conversion_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN uom_conversions.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN uom_conversions.product_id IS 'Product this conversion applies to';
COMMENT ON COLUMN uom_conversions.from_uom_id IS 'Source unit of measure for conversion';
COMMENT ON COLUMN uom_conversions.to_uom_id IS 'Target unit of measure for conversion';
COMMENT ON COLUMN uom_conversions.conversion_factor IS 'Conversion multiplier (e.g., 1 Box = 12 Pieces → 12.0)';
COMMENT ON COLUMN uom_conversions.is_active IS 'Whether this conversion is available for use';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Product-specific unit conversions
-- 2. Flexible inventory tracking across different UoMs
-- 3. Sales and purchasing in multiple units
-- 4. Automated conversion calculations

-- Example conversions:
-- Product: "Apple Juice 1L Carton"
-- - 1 Carton = 12 Bottles (factor: 12.0)
-- - 1 Bottle = 1 Liter (factor: 1.0)
-- - 1 Carton = 12 Liters (factor: 12.0)

-- Next migrations will add:
-- - Conversion validation triggers
-- - Bulk conversion operations
-- - Conversion history tracking
