-- Migration: Add advanced indexes for performance optimization
-- Description: Implements advanced indexing strategies including partial indexes and optimized composites
-- Dependencies: Existing tables (products, stock_moves, inventory_levels)
-- Created: 2025-12-11

-- ==================================
-- ADVANCED INDEXES for PRODUCTS TABLE
-- ==================================

-- Partial index for active products only (faster queries for active catalog)
CREATE INDEX idx_products_tenant_active_sku
    ON products(tenant_id, sku)
    WHERE deleted_at IS NULL AND is_active = true;

-- Partial index for sellable products
CREATE INDEX idx_products_tenant_sellable
    ON products(tenant_id, product_id, sale_price)
    WHERE deleted_at IS NULL AND is_active = true AND is_sellable = true;

-- Composite index for product search with filters
CREATE INDEX idx_products_tenant_type_active
    ON products(tenant_id, product_type, is_active)
    WHERE deleted_at IS NULL AND is_active = true;

-- ==================================
-- ADVANCED INDEXES for STOCK_MOVES TABLE
-- ==================================

-- Partial index for receipts (common query pattern)
CREATE INDEX idx_stock_moves_tenant_receipts
    ON stock_moves(tenant_id, product_id, move_date DESC)
    WHERE move_type = 'receipt';

-- Partial index for deliveries
CREATE INDEX idx_stock_moves_tenant_deliveries
    ON stock_moves(tenant_id, product_id, move_date DESC)
    WHERE move_type = 'delivery';

-- Partial index for transfers
CREATE INDEX idx_stock_moves_tenant_transfers
    ON stock_moves(tenant_id, source_location_id, destination_location_id, move_date DESC)
    WHERE move_type = 'transfer';

-- Composite index for valuation queries
CREATE INDEX idx_stock_moves_tenant_product_cost_date
    ON stock_moves(tenant_id, product_id, unit_cost, move_date DESC)
    WHERE unit_cost IS NOT NULL;

-- ==================================
-- ADVANCED INDEXES for INVENTORY_LEVELS TABLE
-- ==================================

-- Partial index for products with available stock
CREATE INDEX idx_inventory_levels_tenant_available_stock
    ON inventory_levels(tenant_id, product_id, available_quantity DESC)
    WHERE deleted_at IS NULL AND available_quantity > 0;

-- Index for products with reservations
CREATE INDEX idx_inventory_levels_tenant_reserved
    ON inventory_levels(tenant_id, product_id, reserved_quantity DESC)
    WHERE deleted_at IS NULL AND reserved_quantity > 0;

-- ==================================
-- ADDITIONAL INDEXES for OTHER TABLES (if needed)
-- ==================================

-- For warehouse_locations (assuming exists)
-- CREATE INDEX CONCURRENTLY idx_warehouse_locations_tenant_active
--     ON warehouse_locations(tenant_id, location_id)
--     WHERE deleted_at IS NULL AND is_active = true;

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- These indexes optimize for:
-- 1. Active product catalog queries
-- 2. Stock movement analysis by type
-- 3. Inventory level checks for available stock
-- 4. Valuation and costing queries
-- 5. Low stock and reservation monitoring

-- Note: Used CREATE INDEX CONCURRENTLY for production safety (if needed)
-- But for initial migration, standard CREATE INDEX is fine
