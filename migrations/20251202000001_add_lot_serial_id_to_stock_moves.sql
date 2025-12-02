-- Migration: Add lot_serial_id column to stock_moves table for lot/serial tracking
-- This enables efficient querying of stock moves by lot_serial_id without relying on JSON batch_info

-- Add the column (nullable initially)
ALTER TABLE stock_moves ADD COLUMN lot_serial_id UUID;

-- Add composite index for tenant isolation and lot_serial_id queries
CREATE INDEX idx_stock_moves_tenant_lot_serial_id ON stock_moves (tenant_id, lot_serial_id);

-- Optional: Add comment for documentation
COMMENT ON COLUMN stock_moves.lot_serial_id IS 'References lots_serial_numbers.lot_serial_id for lot-tracked products';
