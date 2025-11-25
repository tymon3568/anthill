-- Migration: Create unit_of_measures table (UoM Master)
-- Description: Creates the units of measure table for product definitions
-- Dependencies: Phase 2 database setup (tenants table)
-- Created: 2025-10-29

-- ==================================
-- UNIT_OF_MEASURES TABLE (UoM Master)
-- ==================================
-- This table defines all possible units of measure for products
-- Used for inventory tracking, pricing, and conversions

CREATE TABLE unit_of_measures (
    -- Primary key using UUID v7 (timestamp-based)
    uom_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Unit of measure details
    name VARCHAR(255) NOT NULL,  -- Display name (e.g., "Piece", "Kilogram", "Liter")
    uom_type VARCHAR(50) NOT NULL DEFAULT 'reference',
        -- reference: base unit (e.g., "Piece")
        -- smaller: subunit (e.g., "Box" contains multiple "Piece")
        -- bigger: larger unit (e.g., "Pallet" contains multiple "Box")

    -- Unit classification
    category VARCHAR(50) NOT NULL DEFAULT 'count',
        -- count: discrete items (pieces, boxes)
        -- weight: mass-based (kg, gram, pound)
        -- volume: liquid/solid volume (liter, cubic meter)
        -- length: linear measurements (meter, inch)
        -- area: surface area (square meter)
        -- time: time-based (hour, day)

    -- Precision for decimal calculations
    rounding_precision INTEGER NOT NULL DEFAULT 2,

    -- Unit lifecycle
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT unit_of_measures_name_unique_per_tenant
        UNIQUE (tenant_id, name) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT unit_of_measures_uom_type_check
        CHECK (uom_type IN ('reference', 'smaller', 'bigger')),
    CONSTRAINT unit_of_measures_category_check
        CHECK (category IN ('count', 'weight', 'volume', 'length', 'area', 'time')),
    CONSTRAINT unit_of_measures_rounding_precision_check
        CHECK (rounding_precision >= 0 AND rounding_precision <= 6)
);

-- Add unique constraint for composite foreign keys
ALTER TABLE unit_of_measures ADD CONSTRAINT unit_of_measures_tenant_uom_unique UNIQUE (tenant_id, uom_id);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_unit_of_measures_tenant_name
    ON unit_of_measures(tenant_id, name)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_unit_of_measures_tenant_active
    ON unit_of_measures(tenant_id, uom_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- Query optimization indexes
CREATE INDEX idx_unit_of_measures_tenant_category
    ON unit_of_measures(tenant_id, category)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_unit_of_measures_tenant_type
    ON unit_of_measures(tenant_id, uom_type)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_unit_of_measures_updated_at
    BEFORE UPDATE ON unit_of_measures
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

COMMENT ON TABLE unit_of_measures IS 'Units of Measure master table - defines measurement units for products';
COMMENT ON COLUMN unit_of_measures.uom_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN unit_of_measures.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN unit_of_measures.name IS 'Display name of the unit (e.g., "Piece", "Kilogram")';
COMMENT ON COLUMN unit_of_measures.uom_type IS 'Unit type: reference/smaller/bigger for conversion relationships';
COMMENT ON COLUMN unit_of_measures.category IS 'Unit category: count/weight/volume/length/area/time';
COMMENT ON COLUMN unit_of_measures.rounding_precision IS 'Decimal precision for calculations (0-6)';
COMMENT ON COLUMN unit_of_measures.is_active IS 'Whether this UoM is available for new products';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Product unit definitions
-- 2. Unit conversions and relationships
-- 3. Inventory quantity tracking
-- 4. Pricing calculations

-- Next migrations will add:
-- - uom_conversions table for unit relationships
-- - Foreign key from products.default_uom_id
-- - Seed data for common units
