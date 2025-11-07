-- Migration: Create product_variants table (Product Variations)
-- Description: Creates the product variants table for supporting product variations like color/size
-- Dependencies: Phase 4 database setup, products table
-- Created: 2025-10-29

-- ==================================
-- PRODUCT_VARIANTS TABLE (Product Variations)
-- ==================================
-- This table defines product variations (e.g., color, size) for parent products.
-- Each variant is a distinct record with its own SKU, barcode, and inventory.

CREATE TABLE product_variants (
    -- Primary key using UUID v7 (timestamp-based)
    variant_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Parent product relationship
    parent_product_id UUID NOT NULL REFERENCES products(product_id),

    -- Variant attributes (e.g., {"color": "red", "size": "L"})
    variant_attributes JSONB NOT NULL DEFAULT '{}',

    -- Variant identification
    sku TEXT NOT NULL,
    barcode TEXT,  -- Optional barcode

    -- Pricing (difference from parent product price in cents/xu)
    price_difference BIGINT NOT NULL DEFAULT 0,

    -- Variant lifecycle
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT product_variants_unique_sku_per_tenant
        UNIQUE (tenant_id, sku),
    CONSTRAINT product_variants_unique_variant_per_product
        UNIQUE (tenant_id, parent_product_id, variant_attributes),
    CONSTRAINT product_variants_positive_price_difference
        CHECK (price_difference >= 0),
    CONSTRAINT product_variants_valid_attributes
        CHECK (jsonb_typeof(variant_attributes) = 'object')
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for multi-tenant queries

-- Primary lookup indexes
CREATE INDEX idx_product_variants_tenant_parent
    ON product_variants(tenant_id, parent_product_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_product_variants_tenant_sku
    ON product_variants(tenant_id, sku)
    WHERE deleted_at IS NULL;

-- Query optimization indexes
CREATE INDEX idx_product_variants_tenant_active
    ON product_variants(tenant_id, is_active)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_product_variants_updated_at
    BEFORE UPDATE ON product_variants
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

COMMENT ON TABLE product_variants IS 'Product variations with distinct SKUs and attributes';
COMMENT ON COLUMN product_variants.variant_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN product_variants.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN product_variants.parent_product_id IS 'Reference to parent product';
COMMENT ON COLUMN product_variants.variant_attributes IS 'JSONB object with variant attributes (e.g., color, size)';
COMMENT ON COLUMN product_variants.sku IS 'Stock Keeping Unit - unique per tenant';
COMMENT ON COLUMN product_variants.barcode IS 'Optional barcode for scanning';
COMMENT ON COLUMN product_variants.price_difference IS 'Price adjustment from parent product in smallest currency unit (cents/xu, always >= 0)';
COMMENT ON COLUMN product_variants.is_active IS 'Whether this variant is available for sale';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Product variations (color, size, etc.)
-- 2. Variant-specific inventory tracking
-- 3. Variant-specific pricing
-- 4. SKU and barcode management per variant

-- Example variants for "T-Shirt":
-- - Parent: "Basic T-Shirt" (no variants)
-- - Variant 1: {"color": "red", "size": "M"} SKU: "TSHIRT-RED-M"
-- - Variant 2: {"color": "blue", "size": "L"} SKU: "TSHIRT-BLUE-L"

-- Next migrations will add:
-- - Variant inventory tracking
-- - Variant image management
-- - Bulk variant creation
-- - Variant search and filtering
