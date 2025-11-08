-- Migration: Create product_categories table
-- Description: Creates hierarchical product category structure with materialized path
-- Dependencies: tenants table from Phase 2
-- Created: 2025-01-21

-- ==================================
-- PRODUCT_CATEGORIES TABLE
-- ==================================
-- Hierarchical category structure for product organization
-- Uses materialized path pattern for efficient tree queries

CREATE TABLE product_categories (
    -- Primary key using UUID v7 (timestamp-based)
    category_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Hierarchy structure
    parent_category_id UUID REFERENCES product_categories(category_id) ON DELETE CASCADE,

    -- Category information
    name VARCHAR(255) NOT NULL,
    description TEXT,
    code VARCHAR(100),  -- Optional category code for integration

    -- Materialized path for efficient tree operations
    -- Example: "1/5/12" means root->parent->current
    path TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 0,  -- 0 for root, 1 for first level, etc.

    -- Display attributes
    display_order INTEGER NOT NULL DEFAULT 0,  -- Order within same level
    icon VARCHAR(100),  -- Icon name/class for UI
    color VARCHAR(7),   -- Hex color code (e.g., #FF5733)
    image_url TEXT,     -- Optional category image

    -- Category settings
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_visible BOOLEAN NOT NULL DEFAULT true,  -- Show in public catalogs

    -- SEO and metadata
    slug VARCHAR(255),  -- URL-friendly identifier
    meta_title VARCHAR(255),
    meta_description TEXT,
    meta_keywords TEXT,

    -- Statistics (denormalized for performance)
    product_count INTEGER NOT NULL DEFAULT 0,  -- Number of products in this category
    total_product_count INTEGER NOT NULL DEFAULT 0,  -- Including subcategories

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT product_categories_name_unique_per_parent
        UNIQUE (tenant_id, parent_category_id, name) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT product_categories_code_unique_per_tenant
        UNIQUE (tenant_id, code) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT product_categories_slug_unique_per_tenant
        UNIQUE (tenant_id, slug) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT product_categories_positive_level
        CHECK (level >= 0),
    CONSTRAINT product_categories_positive_order
        CHECK (display_order >= 0),
    CONSTRAINT product_categories_valid_color
        CHECK (color IS NULL OR color ~* '^#[0-9A-F]{6}$'),
    CONSTRAINT product_categories_no_self_reference
        CHECK (category_id != parent_category_id)
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Primary lookup indexes
CREATE INDEX idx_product_categories_tenant
    ON product_categories(tenant_id, category_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_product_categories_tenant_active
    ON product_categories(tenant_id, is_active)
    WHERE deleted_at IS NULL;

-- Hierarchy navigation indexes
CREATE INDEX idx_product_categories_tenant_parent
    ON product_categories(tenant_id, parent_category_id, display_order)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_product_categories_tenant_path
    ON product_categories(tenant_id, path)
    WHERE deleted_at IS NULL;

-- For finding all descendants (materialized path pattern)
CREATE INDEX idx_product_categories_path_pattern
    ON product_categories USING btree(tenant_id, path text_pattern_ops)
    WHERE deleted_at IS NULL;

-- For level-based queries
CREATE INDEX idx_product_categories_tenant_level
    ON product_categories(tenant_id, level)
    WHERE deleted_at IS NULL;

-- Slug lookup for SEO URLs
CREATE INDEX idx_product_categories_tenant_slug
    ON product_categories(tenant_id, slug)
    WHERE deleted_at IS NULL;

-- Code lookup for integrations
CREATE INDEX idx_product_categories_tenant_code
    ON product_categories(tenant_id, code)
    WHERE deleted_at IS NULL AND code IS NOT NULL;

-- Full-text search index
CREATE INDEX idx_product_categories_search
    ON product_categories
    USING GIN (to_tsvector('english', name || ' ' || COALESCE(description, '')))
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_product_categories_updated_at
    BEFORE UPDATE ON product_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger to auto-calculate path and level
CREATE OR REPLACE FUNCTION update_product_category_path()
RETURNS TRIGGER AS $$
DECLARE
    parent_path TEXT;
    parent_level INTEGER;
BEGIN
    -- Root category (no parent)
    IF NEW.parent_category_id IS NULL THEN
        NEW.path := NEW.category_id::TEXT;
        NEW.level := 0;
    ELSE
        -- Get parent's path and level
        SELECT path, level INTO parent_path, parent_level
        FROM product_categories
        WHERE category_id = NEW.parent_category_id
          AND tenant_id = NEW.tenant_id
          AND deleted_at IS NULL;

        IF NOT FOUND THEN
            RAISE EXCEPTION 'Parent category not found or belongs to different tenant';
        END IF;

        -- Build path and set level
        NEW.path := parent_path || '/' || NEW.category_id::TEXT;
        NEW.level := parent_level + 1;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_product_category_path
    BEFORE INSERT OR UPDATE OF parent_category_id ON product_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_product_category_path();

-- Trigger to update product counts
CREATE OR REPLACE FUNCTION update_category_product_count()
RETURNS TRIGGER AS $$
BEGIN
    -- Update direct product count for the category
    UPDATE product_categories
    SET product_count = (
        SELECT COUNT(*)
        FROM products
        WHERE category_id = product_categories.category_id
          AND tenant_id = product_categories.tenant_id
          AND deleted_at IS NULL
    )
    WHERE category_id = COALESCE(NEW.category_id, OLD.category_id)
      AND tenant_id = COALESCE(NEW.tenant_id, OLD.tenant_id);

    -- Update total counts for all ancestor categories
    UPDATE product_categories pc
    SET total_product_count = (
        SELECT COUNT(*)
        FROM products p
        JOIN product_categories child ON p.category_id = child.category_id
        WHERE child.path LIKE pc.path || '%'
          AND child.tenant_id = pc.tenant_id
          AND p.tenant_id = pc.tenant_id
          AND p.deleted_at IS NULL
    )
    WHERE pc.tenant_id = COALESCE(NEW.tenant_id, OLD.tenant_id)
      AND (
        pc.path = ANY(
            SELECT DISTINCT substring(path from '^([^/]+(/[^/]+)*)')
            FROM product_categories
            WHERE category_id = COALESCE(NEW.category_id, OLD.category_id)
        )
      );

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Note: This trigger will be created after products table is updated with category_id
-- CREATE TRIGGER trigger_update_category_product_count
--     AFTER INSERT OR UPDATE OR DELETE ON products
--     FOR EACH ROW
--     WHEN (NEW.category_id IS DISTINCT FROM OLD.category_id)
--     EXECUTE FUNCTION update_category_product_count();

-- ==================================
-- HELPER FUNCTIONS
-- ==================================

-- Function to get all ancestor categories
CREATE OR REPLACE FUNCTION get_category_ancestors(p_category_id UUID, p_tenant_id UUID)
RETURNS TABLE (
    category_id UUID,
    name VARCHAR,
    level INTEGER,
    path TEXT
) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE ancestors AS (
        -- Start with the given category
        SELECT c.category_id, c.name, c.level, c.path, c.parent_category_id
        FROM product_categories c
        WHERE c.category_id = p_category_id
          AND c.tenant_id = p_tenant_id
          AND c.deleted_at IS NULL

        UNION ALL

        -- Recursively get parent categories
        SELECT c.category_id, c.name, c.level, c.path, c.parent_category_id
        FROM product_categories c
        INNER JOIN ancestors a ON c.category_id = a.parent_category_id
        WHERE c.tenant_id = p_tenant_id
          AND c.deleted_at IS NULL
    )
    SELECT a.category_id, a.name, a.level, a.path
    FROM ancestors a
    ORDER BY a.level;
END;
$$ LANGUAGE plpgsql;

-- Function to get all descendant categories
CREATE OR REPLACE FUNCTION get_category_descendants(p_category_id UUID, p_tenant_id UUID)
RETURNS TABLE (
    category_id UUID,
    name VARCHAR,
    level INTEGER,
    path TEXT
) AS $$
DECLARE
    v_path TEXT;
BEGIN
    -- Get the path of the given category
    SELECT path INTO v_path
    FROM product_categories
    WHERE category_id = p_category_id
      AND tenant_id = p_tenant_id
      AND deleted_at IS NULL;

    IF v_path IS NULL THEN
        RETURN;
    END IF;

    RETURN QUERY
    SELECT c.category_id, c.name, c.level, c.path
    FROM product_categories c
    WHERE c.tenant_id = p_tenant_id
      AND c.path LIKE v_path || '%'
      AND c.category_id != p_category_id
      AND c.deleted_at IS NULL
    ORDER BY c.path;
END;
$$ LANGUAGE plpgsql;

-- Function to check if category can be deleted
CREATE OR REPLACE FUNCTION can_delete_category(p_category_id UUID, p_tenant_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    v_has_children BOOLEAN;
    v_has_products BOOLEAN;
BEGIN
    -- Check if category has child categories
    SELECT EXISTS (
        SELECT 1 FROM product_categories
        WHERE parent_category_id = p_category_id
          AND tenant_id = p_tenant_id
          AND deleted_at IS NULL
    ) INTO v_has_children;

    -- Check if category has products (will be checked after products table update)
    -- SELECT EXISTS (
    --     SELECT 1 FROM products
    --     WHERE category_id = p_category_id
    --       AND tenant_id = p_tenant_id
    --       AND deleted_at IS NULL
    -- ) INTO v_has_products;
    v_has_products := FALSE;

    RETURN NOT (v_has_children OR v_has_products);
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE product_categories IS 'Hierarchical product category structure with materialized path';
COMMENT ON COLUMN product_categories.category_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN product_categories.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN product_categories.parent_category_id IS 'Parent category for hierarchy (NULL for root)';
COMMENT ON COLUMN product_categories.path IS 'Materialized path for efficient tree queries (e.g., "uuid1/uuid2/uuid3")';
COMMENT ON COLUMN product_categories.level IS 'Depth in hierarchy (0 for root, 1 for first level, etc.)';
COMMENT ON COLUMN product_categories.display_order IS 'Sort order within same level';
COMMENT ON COLUMN product_categories.icon IS 'Icon name/class for UI representation';
COMMENT ON COLUMN product_categories.color IS 'Hex color code for UI (#RRGGBB)';
COMMENT ON COLUMN product_categories.slug IS 'URL-friendly identifier for SEO';
COMMENT ON COLUMN product_categories.product_count IS 'Direct product count (denormalized)';
COMMENT ON COLUMN product_categories.total_product_count IS 'Total including subcategories (denormalized)';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates:
-- 1. Hierarchical category structure with unlimited depth
-- 2. Materialized path pattern for efficient queries
-- 3. Automatic path/level calculation via triggers
-- 4. Helper functions for tree navigation
-- 5. Support for category analytics and statistics
-- 6. SEO-friendly slugs and metadata

-- Next steps:
-- 1. Add category_id to products table
-- 2. Create product-category linking functionality
-- 3. Implement category-based filtering and search
-- 4. Add category analytics endpoints
