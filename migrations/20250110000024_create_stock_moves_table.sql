-- Migration: Create stock_moves table (Stock Ledger)
-- Description: Creates the immutable stock_moves table for complete audit trail of inventory movements
-- Dependencies: products table, warehouse_locations table
-- Created: 2025-10-29

-- ==================================
-- STOCK_MOVES TABLE (Stock Ledger)
-- ==================================
-- This table serves as the immutable stock ledger - the single source of truth
-- for all inventory movements in the multi-tenant system.
--
-- IMPORTANT: This table is IMMUTABLE. Application code should NEVER perform
-- UPDATE operations on stock_moves records. Only INSERT operations are allowed.

CREATE TABLE stock_moves (
    -- Primary key using UUID v7 (timestamp-based for better index locality)
    move_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Product relationship
    product_id UUID NOT NULL,

    -- Location relationships (nullable for certain move types)
    source_location_id UUID,      -- NULL for receipts (goods coming from outside)
    destination_location_id UUID, -- NULL for deliveries (goods going outside)

    -- Movement classification
    move_type VARCHAR(20) NOT NULL
        CHECK (move_type IN ('receipt', 'delivery', 'transfer', 'adjustment', 'production', 'consumption')),

    -- Quantity and valuation
    quantity INTEGER NOT NULL,  -- Can be negative for deliveries/consumption
    unit_cost BIGINT,           -- Cost per unit in smallest currency unit (cents/xu)
    total_cost BIGINT,          -- Calculated: quantity * unit_cost

    -- Reference to originating document
    reference_type VARCHAR(20) NOT NULL
        CHECK (reference_type IN ('grn', 'do', 'transfer', 'adjustment', 'production', 'consumption', 'manual')),
    reference_id UUID NOT NULL,  -- Foreign key to the originating document table

    -- Idempotency and deduplication
    idempotency_key VARCHAR(255) NOT NULL,  -- Unique key to prevent duplicate moves

    -- Movement timestamp (when the physical movement occurred)
    move_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Movement metadata
    move_reason TEXT,           -- Human-readable reason for the movement
    batch_info JSONB,           -- Lot/serial/batch information if applicable
    metadata JSONB,             -- Additional movement-specific data

    -- Audit fields (immutable table, so created_at only)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT stock_moves_quantity_not_zero
        CHECK (quantity != 0),
    CONSTRAINT stock_moves_positive_unit_cost
        CHECK (unit_cost IS NULL OR unit_cost >= 0),
    CONSTRAINT stock_moves_positive_total_cost
        CHECK (total_cost IS NULL OR total_cost >= 0),
    CONSTRAINT stock_moves_idempotency_unique_per_tenant
        UNIQUE (tenant_id, idempotency_key) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_moves_product_fk
        FOREIGN KEY (product_id)
        REFERENCES products (product_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_moves_source_location_fk
        FOREIGN KEY (source_location_id)
        REFERENCES warehouse_locations (location_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_moves_destination_location_fk
        FOREIGN KEY (destination_location_id)
        REFERENCES warehouse_locations (location_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_moves_locations_different
        CHECK (source_location_id IS NULL OR destination_location_id IS NULL OR
               source_location_id != destination_location_id),
    CONSTRAINT stock_moves_transfer_has_both_locations
        CHECK (move_type != 'transfer' OR
               (source_location_id IS NOT NULL AND destination_location_id IS NOT NULL)),
    CONSTRAINT stock_moves_receipt_no_source
        CHECK (move_type != 'receipt' OR source_location_id IS NULL),
    CONSTRAINT stock_moves_delivery_no_destination
        CHECK (move_type != 'delivery' OR destination_location_id IS NULL)
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for stock ledger queries

-- Primary lookup indexes (most frequently used)
CREATE INDEX idx_stock_moves_tenant_product_date
    ON stock_moves(tenant_id, product_id, move_date DESC);

CREATE INDEX idx_stock_moves_tenant_reference
    ON stock_moves(tenant_id, reference_type, reference_id);

-- Location-based indexes for inventory calculations
CREATE INDEX idx_stock_moves_tenant_source_location
    ON stock_moves(tenant_id, source_location_id, move_date DESC)
    WHERE source_location_id IS NOT NULL;

CREATE INDEX idx_stock_moves_tenant_destination_location
    ON stock_moves(tenant_id, destination_location_id, move_date DESC)
    WHERE destination_location_id IS NOT NULL;

-- Move type and date indexes for reporting
CREATE INDEX idx_stock_moves_tenant_type_date
    ON stock_moves(tenant_id, move_type, move_date DESC);

-- Idempotency index (enforced by unique constraint above)
CREATE INDEX idx_stock_moves_tenant_idempotency
    ON stock_moves(tenant_id, idempotency_key);

-- Composite indexes for complex queries
CREATE INDEX idx_stock_moves_tenant_product_location_date
    ON stock_moves(tenant_id, product_id, source_location_id, destination_location_id, move_date DESC);

-- ==================================
-- PARTIAL INDEXES for Active Records
-- ==================================
-- Note: stock_moves is append-only, no soft delete needed

-- ==================================
-- TRIGGERS
-- ==================================

-- Trigger to auto-calculate total_cost
CREATE OR REPLACE FUNCTION calculate_stock_move_total_cost()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate total_cost as quantity * unit_cost
    IF NEW.unit_cost IS NOT NULL THEN
        NEW.total_cost := NEW.quantity * NEW.unit_cost;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER calculate_stock_move_total_cost_trigger
    BEFORE INSERT ON stock_moves
    FOR EACH ROW
    EXECUTE FUNCTION calculate_stock_move_total_cost();

-- Prevent updates on immutable table (except for allowed fields)
CREATE OR REPLACE FUNCTION prevent_stock_moves_updates()
RETURNS TRIGGER AS $$
BEGIN
    -- Allow updates only to metadata and move_reason (for corrections)
    -- But prevent changes to core movement data
    IF OLD.move_id != NEW.move_id OR
       OLD.tenant_id != NEW.tenant_id OR
       OLD.product_id != NEW.product_id OR
       OLD.source_location_id IS DISTINCT FROM NEW.source_location_id OR
       OLD.destination_location_id IS DISTINCT FROM NEW.destination_location_id OR
       OLD.move_type != NEW.move_type OR
       OLD.quantity != NEW.quantity OR
       OLD.unit_cost IS DISTINCT FROM NEW.unit_cost OR
       OLD.reference_type != NEW.reference_type OR
       OLD.reference_id != NEW.reference_id OR
       OLD.idempotency_key != NEW.idempotency_key OR
       OLD.move_date != NEW.move_date THEN
        RAISE EXCEPTION 'Stock moves are immutable. Core movement data cannot be modified.';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_stock_moves_updates_trigger
    BEFORE UPDATE ON stock_moves
    FOR EACH ROW
    EXECUTE FUNCTION prevent_stock_moves_updates();

-- ==================================
-- ROW LEVEL SECURITY (Future)
-- ==================================
-- Note: We use application-level filtering instead of RLS
-- All queries must include: WHERE tenant_id = $1

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE stock_moves IS 'Immutable stock ledger - Complete audit trail of all inventory movements';
COMMENT ON COLUMN stock_moves.move_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN stock_moves.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN stock_moves.product_id IS 'Reference to products table';
COMMENT ON COLUMN stock_moves.source_location_id IS 'Source warehouse location (NULL for receipts)';
COMMENT ON COLUMN stock_moves.destination_location_id IS 'Destination warehouse location (NULL for deliveries)';
COMMENT ON COLUMN stock_moves.move_type IS 'Movement type: receipt/delivery/transfer/adjustment/production/consumption';
COMMENT ON COLUMN stock_moves.quantity IS 'Movement quantity (positive for additions, negative for reductions)';
COMMENT ON COLUMN stock_moves.unit_cost IS 'Cost per unit in smallest currency unit (cents/xu)';
COMMENT ON COLUMN stock_moves.total_cost IS 'Calculated total cost (quantity * unit_cost)';
COMMENT ON COLUMN stock_moves.reference_type IS 'Type of originating document: grn/do/transfer/adjustment/production/consumption/manual';
COMMENT ON COLUMN stock_moves.reference_id IS 'Foreign key to originating document table';
COMMENT ON COLUMN stock_moves.idempotency_key IS 'Unique key to prevent duplicate movements';
COMMENT ON COLUMN stock_moves.move_date IS 'Timestamp when the physical movement occurred';
COMMENT ON COLUMN stock_moves.move_reason IS 'Human-readable reason for the movement';
COMMENT ON COLUMN stock_moves.batch_info IS 'Lot/serial/batch tracking information JSON';
COMMENT ON COLUMN stock_moves.metadata IS 'Additional movement-specific data JSON';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Complete inventory audit trail
-- 2. Stock valuation calculations
-- 3. Inventory reconciliation
-- 4. Movement history reporting
-- 5. Idempotent operations (prevent duplicates)

-- Key design decisions:
-- - Immutable: No UPDATE operations allowed on core data
-- - Idempotency: Prevents duplicate movements via unique constraint
-- - Flexible locations: Source/destination can be NULL for external movements
-- - Comprehensive indexing: Optimized for common inventory queries
-- - Cost tracking: Built-in valuation support

-- Next features will include:
-- - Inventory levels calculation triggers
-- - Stock valuation layer updates
-- - Movement validation rules
-- - Bulk movement operations
-- - Movement reversal capabilities
