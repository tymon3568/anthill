-- Migration: Create picking_methods table
-- Description: Creates the picking_methods table for advanced warehouse picking strategies
-- Dependencies: tenants table (20250110000002), warehouses table (20250110000023)
-- Created: 2025-12-06

-- ==================================
-- PICKING_METHODS TABLE (Advanced Picking Methods Configuration)
-- ==================================
-- This table defines picking methods for optimizing warehouse operations
-- Supports batch picking, cluster picking, and wave picking strategies
-- Configuration stored as JSON for flexible method-specific settings

CREATE TABLE picking_methods (
    -- Primary key using UUID v7 (timestamp-based)
    method_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Method metadata
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Picking method type
    method_type VARCHAR(50) NOT NULL, -- 'batch', 'cluster', 'wave'

    -- Warehouse scope
    warehouse_id UUID NOT NULL,

    -- Method configuration (JSON for flexible settings)
    config JSONB NOT NULL DEFAULT '{}',

    -- Status and flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID,

    -- Soft delete
    deleted_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT picking_methods_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT picking_methods_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT picking_methods_tenant_updated_by_fk
        FOREIGN KEY (tenant_id, updated_by)
        REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT picking_methods_method_type_check
        CHECK (method_type IN ('batch', 'cluster', 'wave')),

    CONSTRAINT picking_methods_unique_default_per_warehouse
        EXCLUDE (tenant_id WITH =, warehouse_id WITH =, is_default WITH =)
        WHERE (is_default = true AND deleted_at IS NULL)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for picking method evaluation and warehouse operations

-- Primary lookup indexes
CREATE INDEX idx_picking_methods_tenant_warehouse
    ON picking_methods(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_picking_methods_tenant_type
    ON picking_methods(tenant_id, method_type)
    WHERE deleted_at IS NULL AND is_active = true;

-- Active methods for quick lookup
CREATE INDEX idx_picking_methods_tenant_active
    ON picking_methods(tenant_id, warehouse_id, is_active)
    WHERE deleted_at IS NULL;

-- Default method lookup
CREATE INDEX idx_picking_methods_tenant_default
    ON picking_methods(tenant_id, warehouse_id, is_default)
    WHERE deleted_at IS NULL AND is_default = true;

-- Query optimization indexes
CREATE INDEX idx_picking_methods_tenant_created_at
    ON picking_methods(tenant_id, created_at)
    WHERE deleted_at IS NULL;

-- ==================================
-- FUNCTIONS
-- ==================================
-- Note: update_updated_at_column() function is defined in initial_extensions.sql

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_picking_methods_updated_at
    BEFORE UPDATE ON picking_methods
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

COMMENT ON TABLE picking_methods IS 'Configuration for advanced warehouse picking methods (batch, cluster, wave)';
COMMENT ON COLUMN picking_methods.method_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN picking_methods.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN picking_methods.name IS 'Human-readable method name';
COMMENT ON COLUMN picking_methods.description IS 'Detailed method description';
COMMENT ON COLUMN picking_methods.method_type IS 'Type of picking method: batch, cluster, wave';
COMMENT ON COLUMN picking_methods.warehouse_id IS 'Warehouse this method applies to';
COMMENT ON COLUMN picking_methods.config IS 'JSON configuration for method-specific settings';
COMMENT ON COLUMN picking_methods.is_active IS 'Whether this method is active for use';
COMMENT ON COLUMN picking_methods.is_default IS 'Whether this is the default method for the warehouse';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Picking method configuration and management
-- 2. Flexible method settings using JSON configuration
-- 3. Multi-tenant isolation and warehouse scoping
-- 4. Default method assignment per warehouse
-- 5. Soft delete support for method lifecycle management

-- Key improvements from task requirements:
-- - Method types: batch, cluster, wave picking
-- - JSON config for flexible method-specific settings
-- - Warehouse scoping for method application
-- - Default method designation
-- - Comprehensive indexing for performance

-- Future migrations will add:
-- - Picking plan generation tables
-- - Picking execution tracking
-- - Performance analytics
-- - Method optimization algorithms
-- - Mobile picking interfaces
