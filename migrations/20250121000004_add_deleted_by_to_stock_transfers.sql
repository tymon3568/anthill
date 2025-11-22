-- Migration: Add deleted_by to stock_transfers table
-- Description: Adds deleted_by column to stock_transfers for audit trail consistency
-- Dependencies: stock_transfers table (20250121000001)
-- Created: 2025-11-22

-- Add deleted_by column for audit trail
ALTER TABLE stock_transfers
ADD COLUMN deleted_by UUID;

-- Add comment for documentation
COMMENT ON COLUMN stock_transfers.deleted_by IS 'User ID who deleted the transfer';

-- Migration metadata
-- This migration adds audit trail consistency by tracking who deleted transfers
-- The column is nullable to maintain backward compatibility
