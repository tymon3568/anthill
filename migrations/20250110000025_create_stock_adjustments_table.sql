-- Migration: Create stock_adjustments table
-- Description: Creates the stock_adjustments table to record reasons for manual stock adjustments
-- Dependencies: stock_moves table, warehouse_locations table, products table, users table, and UNIQUE constraints from 20250110000026
-- Created: 2025-10-29

-- ==================================
-- STOCK_ADJUSTMENTS TABLE
-- ==================================
-- This table records the business reasons for manual stock adjustments.
-- Each adjustment corresponds to a stock_move record for audit trail.

CREATE TABLE stock_adjustments (
    -- Primary key using UUID v7
    adjustment_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Reference to the corresponding stock move
    move_id UUID NOT NULL,

    -- Product and location context
    product_id UUID NOT NULL,
    warehouse_id UUID NOT NULL,

    -- Adjustment details
    reason_code VARCHAR(50) NOT NULL,  -- e.g., 'damaged', 'lost', 'count_error', 'promotion'
    notes TEXT,                        -- Detailed explanation

    -- Approval workflow
    approved_by UUID,  -- User who approved the adjustment

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT stock_adjustments_unique_move
        UNIQUE (tenant_id, move_id),  -- One adjustment per move
    CONSTRAINT stock_adjustments_reason_not_empty
        CHECK (trim(reason_code) != ''),
    CONSTRAINT stock_adjustments_soft_delete_check
        CHECK (deleted_at IS NULL OR deleted_at >= updated_at),
    CONSTRAINT stock_adjustments_tenant_move_fk
        FOREIGN KEY (tenant_id, move_id)
        REFERENCES stock_moves (tenant_id, move_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_adjustments_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_adjustments_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouse_locations (tenant_id, location_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT stock_adjustments_tenant_user_fk
        FOREIGN KEY (tenant_id, approved_by)
        REFERENCES users (tenant_id, user_id)
        DEFERRABLE INITIALLY DEFERRED
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Primary lookup indexes
CREATE INDEX idx_stock_adjustments_tenant_move
    ON stock_adjustments(tenant_id, move_id)
    WHERE deleted_at IS NULL;

-- Business query indexes
CREATE INDEX idx_stock_adjustments_tenant_product
    ON stock_adjustments(tenant_id, product_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_adjustments_tenant_warehouse
    ON stock_adjustments(tenant_id, warehouse_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_adjustments_tenant_reason
    ON stock_adjustments(tenant_id, reason_code, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_stock_adjustments_tenant_approved_by
    ON stock_adjustments(tenant_id, approved_by, created_at DESC)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_stock_adjustments_updated_at_trigger
    BEFORE UPDATE ON stock_adjustments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE stock_adjustments IS 'Records business reasons for manual stock adjustments';
COMMENT ON COLUMN stock_adjustments.adjustment_id IS 'UUID v7 primary key';
COMMENT ON COLUMN stock_adjustments.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN stock_adjustments.move_id IS 'Foreign key to stock_moves table';
COMMENT ON COLUMN stock_adjustments.product_id IS 'Product being adjusted';
COMMENT ON COLUMN stock_adjustments.warehouse_id IS 'Warehouse location of adjustment';
COMMENT ON COLUMN stock_adjustments.reason_code IS 'Reason code for adjustment (damaged/lost/count_error/etc)';
COMMENT ON COLUMN stock_adjustments.notes IS 'Detailed notes about the adjustment';
COMMENT ON COLUMN stock_adjustments.approved_by IS 'User who approved the adjustment';
COMMENT ON COLUMN stock_adjustments.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN stock_adjustments.updated_at IS 'Last update timestamp';
COMMENT ON COLUMN stock_adjustments.deleted_at IS 'Soft delete timestamp';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration supports:
-- 1. Audit trail for manual adjustments
-- 2. Approval workflow tracking
-- 3. Reporting on adjustment reasons
-- 4. Multi-tenant isolation with composite FKs
-- 5. Soft delete for compliance
