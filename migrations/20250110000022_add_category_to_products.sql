-- Migration: Add category_id to products table
-- Description: Links products to categories and enables category-based organization
-- Dependencies: 20250110000017_create_products_table.sql, 20250110000011_create_product_categories.sql
-- Created: 2025-01-21

-- ==================================
-- ADD CATEGORY COLUMN TO PRODUCTS
-- ==================================

-- Add category_id column to products table
ALTER TABLE products
ADD COLUMN category_id UUID REFERENCES product_categories(category_id) ON DELETE SET NULL;

-- Add index for category-based queries
CREATE INDEX idx_products_tenant_category
    ON products(tenant_id, category_id)
    WHERE deleted_at IS NULL;

-- Add index for filtering products by category
CREATE INDEX idx_products_category_active
    ON products(category_id, is_active)
    WHERE deleted_at IS NULL AND category_id IS NOT NULL;

-- ==================================
-- PRODUCT COUNT UPDATE FUNCTION
-- ==================================

-- Function to maintain category product counts
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
        WHERE (child.path = pc.path OR child.path LIKE pc.path || '/%')
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

-- ==================================
-- ENABLE PRODUCT COUNT TRIGGER
-- ==================================

-- Now that products.category_id exists, create the trigger to maintain counts
CREATE TRIGGER trigger_update_category_product_count
    AFTER INSERT OR UPDATE OF category_id OR DELETE ON products
    FOR EACH ROW
    WHEN (
        (TG_OP = 'DELETE') OR
        (TG_OP = 'INSERT' AND NEW.category_id IS NOT NULL) OR
        (TG_OP = 'UPDATE' AND NEW.category_id IS DISTINCT FROM OLD.category_id)
    )
    EXECUTE FUNCTION update_category_product_count();

-- ==================================
-- UPDATE HELPER FUNCTION
-- ==================================

-- Update the can_delete_category function to check products
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

    -- Check if category has products
    SELECT EXISTS (
        SELECT 1 FROM products
        WHERE category_id = p_category_id
          AND tenant_id = p_tenant_id
          AND deleted_at IS NULL
    ) INTO v_has_products;

    RETURN NOT (v_has_children OR v_has_products);
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- FUNCTION TO MOVE PRODUCTS TO CATEGORY
-- ==================================

-- Helper function to bulk move products to a category
CREATE OR REPLACE FUNCTION move_products_to_category(
    p_product_ids UUID[],
    p_category_id UUID,
    p_tenant_id UUID
)
RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER;
BEGIN
    -- Verify category exists and belongs to tenant
    IF NOT EXISTS (
        SELECT 1 FROM product_categories
        WHERE category_id = p_category_id
          AND tenant_id = p_tenant_id
          AND deleted_at IS NULL
    ) THEN
        RAISE EXCEPTION 'Category not found or does not belong to tenant';
    END IF;

    -- Update products
    UPDATE products
    SET category_id = p_category_id,
        updated_at = NOW()
    WHERE product_id = ANY(p_product_ids)
      AND tenant_id = p_tenant_id
      AND deleted_at IS NULL;

    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- FUNCTION TO GET PRODUCTS BY CATEGORY TREE
-- ==================================

-- Get all products in a category and its subcategories
CREATE OR REPLACE FUNCTION get_products_in_category_tree(
    p_category_id UUID,
    p_tenant_id UUID
)
RETURNS TABLE (
    product_id UUID,
    sku VARCHAR,
    name VARCHAR,
    category_id UUID,
    category_name VARCHAR,
    category_path TEXT
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
        RAISE EXCEPTION 'Category not found';
    END IF;

    -- Return all products in this category and subcategories
    RETURN QUERY
    SELECT
        p.product_id,
        p.sku,
        p.name,
        p.category_id,
        pc.name as category_name,
        pc.path as category_path
    FROM products p
    INNER JOIN product_categories pc ON p.category_id = pc.category_id
    WHERE p.tenant_id = p_tenant_id
      AND pc.tenant_id = p_tenant_id
      AND (pc.path = v_path OR pc.path LIKE v_path || '/%')
      AND p.deleted_at IS NULL
      AND pc.deleted_at IS NULL
    ORDER BY pc.path, p.name;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- COMMENTS
-- ==================================

COMMENT ON COLUMN products.category_id IS 'Links product to category for organization and filtering';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration:
-- 1. Adds category_id to products table
-- 2. Creates indexes for category-based queries
-- 3. Enables automatic category product count updates
-- 4. Provides helper functions for category operations
-- 5. Enables product filtering by category tree

-- Usage examples:
-- - Get all products in Electronics and subcategories:
--   SELECT * FROM get_products_in_category_tree('electronics-uuid', 'tenant-uuid');
-- - Move products to a category:
--   SELECT move_products_to_category(ARRAY['prod1', 'prod2'], 'category-uuid', 'tenant-uuid');
-- - Check if category can be deleted:
--   SELECT can_delete_category('category-uuid', 'tenant-uuid');
