-- Migration: Create inventory_levels table (Stock Levels)
-- Description: Creates the inventory levels table for tracking available and reserved stock per product per tenant
-- Dependencies: Phase 4 database setup, products table
-- Created: 2025-11-20

-- ==================================
-- INVENTORY_LEVELS TABLE (Stock Levels)
-- ==================================
-- This table tracks current stock levels for products.
-- Used for stock reservation during order processing.

CREATE TABLE inventory_levels (
    -- Primary key using UUID v7 (timestamp-based)
    inventory_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Product relationship
    product_id UUID NOT NULL REFERENCES products(product_id),

    -- Stock quantities (BIGINT for large quantities)
    available_quantity BIGINT NOT NULL DEFAULT 0 CHECK (available_quantity >= 0),
    reserved_quantity BIGINT NOT NULL DEFAULT 0 CHECK (reserved_quantity >= 0),

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT inventory_levels_unique_product_per_tenant
        UNIQUE (tenant_id, product_id)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_inventory_levels_tenant_product
    ON inventory_levels(tenant_id, product_id)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_inventory_levels_tenant_available
    ON inventory_levels(tenant_id, available_quantity)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_inventory_levels_updated_at
    BEFORE UPDATE ON inventory_levels
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

COMMENT ON TABLE inventory_levels IS 'Current stock levels for products (available and reserved)';
COMMENT ON COLUMN inventory_levels.inventory_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN inventory_levels.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN inventory_levels.product_id IS 'Reference to product';
COMMENT ON COLUMN inventory_levels.available_quantity IS 'Quantity available for sale (BIGINT)';
COMMENT ON COLUMN inventory_levels.reserved_quantity IS 'Quantity reserved for pending orders (BIGINT)';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Stock level tracking per product per tenant
-- 2. Stock reservation during order processing
-- 3. Inventory reporting and alerts

-- Next considerations:
-- - Add warehouse_id if multi-warehouse support is needed
-- - Add triggers for stock level validation
-- - Add functions for bulk stock updates
