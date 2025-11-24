-- Migration: Change stock_moves.quantity from INTEGER to BIGINT
-- Description: Changes the quantity column from INTEGER to BIGINT to support larger values and prevent truncation
-- Dependencies: stock_moves table must exist
-- Created: 2025-11-25

-- ==================================
-- CHANGE QUANTITY COLUMN TYPE
-- ==================================

-- Change quantity column from INTEGER to BIGINT
ALTER TABLE stock_moves ALTER COLUMN quantity TYPE BIGINT;

-- ==================================
-- UPDATE CONSTRAINTS
-- ==================================

-- Update the total_cost consistency constraint to use BIGINT arithmetic
-- Drop the old constraint
ALTER TABLE stock_moves DROP CONSTRAINT stock_moves_total_cost_consistency;

-- Add the updated constraint with NOT VALID to avoid immediate validation
-- This prevents full-table scan and potential locking during migration
ALTER TABLE stock_moves ADD CONSTRAINT stock_moves_total_cost_consistency
    CHECK (total_cost IS NULL OR total_cost = quantity::BIGINT * unit_cost::BIGINT) NOT VALID;

-- ==================================
-- RECOMPUTE TOTAL_COST FOR EXISTING ROWS
-- ==================================

-- Recompute total_cost using BIGINT arithmetic to match the new constraint
-- This ensures existing rows comply with the CHECK constraint
UPDATE stock_moves
SET total_cost = quantity::BIGINT * unit_cost::BIGINT
WHERE unit_cost IS NOT NULL;

-- ==================================
-- VALIDATE CONSTRAINT
-- ==================================

-- Validate the constraint after updating existing rows
-- This performs validation in a separate step to avoid blocking other operations
ALTER TABLE stock_moves VALIDATE CONSTRAINT stock_moves_total_cost_consistency;

-- ==================================
-- UPDATE TRIGGER
-- ==================================

-- The calculate_stock_move_total_cost function already uses BIGINT casting, so no changes needed

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration addresses the issue where large quantity values (> 2^31 - 1)
-- would be truncated when cast from i64 to i32 in application code.
--
-- By changing the database column to BIGINT, we ensure consistency between
-- the application domain (i64) and the database schema (BIGINT).
--
-- The total_cost calculation is updated to ensure proper BIGINT arithmetic.
