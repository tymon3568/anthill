-- Migration: Create storage_locations table
-- Description: Creates the storage_locations table for warehouse location management with hierarchical structure
-- Dependencies: warehouses table (20250110000023)
-- Created: 2025-12-05

-- ==================================
-- STORAGE_LOCATIONS TABLE (Warehouse Storage Locations)
-- ==================================
-- This table manages individual storage locations within warehouses
-- Supports hierarchical warehouse structure: zone -> aisle -> rack -> level -> position
-- Used for putaway rules, inventory tracking, and picking optimization

CREATE TABLE storage_locations (
    -- Primary key using UUID v7 (timestamp-based)
    location_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Warehouse relationship
    warehouse_id UUID NOT NULL,

    -- Location identification
    location_code VARCHAR(50) NOT NULL,  -- Human-readable code (e.g., "A-01-02-03-04")

    -- Location type (picking, bulk, quarantine, etc.)
    location_type VARCHAR(50) NOT NULL DEFAULT 'standard',

    -- Hierarchical structure
    zone VARCHAR(50),      -- Zone within warehouse (e.g., "A", "B")
    aisle VARCHAR(50),     -- Aisle number (e.g., "01", "02")
    rack VARCHAR(50),      -- Rack identifier (e.g., "R01", "R02")
    level INTEGER,         -- Level/shelf number (1, 2, 3...)
    position INTEGER,      -- Position on level (1, 2, 3...)

    -- Capacity and current stock
    capacity BIGINT,       -- Maximum capacity in base units
    current_stock BIGINT DEFAULT 0,  -- Current stock quantity

    -- Status and flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_quarantine BOOLEAN NOT NULL DEFAULT false,
    is_picking_location BOOLEAN NOT NULL DEFAULT true,

    -- Dimensions (optional, for space optimization)
    length_cm INTEGER,
    width_cm INTEGER,
    height_cm INTEGER,
    weight_limit_kg INTEGER,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID,

    -- Soft delete
    deleted_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT storage_locations_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),
    CONSTRAINT storage_locations_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT storage_locations_tenant_updated_by_fk
        FOREIGN KEY (tenant_id, updated_by)
        REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED,
    -- Composite unique constraint for FK referenceability (e.g., stock_takes.location_id)
    CONSTRAINT storage_locations_tenant_location_unique
        UNIQUE (tenant_id, location_id),
    CONSTRAINT storage_locations_capacity_check
        CHECK (capacity IS NULL OR capacity > 0),
    CONSTRAINT storage_locations_current_stock_check
        CHECK (current_stock >= 0),
    CONSTRAINT storage_locations_stock_within_capacity_check
        CHECK (capacity IS NULL OR current_stock <= capacity),
    CONSTRAINT storage_locations_dimensions_check
        CHECK (
            (length_cm IS NULL AND width_cm IS NULL AND height_cm IS NULL) OR
            (length_cm > 0 AND width_cm > 0 AND height_cm > 0)
        ),
    CONSTRAINT storage_locations_hierarchy_check
        CHECK (
            -- At least one hierarchical field must be specified
            zone IS NOT NULL OR aisle IS NOT NULL OR rack IS NOT NULL OR
            level IS NOT NULL OR position IS NOT NULL
        )
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries and putaway operations

-- Primary lookup indexes
CREATE INDEX idx_storage_locations_tenant_warehouse
    ON storage_locations(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;



CREATE INDEX idx_storage_locations_tenant_type
    ON storage_locations(tenant_id, location_type)
    WHERE deleted_at IS NULL AND is_active = true;

-- Hierarchical navigation indexes
CREATE INDEX idx_storage_locations_tenant_hierarchy
    ON storage_locations(tenant_id, warehouse_id, zone, aisle, rack, level, position)
    WHERE deleted_at IS NULL;

-- Capacity and stock management indexes
CREATE INDEX idx_storage_locations_tenant_capacity
    ON storage_locations(tenant_id, warehouse_id, capacity, current_stock)
    WHERE deleted_at IS NULL AND is_active = true;

-- Picking optimization indexes
CREATE INDEX idx_storage_locations_tenant_picking
    ON storage_locations(tenant_id, warehouse_id, location_type, current_stock)
    WHERE deleted_at IS NULL AND is_active = true AND is_picking_location = true;

-- Query optimization indexes
CREATE INDEX idx_storage_locations_tenant_active
    ON storage_locations(tenant_id, warehouse_id, is_active)
    WHERE deleted_at IS NULL;

-- Uniqueness constraints
CREATE UNIQUE INDEX uq_storage_locations_tenant_wh_code
    ON storage_locations(tenant_id, warehouse_id, location_code)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_storage_locations_updated_at
    BEFORE UPDATE ON storage_locations
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

COMMENT ON TABLE storage_locations IS 'Warehouse storage locations with hierarchical structure for putaway and picking optimization';
COMMENT ON COLUMN storage_locations.location_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN storage_locations.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN storage_locations.warehouse_id IS 'Warehouse this location belongs to';
COMMENT ON COLUMN storage_locations.location_code IS 'Human-readable location code (e.g., "A-01-R01-02-03")';
COMMENT ON COLUMN storage_locations.location_type IS 'Type of location: standard, bulk, quarantine, damaged, etc.';
COMMENT ON COLUMN storage_locations.zone IS 'Zone within warehouse for grouping locations';
COMMENT ON COLUMN storage_locations.aisle IS 'Aisle identifier within zone';
COMMENT ON COLUMN storage_locations.rack IS 'Rack identifier within aisle';
COMMENT ON COLUMN storage_locations.level IS 'Level/shelf number on rack';
COMMENT ON COLUMN storage_locations.position IS 'Position number on level';
COMMENT ON COLUMN storage_locations.capacity IS 'Maximum capacity in base units';
COMMENT ON COLUMN storage_locations.current_stock IS 'Current stock quantity at this location';
COMMENT ON COLUMN storage_locations.is_active IS 'Whether this location is active for use';
COMMENT ON COLUMN storage_locations.is_quarantine IS 'Whether this location is for quarantined stock';
COMMENT ON COLUMN storage_locations.is_picking_location IS 'Whether this location is used for picking operations';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Warehouse location management with hierarchical structure
-- 2. Capacity planning and stock level tracking
-- 3. Putaway rule evaluation and location suggestions
-- 4. Picking optimization and warehouse layout design
-- 5. Quarantine and special location handling

-- Key improvements from task requirements:
-- - Hierarchical structure supports complex warehouse layouts
-- - Capacity and stock tracking for putaway validation
-- - Location types for different storage strategies
-- - Soft delete support for location management
-- - Comprehensive indexing for performance

-- Future migrations will add:
-- - Putaway rules integration
-- - Location utilization analytics
-- - Automated location assignment
-- - Mobile picking interfaces
-- - Location maintenance workflows
