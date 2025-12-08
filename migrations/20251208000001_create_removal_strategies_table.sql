-- Migration: Create removal_strategies table
-- Description: Creates the removal_strategies table for configuring inventory picking strategies (FIFO, FEFO, location-based)
-- Dependencies: tenants (20250110000002), warehouses (20250110000023), products (20250110000017)
-- Created: 2025-12-08

-- Define ENUM type for removal strategy types
CREATE TYPE removal_strategy_type AS ENUM ('fifo', 'lifo', 'fefo', 'closest_location', 'least_packages');

-- ==================================
-- REMOVAL_STRATEGIES TABLE (Inventory Removal Strategy Configuration)
-- ==================================
-- This table defines strategies for optimal stock removal during picking operations
-- Supports FIFO, FEFO, and location-based strategies with flexible configuration

CREATE TABLE removal_strategies (
    -- Primary key using UUID v7 (timestamp-based)
    strategy_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Strategy identification
    name VARCHAR(100) NOT NULL,
    type removal_strategy_type NOT NULL,

    -- Scope: warehouse-wide or product-specific
    warehouse_id UUID,           -- NULL for global strategies, specific warehouse otherwise
    product_id UUID,             -- NULL for all products, specific product otherwise

    -- Configuration
    active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,                -- Flexible configuration (e.g., buffer days for FEFO, zone priorities)

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT removal_strategies_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT removal_strategies_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT removal_strategies_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT removal_strategies_tenant_updated_by_fk
        FOREIGN KEY (tenant_id, updated_by)
        REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT removal_strategies_name_not_empty
        CHECK (length(trim(name)) > 0),
    CONSTRAINT removal_strategies_scope_validation
        CHECK (
            (warehouse_id IS NOT NULL) OR
            (product_id IS NOT NULL) OR
            (warehouse_id IS NULL AND product_id IS NULL)
        )
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_removal_strategies_tenant_active
    ON removal_strategies(tenant_id, active)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_removal_strategies_tenant_warehouse
    ON removal_strategies(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL AND warehouse_id IS NOT NULL;

CREATE INDEX idx_removal_strategies_tenant_product
    ON removal_strategies(tenant_id, product_id)
    WHERE deleted_at IS NULL AND product_id IS NOT NULL;

CREATE INDEX idx_removal_strategies_tenant_type
    ON removal_strategies(tenant_id, type, active)
    WHERE deleted_at IS NULL;

-- Unique constraint for strategy names per tenant
CREATE UNIQUE INDEX idx_removal_strategies_unique_name_per_tenant
    ON removal_strategies(tenant_id, name)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_removal_strategies_updated_at
    BEFORE UPDATE ON removal_strategies
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

COMMENT ON TABLE removal_strategies IS 'Removal strategy configurations for inventory picking optimization';
COMMENT ON COLUMN removal_strategies.strategy_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN removal_strategies.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN removal_strategies.name IS 'Human-readable strategy name';
COMMENT ON COLUMN removal_strategies.type IS 'Strategy type: fifo/lifo/fefo/closest_location/least_packages';
COMMENT ON COLUMN removal_strategies.warehouse_id IS 'Warehouse scope (NULL for global)';
COMMENT ON COLUMN removal_strategies.product_id IS 'Product scope (NULL for all products)';
COMMENT ON COLUMN removal_strategies.active IS 'Whether this strategy is currently active';
COMMENT ON COLUMN removal_strategies.config IS 'JSON configuration for strategy-specific settings';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Configurable removal strategies with ENUM types for type safety
-- 2. Multi-tenant isolation with proper FK constraints
-- 3. Flexible scoping (warehouse/product/global)
-- 4. JSONB config for extensible strategy parameters
-- 5. Soft delete support with partial indexes

-- Future migrations will add:
-- - Strategy evaluation engine
-- - Integration with picking operations
-- - Performance analytics
-- - Strategy override mechanisms
