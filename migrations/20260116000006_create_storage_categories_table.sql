-- Migration: Create storage_categories table
-- Description: Creates storage categories table for advanced warehouse configuration
-- Purpose: Foundational configuration layer for putaway rules, location suitability, and picking/removal constraints
-- Dependencies: tenants table (Phase 2 database setup)
-- Task: PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.03_create_storage_categories_table.md
-- Created: 2026-01-16

-- ==================================
-- STORAGE_CATEGORIES TABLE
-- ==================================
-- Configuration table for storage categories used by advanced warehouse processes
-- Referenced by: putaway rules, location suitability, picking/removal strategies (follow-up tasks)
--
-- Design decisions:
-- - Uses `deleted_at` for soft delete (consistent with project standard for config tables)
-- - Composite PK with tenant_id for multi-tenant isolation
-- - Optional `code` field with partial unique index (allows multiple NULLs)
-- - `is_active` boolean for quick filtering without checking deleted_at

CREATE TABLE storage_categories (
    -- Primary key using UUID v4 (standard random UUID)
    storage_category_id UUID NOT NULL DEFAULT gen_random_uuid(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Storage category identifiers
    name TEXT NOT NULL,                    -- Human-readable name (unique per tenant)
    code TEXT,                             -- Optional stable identifier (unique per tenant when non-null)
    description TEXT,                      -- Optional description of the category

    -- Category properties (extensible)
    -- Examples: temperature_range, humidity_range, hazard_level, security_level
    properties JSONB DEFAULT '{}'::JSONB,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,                -- Soft delete (project standard for config tables)

    -- Composite primary key for multi-tenant isolation
    PRIMARY KEY (tenant_id, storage_category_id),

    -- Uniqueness constraints per tenant
    CONSTRAINT storage_categories_name_unique_per_tenant
        UNIQUE (tenant_id, name)
);

-- ==================================
-- PARTIAL UNIQUE INDEX FOR CODE
-- ==================================
-- Ensures code is unique per tenant when non-null
-- Allows multiple NULL values (which a regular UNIQUE constraint would prevent)
CREATE UNIQUE INDEX storage_categories_code_unique_per_tenant
    ON storage_categories (tenant_id, code)
    WHERE code IS NOT NULL AND deleted_at IS NULL;

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Primary lookup by name (most common query pattern)
CREATE INDEX idx_storage_categories_tenant_name
    ON storage_categories (tenant_id, name)
    WHERE deleted_at IS NULL;

-- Lookup by code (when code is used as stable identifier)
CREATE INDEX idx_storage_categories_tenant_code
    ON storage_categories (tenant_id, code)
    WHERE deleted_at IS NULL AND code IS NOT NULL;

-- Active categories filter (common query: list all active categories for a tenant)
CREATE INDEX idx_storage_categories_tenant_active
    ON storage_categories (tenant_id, storage_category_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- ==================================
-- TRIGGER
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_storage_categories_updated_at
    BEFORE UPDATE ON storage_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE storage_categories IS 'Storage category configuration for advanced warehouse management (putaway rules, location suitability, picking strategies)';
COMMENT ON COLUMN storage_categories.storage_category_id IS 'UUID v4 primary key (random)';
COMMENT ON COLUMN storage_categories.tenant_id IS 'Multi-tenant isolation field - all queries must filter by this';
COMMENT ON COLUMN storage_categories.name IS 'Human-readable category name, unique per tenant';
COMMENT ON COLUMN storage_categories.code IS 'Optional stable identifier code, unique per tenant when non-null';
COMMENT ON COLUMN storage_categories.description IS 'Optional description of the storage category';
COMMENT ON COLUMN storage_categories.properties IS 'Extensible properties JSON: {temperature_range, humidity_range, hazard_level, security_level}';
COMMENT ON COLUMN storage_categories.is_active IS 'Whether the category is currently active';
COMMENT ON COLUMN storage_categories.deleted_at IS 'Soft delete timestamp - follows project standard for config tables';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Putaway rules (e.g., "store chilled items in cold zone")
-- 2. Location suitability constraints
-- 3. Picking/removal strategy constraints
--
-- Follow-up tasks may add:
-- - location_storage_categories join table to attach categories to locations
-- - CRUD API endpoints for managing categories
-- - Integration with putaway rule engine
