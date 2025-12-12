-- Migration: Alter delivery_order_items.expiry_date from DATE to TIMESTAMPTZ
-- Description: Change expiry_date column type to support full timestamp precision for consistency with LotSerial and GoodsReceiptItems
-- Dependencies: delivery_order_items (20250110000031)
-- Created: 2025-12-16

-- Alter the expiry_date column from DATE to TIMESTAMPTZ
ALTER TABLE delivery_order_items
ALTER COLUMN expiry_date TYPE TIMESTAMPTZ;

-- Update the index to use TIMESTAMPTZ (though the index name stays the same)
-- The existing index should automatically adapt to the new type

-- Update comments
COMMENT ON COLUMN delivery_order_items.expiry_date IS 'Expiry timestamp for perishable items (full precision)';
