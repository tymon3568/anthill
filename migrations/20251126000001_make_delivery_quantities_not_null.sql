-- Migration: Make delivery quantities NOT NULL
-- Description: Ensures picked_quantity and delivered_quantity are NOT NULL with default 0
-- Dependencies: delivery_order_items table must exist
-- Created: 2025-11-26

-- ==================================
-- MAKE QUANTITIES NOT NULL
-- ==================================

-- Make picked_quantity NOT NULL (already has DEFAULT 0)
ALTER TABLE delivery_order_items ALTER COLUMN picked_quantity SET NOT NULL;

-- Make delivered_quantity NOT NULL (already has DEFAULT 0)
ALTER TABLE delivery_order_items ALTER COLUMN delivered_quantity SET NOT NULL;

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration ensures data integrity by making quantity fields NOT NULL.
-- Since they have DEFAULT 0, existing rows are unaffected.
-- This aligns the database schema with the application domain model (i64 not Option<i64>).
