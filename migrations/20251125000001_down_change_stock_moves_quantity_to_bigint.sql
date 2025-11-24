<file_path>
anthill-windsurf/migrations/20251125000001_down_change_stock_moves_quantity_to_bigint.sql
</file_path>

<edit_description>
Create down migration to revert stock_moves.quantity from BIGINT back to INTEGER for reversibility
</edit_description>

-- Rollback Migration: Revert stock_moves.quantity from BIGINT to INTEGER
-- Description: Reverts the quantity column back to INTEGER type for rollback capability
-- Dependencies: Must be run after the up migration 20251125000001_change_stock_moves_quantity_to_bigint.sql
-- Warning: Data loss risk if any quantity values exceed 2^31 - 1 (INT_MAX)
-- Created: 2025-11-25

-- ==================================
-- ROLLBACK QUANTITY COLUMN TYPE
-- ==================================

-- Drop the updated constraint that uses BIGINT arithmetic
ALTER TABLE stock_moves DROP CONSTRAINT stock_moves_total_cost_consistency;

-- Revert quantity column from BIGINT back to INTEGER
-- WARNING: This may cause data loss if quantity values exceed INTEGER range
ALTER TABLE stock_moves ALTER COLUMN quantity TYPE INTEGER;

-- ==================================
-- RECOMPUTE TOTAL_COST FOR EXISTING ROWS
-- ==================================

-- Recompute total_cost using INTEGER arithmetic for the reverted constraint
-- This ensures existing rows comply with the CHECK constraint after rollback
UPDATE stock_moves
SET total_cost = quantity::INTEGER * unit_cost::INTEGER
WHERE unit_cost IS NOT NULL;

-- ==================================
-- RESTORE ORIGINAL CONSTRAINTS
-- ==================================

-- Recreate the original total_cost consistency constraint with INTEGER arithmetic
ALTER TABLE stock_moves ADD CONSTRAINT stock_moves_total_cost_consistency
    CHECK (total_cost IS NULL OR total_cost = quantity::INTEGER * unit_cost::INTEGER);

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This rollback migration provides reversibility for the up migration that changed
-- stock_moves.quantity from INTEGER to BIGINT.
--
-- IMPORTANT WARNINGS:
-- 1. Data Loss Risk: Any quantity values > 2^31 - 1 will be truncated to fit INTEGER
-- 2. Constraint Updates: The total_cost consistency check is reverted to INTEGER arithmetic
-- 3. Testing Required: Thoroughly test this rollback in staging before production use
--
-- Use this migration only when absolutely necessary for deployment rollback.
