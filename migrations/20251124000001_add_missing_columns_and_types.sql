-- Migration: Add missing columns and types for stock take functionality
-- Description: Adds deleted_by columns to stock_takes and stock_take_lines tables, and creates stock_take_status enum type
-- Dependencies: stock_takes and stock_take_lines tables must exist
-- Created: 2025-11-24

-- ==================================
-- CREATE STOCK TAKE STATUS ENUM TYPE
-- ==================================

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'stock_take_status') THEN
        CREATE TYPE stock_take_status AS ENUM (
            'draft',
            'scheduled',
            'in_progress',
            'completed',
            'cancelled'
        );
    END IF;
END $$;

-- ==================================
-- ADD DELETED_BY COLUMN TO STOCK_TAKES
-- ==================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'stock_takes'
        AND column_name = 'deleted_by'
        AND table_schema = 'public'
    ) THEN
        ALTER TABLE stock_takes ADD COLUMN deleted_by UUID;

        -- Add foreign key constraint
        ALTER TABLE stock_takes
        ADD CONSTRAINT stock_takes_tenant_deleted_by_fk
        FOREIGN KEY (tenant_id, deleted_by)
        REFERENCES users (tenant_id, user_id);

        -- Add comment
        COMMENT ON COLUMN stock_takes.deleted_by IS 'User ID who deleted the stock take';
    END IF;
END $$;

-- ==================================
-- ADD DELETED_BY COLUMN TO STOCK_TAKE_LINES
-- ==================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'stock_take_lines'
        AND column_name = 'deleted_by'
        AND table_schema = 'public'
    ) THEN
        ALTER TABLE stock_take_lines ADD COLUMN deleted_by UUID;

        -- Add foreign key constraint
        ALTER TABLE stock_take_lines
        ADD CONSTRAINT stock_take_lines_tenant_deleted_by_fk
        FOREIGN KEY (tenant_id, deleted_by)
        REFERENCES users (tenant_id, user_id);

        -- Add comment
        COMMENT ON COLUMN stock_take_lines.deleted_by IS 'User ID who deleted the stock take line';
    END IF;
END $$;

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration adds:
-- 1. stock_take_status enum type for type safety
-- 2. deleted_by columns for audit trails
-- 3. Proper foreign key constraints

-- Next: Update existing code to use these new columns/types
