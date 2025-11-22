-- Migration: Add deleted_by to stock_transfer_items table
-- Description: Adds deleted_by column to stock_transfer_items for audit trail consistency
-- Dependencies: stock_transfer_items table (20250121000002)
-- Created: 2025-11-22

-- Add deleted_by column for audit trail
ALTER TABLE stock_transfer_items
ADD COLUMN deleted_by UUID;

-- Add comment for documentation
COMMENT ON COLUMN stock_transfer_items.deleted_by IS 'User ID who deleted the transfer item';

-- Migration metadata
-- This migration adds audit trail consistency by tracking who deleted transfer items
-- The column is nullable to maintain backward compatibility
