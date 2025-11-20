-- Migration: Create products table (Item Master)
-- Description: Creates the core products table for inventory management
-- Dependencies: Phase 2 database setup (tenants table)
-- Created: 2025-10-29

-- ==================================
-- PRODUCTS TABLE (Item Master)
-- ==================================
-- This table serves as the single source of truth for all product data
-- in the multi-tenant inventory system.

CREATE TABLE products (
    -- Primary key using UUID v7 (timestamp-based)
    product_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Core product identifiers
    sku VARCHAR(100) NOT NULL,  -- Stock Keeping Unit (unique per tenant)
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Product classification
    product_type VARCHAR(50) NOT NULL DEFAULT 'goods'
        CHECK (product_type IN ('goods', 'service', 'consumable')),
    item_group_id UUID,  -- References future item_groups table

    -- Inventory tracking
    track_inventory BOOLEAN NOT NULL DEFAULT true,
    tracking_method VARCHAR(20) DEFAULT 'none'
        CHECK (tracking_method IN ('none', 'lot', 'serial')),

    -- Units of measure
    default_uom_id UUID,  -- References future unit_of_measures table

    -- Pricing (stored in smallest currency unit: cents/xu)
    sale_price BIGINT,  -- Sale price in cents
    cost_price BIGINT,  -- Cost price in cents
    currency_code VARCHAR(3) DEFAULT 'VND',  -- ISO 4217 currency code

    -- Product attributes
    weight_grams INTEGER,  -- Weight in grams for shipping calculations
    dimensions JSONB,     -- Length, width, height in mm
    attributes JSONB,     -- Flexible product attributes (color, size, etc.)

    -- Product lifecycle
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_sellable BOOLEAN NOT NULL DEFAULT true,
    is_purchaseable BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT products_sku_unique_per_tenant
        UNIQUE (tenant_id, sku) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT products_tenant_product_unique
        UNIQUE (tenant_id, product_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT products_positive_prices
        CHECK (sale_price IS NULL OR sale_price >= 0),
    CONSTRAINT products_positive_cost
        CHECK (cost_price IS NULL OR cost_price >= 0),
    CONSTRAINT products_positive_weight
        CHECK (weight_grams IS NULL OR weight_grams > 0)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_products_tenant_sku
    ON products(tenant_id, sku)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_products_tenant_active
    ON products(tenant_id, product_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- Foreign key indexes (for future tables)
CREATE INDEX idx_products_tenant_item_group
    ON products(tenant_id, item_group_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_products_tenant_uom
    ON products(tenant_id, default_uom_id)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_products_tenant_type
    ON products(tenant_id, product_type)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_products_tenant_tracking
    ON products(tenant_id, tracking_method)
    WHERE deleted_at IS NULL AND track_inventory = true;

-- Full-text search index for product search
CREATE INDEX idx_products_search
    ON products
    USING GIN (to_tsvector('english', name || ' ' || COALESCE(description, '')))
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_products_updated_at
    BEFORE UPDATE ON products
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

COMMENT ON TABLE products IS 'Item Master table - Single source of truth for product data';
COMMENT ON COLUMN products.product_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN products.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN products.sku IS 'Stock Keeping Unit - unique per tenant';
COMMENT ON COLUMN products.product_type IS 'Product classification: goods/service/consumable';
COMMENT ON COLUMN products.track_inventory IS 'Whether this product tracks inventory levels';
COMMENT ON COLUMN products.tracking_method IS 'Inventory tracking: none/lot/serial';
COMMENT ON COLUMN products.sale_price IS 'Sale price in smallest currency unit (cents/xu)';
COMMENT ON COLUMN products.cost_price IS 'Cost price in smallest currency unit';
COMMENT ON COLUMN products.currency_code IS 'ISO 4217 currency code (VND, USD, etc.)';
COMMENT ON COLUMN products.weight_grams IS 'Product weight in grams for shipping';
COMMENT ON COLUMN products.dimensions IS 'Product dimensions JSON: {length_mm, width_mm, height_mm}';
COMMENT ON COLUMN products.attributes IS 'Flexible product attributes JSON';
COMMENT ON COLUMN products.is_active IS 'Whether product is active for transactions';
COMMENT ON COLUMN products.is_sellable IS 'Whether product can be sold';
COMMENT ON COLUMN products.is_purchaseable IS 'Whether product can be purchased';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Product catalog management
-- 2. Inventory tracking
-- 3. Pricing and costing
-- 4. Product variants (future)
-- 5. Lot/Serial tracking (future)
-- 6. Product search and filtering

-- Next migrations will add:
-- - unit_of_measures table
-- - item_groups table
-- - product_variants table
-- - Foreign key constraints to new tables
