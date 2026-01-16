-- Migration: Create inventory valuation settings table
-- Description: Tenant-scoped valuation method configuration with optional product/category overrides
-- Task: 04.06.03 - Inventory Valuation Methods
-- Created: 2026-01-16

-- ==================================
-- INVENTORY_VALUATION_SETTINGS TABLE
-- ==================================
-- Stores valuation method configuration at different scope levels:
-- - Tenant default (scope_type = 'tenant', scope_id = NULL)
-- - Category override (scope_type = 'category', scope_id = category_id)
-- - Product override (scope_type = 'product', scope_id = product_id)
--
-- Precedence: product > category > tenant default

CREATE TABLE inventory_valuation_settings (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,

    -- Scope definition
    -- 'tenant' = tenant-wide default (scope_id NULL)
    -- 'category' = category-level override (scope_id = category_id)
    -- 'product' = product-level override (scope_id = product_id)
    scope_type TEXT NOT NULL CHECK (scope_type IN ('tenant', 'category', 'product')),
    scope_id UUID NULL,

    -- Valuation method
    method TEXT NOT NULL CHECK (method IN ('fifo', 'avco', 'standard', 'lifo')),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT chk_scope_id_required_for_overrides
        CHECK (
            (scope_type = 'tenant' AND scope_id IS NULL) OR
            (scope_type != 'tenant' AND scope_id IS NOT NULL)
        )
);

-- ==================================
-- UNIQUE CONSTRAINTS
-- ==================================

-- Unique constraint for tenant defaults (scope_id is NULL)
CREATE UNIQUE INDEX idx_valuation_settings_tenant_unique
    ON inventory_valuation_settings(tenant_id, scope_type)
    WHERE scope_type = 'tenant';

-- Unique constraint for category/product overrides (scope_id is NOT NULL)
CREATE UNIQUE INDEX idx_valuation_settings_override_unique
    ON inventory_valuation_settings(tenant_id, scope_type, scope_id)
    WHERE scope_id IS NOT NULL;

-- ==================================
-- INDEXES
-- ==================================

-- Index for looking up category overrides
CREATE INDEX idx_valuation_settings_category
    ON inventory_valuation_settings(tenant_id, scope_id)
    WHERE scope_type = 'category';

-- Index for looking up product overrides
CREATE INDEX idx_valuation_settings_product
    ON inventory_valuation_settings(tenant_id, scope_id)
    WHERE scope_type = 'product';

-- ==================================
-- TRIGGERS
-- ==================================

-- Update updated_at timestamp
CREATE TRIGGER update_valuation_settings_updated_at
    BEFORE UPDATE ON inventory_valuation_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS
-- ==================================

COMMENT ON TABLE inventory_valuation_settings IS 'Tenant-scoped valuation method configuration with hierarchical overrides';
COMMENT ON COLUMN inventory_valuation_settings.scope_type IS 'Scope level: tenant (default), category (override), product (override)';
COMMENT ON COLUMN inventory_valuation_settings.scope_id IS 'UUID of category or product for overrides, NULL for tenant default';
COMMENT ON COLUMN inventory_valuation_settings.method IS 'Valuation method: fifo, avco (weighted average), standard, lifo';
