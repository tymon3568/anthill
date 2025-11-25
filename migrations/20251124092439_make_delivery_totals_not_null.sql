-- Make total_quantity, total_value, currency_code NOT NULL in delivery_orders table
-- These fields are required for non-draft delivery orders

-- First, ensure no NULL values exist (backfill with defaults if needed)
UPDATE delivery_orders
SET total_quantity = COALESCE(total_quantity, 0),
    total_value = COALESCE(total_value, 0),
    currency_code = COALESCE(currency_code, 'VND')
WHERE total_quantity IS NULL OR total_value IS NULL OR currency_code IS NULL;

-- Alter columns to NOT NULL
ALTER TABLE delivery_orders
ALTER COLUMN total_quantity SET NOT NULL,
ALTER COLUMN total_value SET NOT NULL,
ALTER COLUMN currency_code SET NOT NULL;

-- Update comments
COMMENT ON COLUMN delivery_orders.total_quantity IS 'Total quantity of all items in the delivery (required)';
COMMENT ON COLUMN delivery_orders.total_value IS 'Total value of the delivery in smallest currency unit (required)';
COMMENT ON COLUMN delivery_orders.currency_code IS 'ISO 4217 currency code (required)';
