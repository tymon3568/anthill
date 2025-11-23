-- Migration: Add unique constraint to stock_takes for composite FK
-- Description: Adds a unique constraint on (tenant_id, stock_take_id) to support foreign key from stock_take_lines
-- Dependencies: stock_takes table (20250121000006)
-- Created: 2025-11-23

-- ==================================
-- ADD UNIQUE CONSTRAINT TO STOCK_TAKES
-- ==================================
-- This migration adds a unique constraint on (tenant_id, stock_take_id) to the stock_takes table.
-- This is required for the composite foreign key in the stock_take_lines table.

ALTER TABLE stock_takes
ADD CONSTRAINT stock_takes_tenant_id_unique
UNIQUE (tenant_id, stock_take_id)
DEFERRABLE INITIALLY DEFERRED;

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON CONSTRAINT stock_takes_tenant_id_unique ON stock_takes IS 'Unique constraint on (tenant_id, stock_take_id) for composite FK from stock_take_lines';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration ensures data integrity for multi-tenant relationships.
-- The DEFERRABLE INITIALLY DEFERRED allows for transaction-level constraint checking.
