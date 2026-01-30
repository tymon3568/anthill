-- Migration: Add zone/location columns to stock_transfer_items
-- Description: Add source/destination zone and location columns to enable location-level transfers
-- Dependencies: 20260128300003_unify_location_tables_drop_old.sql
-- Part of: Module 4.5 - Location Architecture Fix
-- Created: 2026-01-28

-- ==================================
-- ADD ZONE/LOCATION COLUMNS
-- ==================================

-- Add zone/location columns to stock_transfer_items
ALTER TABLE stock_transfer_items
  ADD COLUMN IF NOT EXISTS source_zone_id UUID,
  ADD COLUMN IF NOT EXISTS source_location_id UUID,
  ADD COLUMN IF NOT EXISTS destination_zone_id UUID,
  ADD COLUMN IF NOT EXISTS destination_location_id UUID;

-- ==================================
-- ADD FK CONSTRAINTS
-- ==================================

-- Source zone must belong to tenant
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_transfer_items_source_zone') THEN
        ALTER TABLE stock_transfer_items
        ADD CONSTRAINT fk_transfer_items_source_zone
        FOREIGN KEY (tenant_id, source_zone_id)
        REFERENCES warehouse_zones(tenant_id, zone_id);
    END IF;
END $$;

-- Source location must belong to tenant
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_transfer_items_source_location') THEN
        ALTER TABLE stock_transfer_items
        ADD CONSTRAINT fk_transfer_items_source_location
        FOREIGN KEY (tenant_id, source_location_id)
        REFERENCES warehouse_locations(tenant_id, location_id);
    END IF;
END $$;

-- Destination zone must belong to tenant
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_transfer_items_dest_zone') THEN
        ALTER TABLE stock_transfer_items
        ADD CONSTRAINT fk_transfer_items_dest_zone
        FOREIGN KEY (tenant_id, destination_zone_id)
        REFERENCES warehouse_zones(tenant_id, zone_id);
    END IF;
END $$;

-- Destination location must belong to tenant
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_transfer_items_dest_location') THEN
        ALTER TABLE stock_transfer_items
        ADD CONSTRAINT fk_transfer_items_dest_location
        FOREIGN KEY (tenant_id, destination_location_id)
        REFERENCES warehouse_locations(tenant_id, location_id);
    END IF;
END $$;

-- ==================================
-- ADD INDEXES FOR LOCATION-BASED QUERIES
-- ==================================

CREATE INDEX IF NOT EXISTS idx_transfer_items_source_zone
ON stock_transfer_items(tenant_id, source_zone_id)
WHERE deleted_at IS NULL AND source_zone_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_transfer_items_source_location
ON stock_transfer_items(tenant_id, source_location_id)
WHERE deleted_at IS NULL AND source_location_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_transfer_items_dest_zone
ON stock_transfer_items(tenant_id, destination_zone_id)
WHERE deleted_at IS NULL AND destination_zone_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_transfer_items_dest_location
ON stock_transfer_items(tenant_id, destination_location_id)
WHERE deleted_at IS NULL AND destination_location_id IS NOT NULL;

-- ==================================
-- ADD COMMENTS
-- ==================================

COMMENT ON COLUMN stock_transfer_items.source_zone_id IS 'Source zone within source warehouse (optional, for precise tracking)';
COMMENT ON COLUMN stock_transfer_items.source_location_id IS 'Source location/bin within source warehouse (optional)';
COMMENT ON COLUMN stock_transfer_items.destination_zone_id IS 'Destination zone within destination warehouse (optional)';
COMMENT ON COLUMN stock_transfer_items.destination_location_id IS 'Destination location/bin within destination warehouse (optional)';

-- ==================================
-- MIGRATION METADATA
-- ==================================
-- This migration enables location-level stock transfers:
-- 1. Each transfer item can specify source zone/location
-- 2. Each transfer item can specify destination zone/location
-- 3. Columns are optional for backward compatibility
-- 4. FKs ensure zone/location belong to correct tenant
