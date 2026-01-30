-- Migration: Fix products SKU constraint for upsert operations
-- Description: Changes the (tenant_id, sku) unique constraint from DEFERRABLE to IMMEDIATE
--              so that ON CONFLICT clause works properly for CSV import upsert
-- Created: 2026-01-30
-- Related: BUG-004 in 8.10.6_Products_Enhancement

-- ==================================
-- PROBLEM
-- ==================================
-- The original constraint was:
--   CONSTRAINT products_sku_unique_per_tenant
--       UNIQUE (tenant_id, sku) DEFERRABLE INITIALLY DEFERRED
--
-- PostgreSQL's ON CONFLICT clause doesn't work with deferred constraints because
-- conflict detection happens at INSERT time, but deferred constraints are only
-- checked at COMMIT time.
--
-- Error encountered:
--   "there is no unique or exclusion constraint matching the ON CONFLICT specification"

-- ==================================
-- FIX
-- ==================================
-- Drop the deferred constraint and create a regular (immediate) unique constraint

-- Drop the existing deferred constraint
ALTER TABLE products DROP CONSTRAINT IF EXISTS products_sku_unique_per_tenant;

-- Create a new non-deferred unique constraint that works with ON CONFLICT
ALTER TABLE products ADD CONSTRAINT products_sku_unique_per_tenant
    UNIQUE (tenant_id, sku);

-- ==================================
-- VERIFICATION
-- ==================================
-- After this migration, the following query will work:
-- INSERT INTO products (...) VALUES (...)
-- ON CONFLICT (tenant_id, sku) DO UPDATE SET ...

COMMENT ON CONSTRAINT products_sku_unique_per_tenant ON products IS
    'Unique constraint for SKU per tenant - supports ON CONFLICT for upsert operations';
