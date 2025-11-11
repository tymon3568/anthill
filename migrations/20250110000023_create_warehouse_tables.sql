-- Migration: Create warehouse hierarchy tables
-- Description: Creates warehouses, warehouse_zones, and warehouse_locations tables for hierarchical warehouse management
-- Dependencies: Phase 2 database setup (tenants table)
-- Created: 2025-11-11

-- ==================================
-- WAREHOUSES TABLE (Warehouse Hierarchy)
-- ==================================
-- Root level of warehouse hierarchy with unlimited depth support

CREATE TABLE warehouses (
    -- Primary key using UUID v4 (standard random UUID)
    warehouse_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Warehouse identifiers
    warehouse_code VARCHAR(50) NOT NULL,  -- Unique code per tenant (e.g., "WH001", "MAIN")
    warehouse_name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Warehouse classification
    warehouse_type VARCHAR(50) NOT NULL DEFAULT 'main'
        CHECK (warehouse_type IN ('main', 'transit', 'quarantine', 'distribution', 'retail', 'satellite')),

    -- Hierarchy support (unlimited depth)
    parent_warehouse_id UUID,

    -- Location and contact information
    address JSONB,  -- {street, city, state, postal_code, country, latitude, longitude}
    contact_info JSONB,  -- {phone, email, manager_name, manager_id}

    -- Capacity and operational data
    capacity_info JSONB,  -- {max_volume_m3, max_weight_kg, max_pallets, operating_hours}

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT warehouses_code_unique_per_tenant
        UNIQUE (tenant_id, warehouse_code) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT warehouses_tenant_warehouse_unique
        UNIQUE (tenant_id, warehouse_id),
    CONSTRAINT warehouses_no_self_reference
        CHECK (warehouse_id != parent_warehouse_id),
    CONSTRAINT warehouses_tenant_parent_fk
        FOREIGN KEY (tenant_id, parent_warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- WAREHOUSE_ZONES TABLE (Internal Organization)
-- ==================================
-- Zones within warehouses for organizational purposes

CREATE TABLE warehouse_zones (
    -- Primary key using UUID v4 (standard random UUID)
    zone_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Warehouse relationship
    warehouse_id UUID NOT NULL REFERENCES warehouses(warehouse_id),

    -- Zone identifiers
    zone_code VARCHAR(50) NOT NULL,  -- Unique code per warehouse (e.g., "A01", "PICKING")
    zone_name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Zone classification
    zone_type VARCHAR(50) NOT NULL DEFAULT 'storage'
        CHECK (zone_type IN ('storage', 'picking', 'quarantine', 'receiving', 'shipping', 'bulk', 'damaged', 'returns')),

    -- Zone properties
    zone_attributes JSONB,  -- {temperature_controlled, hazardous_materials, security_level}

    -- Capacity information
    capacity_info JSONB,  -- {max_volume_m3, max_weight_kg, max_locations}

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT warehouse_zones_code_unique_per_warehouse
        UNIQUE (warehouse_id, zone_code) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT warehouse_zones_warehouse_zone_unique
        UNIQUE (warehouse_id, zone_id),
    CONSTRAINT warehouse_zones_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- WAREHOUSE_LOCATIONS TABLE (Storage Positions)
-- ==================================
-- Individual storage locations within warehouse zones

CREATE TABLE warehouse_locations (
    -- Primary key using UUID v4 (standard random UUID)
    location_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Warehouse and zone relationships
    warehouse_id UUID NOT NULL REFERENCES warehouses(warehouse_id),
    zone_id UUID REFERENCES warehouse_zones(zone_id),

    -- Location identifiers
    location_code VARCHAR(100) NOT NULL,  -- Unique code per warehouse (e.g., "A01-01-01", "BIN-001")
    location_name VARCHAR(255),
    description TEXT,

    -- Location classification
    location_type VARCHAR(50) NOT NULL DEFAULT 'bin'
        CHECK (location_type IN ('bin', 'shelf', 'pallet', 'floor', 'rack', 'container', 'bulk')),

    -- Physical coordinates and dimensions
    coordinates JSONB,  -- {aisle, rack, level, position, x, y, z}
    dimensions JSONB,   -- {length_mm, width_mm, height_mm, volume_m3}

    -- Capacity and operational data
    capacity_info JSONB,  -- {max_volume_m3, max_weight_kg, max_units, stacking_limit}

    -- Location properties
    location_attributes JSONB,  -- {powered, climate_controlled, accessible, priority}

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT warehouse_locations_code_unique_per_warehouse
        UNIQUE (warehouse_id, location_code) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT warehouse_locations_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT warehouse_locations_zone_fk
        FOREIGN KEY (warehouse_id, zone_id)
        REFERENCES warehouse_zones (warehouse_id, zone_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Warehouses indexes
CREATE INDEX idx_warehouses_tenant_code
    ON warehouses(tenant_id, warehouse_code)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouses_tenant_active
    ON warehouses(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL AND is_active = true;

CREATE INDEX idx_warehouses_tenant_parent
    ON warehouses(tenant_id, parent_warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouses_tenant_type
    ON warehouses(tenant_id, warehouse_type)
    WHERE deleted_at IS NULL;

-- Warehouse zones indexes
CREATE INDEX idx_warehouse_zones_tenant_warehouse
    ON warehouse_zones(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouse_zones_warehouse_code
    ON warehouse_zones(warehouse_id, zone_code)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouse_zones_tenant_type
    ON warehouse_zones(tenant_id, zone_type)
    WHERE deleted_at IS NULL;

-- Warehouse locations indexes
CREATE INDEX idx_warehouse_locations_tenant_warehouse
    ON warehouse_locations(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouse_locations_warehouse_code
    ON warehouse_locations(warehouse_id, location_code)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouse_locations_zone
    ON warehouse_locations(tenant_id, zone_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_warehouse_locations_tenant_type
    ON warehouse_locations(tenant_id, location_type)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamps
CREATE TRIGGER update_warehouses_updated_at
    BEFORE UPDATE ON warehouses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_warehouse_zones_updated_at
    BEFORE UPDATE ON warehouse_zones
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_warehouse_locations_updated_at
    BEFORE UPDATE ON warehouse_locations
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

COMMENT ON TABLE warehouses IS 'Warehouse hierarchy root table with unlimited depth support';
COMMENT ON COLUMN warehouses.warehouse_id IS 'UUID v4 primary key (random)';
COMMENT ON COLUMN warehouses.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN warehouses.warehouse_code IS 'Unique warehouse code per tenant';
COMMENT ON COLUMN warehouses.warehouse_type IS 'Warehouse classification: main/transit/quarantine/distribution/retail/satellite';
COMMENT ON COLUMN warehouses.parent_warehouse_id IS 'Parent warehouse for hierarchy (NULL for root warehouses)';
COMMENT ON COLUMN warehouses.address IS 'Warehouse address JSON: {street, city, state, postal_code, country, coordinates}';
COMMENT ON COLUMN warehouses.contact_info IS 'Contact information JSON: {phone, email, manager}';
COMMENT ON COLUMN warehouses.capacity_info IS 'Capacity information JSON: {max_volume_m3, max_weight_kg, max_pallets}';

COMMENT ON TABLE warehouse_zones IS 'Warehouse zones for internal organization within warehouses';
COMMENT ON COLUMN warehouse_zones.zone_id IS 'UUID v4 primary key (random)';
COMMENT ON COLUMN warehouse_zones.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN warehouse_zones.warehouse_id IS 'Reference to parent warehouse';
COMMENT ON COLUMN warehouse_zones.zone_code IS 'Unique zone code per warehouse';
COMMENT ON COLUMN warehouse_zones.zone_type IS 'Zone classification: storage/picking/quarantine/receiving/shipping/bulk';
COMMENT ON COLUMN warehouse_zones.zone_attributes IS 'Zone attributes JSON: {temperature_controlled, hazardous, security_level}';
COMMENT ON COLUMN warehouse_zones.capacity_info IS 'Zone capacity JSON: {max_volume_m3, max_weight_kg, max_locations}';

COMMENT ON TABLE warehouse_locations IS 'Individual storage locations within warehouse zones';
COMMENT ON COLUMN warehouse_locations.location_id IS 'UUID v4 primary key (random)';
COMMENT ON COLUMN warehouse_locations.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN warehouse_locations.warehouse_id IS 'Reference to parent warehouse';
COMMENT ON COLUMN warehouse_locations.zone_id IS 'Optional reference to warehouse zone';
COMMENT ON COLUMN warehouse_locations.location_code IS 'Unique location code per warehouse';
COMMENT ON COLUMN warehouse_locations.location_type IS 'Location type: bin/shelf/pallet/floor/rack/container/bulk';
COMMENT ON COLUMN warehouse_locations.coordinates IS 'Physical coordinates JSON: {aisle, rack, level, position, x, y, z}';
COMMENT ON COLUMN warehouse_locations.dimensions IS 'Location dimensions JSON: {length_mm, width_mm, height_mm, volume_m3}';
COMMENT ON COLUMN warehouse_locations.capacity_info IS 'Location capacity JSON: {max_volume_m3, max_weight_kg, max_units}';
COMMENT ON COLUMN warehouse_locations.location_attributes IS 'Location attributes JSON: {powered, climate_controlled, accessible}';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Hierarchical warehouse management (unlimited depth)
-- 2. Zone-based warehouse organization
-- 3. Detailed location tracking for inventory
-- 4. Capacity planning and utilization tracking
-- 5. Warehouse transfer and consolidation operations

-- Next features will include:
-- - Inventory location assignments
-- - Warehouse transfer APIs
-- - Capacity utilization analytics
-- - Location optimization algorithms
