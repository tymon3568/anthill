-- Migration: Relax stock_transfers_receive_dates constraint
--
-- Problem: The current constraint requires actual_receive_date >= expected_receive_date,
-- which prevents receiving transfers earlier than expected. This is too restrictive
-- for real-world scenarios where shipments may arrive early.
--
-- Solution: Remove the constraint as it doesn't provide meaningful validation.
-- Business logic should allow receiving transfers at any time after they are shipped.

-- Drop the overly restrictive constraint
ALTER TABLE stock_transfers DROP CONSTRAINT IF EXISTS stock_transfers_receive_dates;

-- Also check and drop the similar constraint for ship dates if it exists
ALTER TABLE stock_transfers DROP CONSTRAINT IF EXISTS stock_transfers_ship_dates;
