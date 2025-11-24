-- Make customer_id NOT NULL in delivery_orders table
-- Since delivery orders always require a customer, this field should be required

-- First, ensure no NULL values exist (should be none since we always set it)
DO $$
DECLARE
    null_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO null_count
    FROM delivery_orders
    WHERE customer_id IS NULL;

    IF null_count > 0 THEN
        RAISE EXCEPTION 'Cannot make customer_id NOT NULL: % rows have NULL customer_id', null_count;
    END IF;
END $$;

-- Alter column to NOT NULL
ALTER TABLE delivery_orders
ALTER COLUMN customer_id SET NOT NULL;

-- Update comment
COMMENT ON COLUMN delivery_orders.customer_id IS 'Customer receiving the goods (required)';
