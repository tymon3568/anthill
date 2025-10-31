-- Migration: Fix casbin_rule v4 and v5 to NOT NULL
-- Description: sqlx-adapter requires v4 and v5 to be NOT NULL
-- Author: System
-- Date: 2025-01-10
-- Reference: https://github.com/casbin-rs/sqlx-adapter

-- First, update any NULL values to empty string
UPDATE casbin_rule SET v4 = '' WHERE v4 IS NULL;
UPDATE casbin_rule SET v5 = '' WHERE v5 IS NULL;

-- Then add NOT NULL constraint
ALTER TABLE casbin_rule
    ALTER COLUMN v4 SET NOT NULL,
    ALTER COLUMN v4 SET DEFAULT '',
    ALTER COLUMN v5 SET NOT NULL,
    ALTER COLUMN v5 SET DEFAULT '';

COMMENT ON COLUMN casbin_rule.v4 IS 'Additional field 1 (must be NOT NULL for sqlx-adapter compatibility)';
COMMENT ON COLUMN casbin_rule.v5 IS 'Additional field 2 (must be NOT NULL for sqlx-adapter compatibility)';
