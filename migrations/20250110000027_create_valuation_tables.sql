-- Migration: Create inventory valuation tables
-- Description: Creates tables for inventory valuation system supporting FIFO, AVCO, and Standard costing methods
-- Dependencies: stock_moves table, products table
-- Created: 2025-11-15

-- ==================================
-- INVENTORY_VALUATION_LAYERS TABLE
-- ==================================
-- Tracks cost layers for FIFO valuation method.
-- Each layer represents a cost level for remaining inventory quantity.

CREATE TABLE inventory_valuation_layers (
    -- Primary key using UUID v7
    layer_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Product relationship
    product_id UUID NOT NULL REFERENCES products(product_id),

    -- Layer details
    quantity BIGINT NOT NULL CHECK (quantity >= 0),  -- Remaining quantity in this layer (zero once consumed)
    unit_cost BIGINT NOT NULL CHECK (unit_cost >= 0),  -- Cost per unit in cents
    total_value BIGINT NOT NULL,  -- Calculated total

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints

    -- Foreign keys
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- INVENTORY_VALUATIONS TABLE
-- ==================================
-- Stores current valuation information for each product.
-- Supports multiple valuation methods with their respective cost calculations.

CREATE TABLE inventory_valuations (
    -- Primary key using UUID v7
    valuation_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy and product
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    product_id UUID NOT NULL REFERENCES products(product_id),

    -- Valuation method
    valuation_method VARCHAR(20) NOT NULL
        CHECK (valuation_method IN ('fifo', 'avco', 'standard')),

    -- Current valuation data
    current_unit_cost BIGINT CHECK (current_unit_cost >= 0),  -- For FIFO/AVCO
    total_quantity BIGINT NOT NULL DEFAULT 0 CHECK (total_quantity >= 0),
    total_value BIGINT NOT NULL DEFAULT 0 CHECK (total_value >= 0),

    -- Standard cost (only used when method = 'standard')
    standard_cost BIGINT CHECK (standard_cost >= 0),

    -- Metadata
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,  -- User who last updated (optional)

    -- Constraints
    UNIQUE (tenant_id, product_id),  -- One valuation per product per tenant

    -- Foreign keys
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id)
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (updated_by) REFERENCES users(user_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- INVENTORY_VALUATION_HISTORY TABLE
-- ==================================
-- Audit trail for valuation changes.
-- Tracks historical valuations for reporting and compliance.

CREATE TABLE inventory_valuation_history (
    -- Primary key
    history_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Reference to valuation
    valuation_id UUID NOT NULL REFERENCES inventory_valuations(valuation_id),

    -- Historical data snapshot
    tenant_id UUID NOT NULL,
    product_id UUID NOT NULL,
    valuation_method VARCHAR(20) NOT NULL,
    unit_cost BIGINT,
    total_quantity BIGINT NOT NULL,
    total_value BIGINT NOT NULL,
    standard_cost BIGINT,

    -- Change metadata
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by UUID,  -- User who made the change
    change_reason TEXT,  -- Reason for change (e.g., 'stock_move', 'manual_adjustment')

    -- Foreign keys
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id)
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (changed_by) REFERENCES users(user_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- INDEXES
-- ==================================

-- Valuation layers indexes
CREATE INDEX idx_valuation_layers_tenant_product_created
    ON inventory_valuation_layers(tenant_id, product_id, created_at);

CREATE INDEX idx_valuation_layers_tenant_product
    ON inventory_valuation_layers(tenant_id, product_id);

-- Valuations indexes
CREATE INDEX idx_valuations_tenant_product
    ON inventory_valuations(tenant_id, product_id);

CREATE INDEX idx_valuations_method
    ON inventory_valuations(valuation_method);

-- History indexes
CREATE INDEX idx_valuation_history_valuation_changed
    ON inventory_valuation_history(valuation_id, changed_at DESC);

CREATE INDEX idx_valuation_history_tenant_product_changed
    ON inventory_valuation_history(tenant_id, product_id, changed_at DESC);

-- ==================================
-- TRIGGERS
-- ==================================

-- Update updated_at on valuation layers
CREATE OR REPLACE FUNCTION update_valuation_layers_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_valuation_layers_updated_at_trigger
    BEFORE UPDATE ON inventory_valuation_layers
    FOR EACH ROW
    EXECUTE FUNCTION update_valuation_layers_updated_at();

-- Update last_updated on valuations
CREATE OR REPLACE FUNCTION update_valuations_last_updated()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_valuations_last_updated_trigger
    BEFORE UPDATE ON inventory_valuations
    FOR EACH ROW
    EXECUTE FUNCTION update_valuations_last_updated();

-- Insert history on valuation changes
CREATE OR REPLACE FUNCTION log_valuation_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN
        INSERT INTO inventory_valuation_history (
            valuation_id, tenant_id, product_id, valuation_method,
            unit_cost, total_quantity, total_value, standard_cost,
            changed_by, change_reason
        ) VALUES (
            NEW.valuation_id, NEW.tenant_id, NEW.product_id, NEW.valuation_method,
            NEW.current_unit_cost, NEW.total_quantity, NEW.total_value, NEW.standard_cost,
            NEW.updated_by, 'valuation_update'
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER log_valuation_changes_trigger
    AFTER UPDATE ON inventory_valuations
    FOR EACH ROW
    EXECUTE FUNCTION log_valuation_changes();

-- ==================================
-- COMMENTS
-- ==================================

COMMENT ON TABLE inventory_valuation_layers IS 'Cost layers for FIFO valuation tracking';
COMMENT ON TABLE inventory_valuations IS 'Current inventory valuations by product and method';
COMMENT ON TABLE inventory_valuation_history IS 'Audit trail of valuation changes';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration establishes the foundation for:
-- 1. FIFO cost layer management
-- 2. Multiple valuation method support
-- 3. Historical valuation tracking
-- 4. Audit compliance for financial reporting
