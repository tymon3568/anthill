-- Migration: Fix product_categories soft-delete unique constraints
-- Description: Replace UNIQUE CONSTRAINTs with partial unique indexes that exclude soft-deleted records
-- Issue: When a category is soft-deleted, its slug/code/name still blocks new categories with the same values
-- Solution: Use partial unique indexes with WHERE deleted_at IS NULL
-- Created: 2026-01-27

-- ==================================
-- DROP EXISTING UNIQUE CONSTRAINTS
-- ==================================

-- Drop the unique constraints that include soft-deleted records
ALTER TABLE product_categories
    DROP CONSTRAINT IF EXISTS product_categories_name_unique_per_parent;

ALTER TABLE product_categories
    DROP CONSTRAINT IF EXISTS product_categories_code_unique_per_tenant;

ALTER TABLE product_categories
    DROP CONSTRAINT IF EXISTS product_categories_slug_unique_per_tenant;

-- Note: product_categories_tenant_category_unique is used by foreign keys from other tables
-- (putaway_rules, cycle_count_schedule_categories), so we cannot drop it.
-- This constraint on (tenant_id, category_id) doesn't cause issues with soft-delete
-- since category_id is unique and won't be reused.

-- ==================================
-- CREATE PARTIAL UNIQUE INDEXES
-- ==================================
-- These indexes enforce uniqueness ONLY for non-deleted records

-- Drop indexes if they already exist (in case of partial migration)
DROP INDEX IF EXISTS product_categories_name_unique_per_parent;
DROP INDEX IF EXISTS product_categories_code_unique_per_tenant;
DROP INDEX IF EXISTS product_categories_slug_unique_per_tenant;
DROP INDEX IF EXISTS product_categories_tenant_category_active_unique;

-- Name must be unique within the same parent category (for active records)
CREATE UNIQUE INDEX product_categories_name_unique_per_parent
    ON product_categories (tenant_id, parent_category_id, name)
    WHERE deleted_at IS NULL;

-- Code must be unique per tenant (for active records)
CREATE UNIQUE INDEX product_categories_code_unique_per_tenant
    ON product_categories (tenant_id, code)
    WHERE deleted_at IS NULL AND code IS NOT NULL;

-- Slug must be unique per tenant (for active records)
CREATE UNIQUE INDEX product_categories_slug_unique_per_tenant
    ON product_categories (tenant_id, slug)
    WHERE deleted_at IS NULL AND slug IS NOT NULL;

-- Note: We do NOT recreate product_categories_tenant_category_unique as a partial index
-- because it's used by foreign keys from other tables (putaway_rules, cycle_count_schedule_categories).
-- The original constraint on (tenant_id, category_id) doesn't cause issues with soft-delete
-- since category_id is a UUID and won't be reused.

-- ==================================
-- MIGRATION NOTES
-- ==================================
--
-- Before this migration:
--   - UNIQUE CONSTRAINT on (tenant_id, parent_category_id, name) - includes deleted records
--   - UNIQUE CONSTRAINT on (tenant_id, code) - includes deleted records
--   - UNIQUE CONSTRAINT on (tenant_id, slug) - includes deleted records
--
-- After this migration:
--   - UNIQUE INDEX on (tenant_id, parent_category_id, name) WHERE deleted_at IS NULL
--   - UNIQUE INDEX on (tenant_id, code) WHERE deleted_at IS NULL AND code IS NOT NULL
--   - UNIQUE INDEX on (tenant_id, slug) WHERE deleted_at IS NULL AND slug IS NOT NULL
--
-- This allows:
--   - Soft-deleted categories to be "recreated" with the same slug/code/name
--   - Multiple soft-deleted categories can have the same values
--   - Only active (non-deleted) categories are subject to uniqueness rules
