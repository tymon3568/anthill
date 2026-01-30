-- Migration: Normalize stock_takes.status to snake_case convention
-- This ensures consistency between database, API, and frontend
-- Following the convention defined in module-implementation-workflow.md

-- ============================================
-- Step 1: Drop existing PascalCase CHECK constraint
-- ============================================
ALTER TABLE stock_takes DROP CONSTRAINT IF EXISTS stock_takes_status_check;

-- ============================================
-- Step 2: Stock Takes Status Migration
-- ============================================
-- Convert PascalCase to snake_case
UPDATE stock_takes SET status = 'draft' WHERE status = 'Draft';
UPDATE stock_takes SET status = 'scheduled' WHERE status = 'Scheduled';
UPDATE stock_takes SET status = 'in_progress' WHERE status = 'InProgress';
UPDATE stock_takes SET status = 'completed' WHERE status = 'Completed';
UPDATE stock_takes SET status = 'cancelled' WHERE status = 'Cancelled';

-- ============================================
-- Step 3: Add new CHECK constraint with snake_case values
-- ============================================
ALTER TABLE stock_takes
ADD CONSTRAINT stock_takes_status_check
CHECK (status IN ('draft', 'scheduled', 'in_progress', 'completed', 'cancelled'));

-- ============================================
-- Step 4: Comments for documentation
-- ============================================
COMMENT ON COLUMN stock_takes.status IS 'Stock take status: draft, scheduled, in_progress, completed, cancelled (snake_case)';
