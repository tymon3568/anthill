-- Migration: Create storage_categories table
-- Description: Storage categories for advanced warehouse configuration (putaway rules, location suitability, picking constraints)
-- Dependencies: tenants table (Phase 2)
-- Created: 2026-01-16
-- Task: task_04.02.03_create_storage_categories_table.md

-- ==================================
-- STORAGE_CATEGORIES TABLE
-- ==================================
-- Configuration table for storage categorization used by:
-- - Putaway rules (e.g., "store chilled items in cold zone")
-- - Location suitability constraints
-- - Picking/removal strategy constraints
--
-- Design decisions:
-- - Uses composite PK (tenant_id, storage_category_id) for tenant isolation
-- - Includes deleted_at for soft delete (consistent with project patterns)
-- - Partial indexes for active records WHERE deleted_at IS NULL

CREATE TABLE storage_categories (
    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Primary key using UUID v4
    storage_category_id UUID NOT NULL DEFAULT gen_random_uuid(),

    -- Category identifiers
    name TEXT NOT NULL,                -- Human-readable name (e.g., "Chilled", "Hazardous")
    code TEXT,                         -- Optional stable identifier (e.g., "CHILLED", "HAZ")
    description TEXT,                  -- Optional description

    -- Category attributes for matching/filtering
    attributes JSONB,                  -- Flexible attributes: {temperature_range, humidity, security_level, etc.}

    -- Display order for UI
    sort_order INT NOT NULL DEFAULT 0,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,            -- Soft delete (project standard for config tables)

    -- Composite primary key for tenant isolation
    PRIMARY KEY (tenant_id, storage_category_id)
);

-- ==================================
-- UNIQUENESS CONSTRAINTS
-- ==================================

-- Unique name per tenant (active records only)
CREATE UNIQUE INDEX idx_storage_categories_unique_name
    ON storage_categories(tenant_id, name)
    WHERE deleted_at IS NULL;

-- Unique code per tenant where code is not null (active records only)
CREATE UNIQUE INDEX idx_storage_categories_unique_code
    ON storage_categories(tenant_id, code)
    WHERE deleted_at IS NULL AND code IS NOT NULL;

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Name lookup (supports name searches)
CREATE INDEX idx_storage_categories_tenant_name
    ON storage_categories(tenant_id, name)
    WHERE deleted_at IS NULL;

-- Code lookup (supports code lookups)
CREATE INDEX idx_storage_categories_tenant_code
    ON storage_categories(tenant_id, code)
    WHERE deleted_at IS NULL AND code IS NOT NULL;

-- Active categories list (for dropdown/listing queries)
CREATE INDEX idx_storage_categories_tenant_active
    ON storage_categories(tenant_id, sort_order)
    WHERE deleted_at IS NULL AND is_active = TRUE;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_storage_categories_updated_at
    BEFORE UPDATE ON storage_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE storage_categories IS 'Storage category configuration for advanced warehouse management';
COMMENT ON COLUMN storage_categories.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN storage_categories.storage_category_id IS 'UUID v4 primary key';
COMMENT ON COLUMN storage_categories.name IS 'Human-readable category name (unique per tenant)';
COMMENT ON COLUMN storage_categories.code IS 'Optional stable code identifier (unique per tenant when not null)';
COMMENT ON COLUMN storage_categories.description IS 'Optional description of the category';
COMMENT ON COLUMN storage_categories.attributes IS 'Flexible JSONB attributes: {temperature_range, humidity, security_level, etc.}';
COMMENT ON COLUMN storage_categories.sort_order IS 'Display order for UI listings';
COMMENT ON COLUMN storage_categories.is_active IS 'Whether the category is available for use';
COMMENT ON COLUMN storage_categories.deleted_at IS 'Soft delete timestamp (NULL = active)';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Storage category management for warehouse configuration
-- 2. Putaway rule constraints (follow-up)
-- 3. Location suitability matching (follow-up)
-- 4. Picking/removal strategy constraints (follow-up)

-- Next features will include:
-- - API endpoints for storage category CRUD
-- - Join table location_storage_categories (if needed)
-- - Integration with putaway rules engine
