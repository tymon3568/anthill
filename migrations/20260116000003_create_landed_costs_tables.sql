-- Migration: Create landed costs tables
-- Description: Creates tables for landed cost allocation to GRN receipts
-- Dependencies: goods_receipts (20250110000028), goods_receipt_items (20250110000029), valuation tables (20250110000027)
-- Task: task_04.06.02_implement_landed_costs.md
-- Created: 2026-01-16

-- ==================================
-- LANDED_COSTS TABLE
-- ==================================
-- Master table for landed cost documents that allocate additional costs
-- (freight, customs, handling, insurance) to goods receipts

CREATE TABLE landed_costs (
    -- Multi-tenancy: composite PK
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    landed_cost_id UUID NOT NULL DEFAULT uuid_generate_v7(),

    -- Human-readable reference
    reference TEXT,

    -- Status workflow: draft -> posted | cancelled
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'posted', 'cancelled')),

    -- Link to goods receipt (optional, can also link via allocations)
    grn_id UUID,

    -- Posting metadata
    posted_at TIMESTAMPTZ,
    posted_by UUID,

    -- Notes/description
    notes TEXT,

    -- Audit fields
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Composite primary key for multi-tenancy
    PRIMARY KEY (tenant_id, landed_cost_id),

    -- Composite FK to goods_receipts
    CONSTRAINT landed_costs_grn_fk
        FOREIGN KEY (tenant_id, grn_id)
        REFERENCES goods_receipts (tenant_id, receipt_id)
        ON DELETE RESTRICT,

    -- Composite FK to users for created_by
    CONSTRAINT landed_costs_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id)
        ON DELETE RESTRICT,

    -- Composite FK to users for posted_by
    CONSTRAINT landed_costs_posted_by_fk
        FOREIGN KEY (tenant_id, posted_by)
        REFERENCES users (tenant_id, user_id)
        ON DELETE RESTRICT
);

-- Indexes for landed_costs
CREATE INDEX idx_landed_costs_tenant_status
    ON landed_costs(tenant_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_landed_costs_tenant_grn
    ON landed_costs(tenant_id, grn_id)
    WHERE deleted_at IS NULL AND grn_id IS NOT NULL;

CREATE INDEX idx_landed_costs_tenant_created_at
    ON landed_costs(tenant_id, created_at DESC)
    WHERE deleted_at IS NULL;

-- Unique reference per tenant (partial index for active records)
CREATE UNIQUE INDEX idx_landed_costs_unique_reference
    ON landed_costs(tenant_id, reference)
    WHERE deleted_at IS NULL AND reference IS NOT NULL;

-- ==================================
-- LANDED_COST_LINES TABLE
-- ==================================
-- Individual cost lines within a landed cost document
-- Each line represents a specific cost type (freight, customs, etc.)

CREATE TABLE landed_cost_lines (
    -- Multi-tenancy: composite PK
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    landed_cost_line_id UUID NOT NULL DEFAULT uuid_generate_v7(),

    -- Parent landed cost
    landed_cost_id UUID NOT NULL,

    -- Cost type: freight, customs, handling, insurance, other
    cost_type TEXT NOT NULL
        CHECK (cost_type IN ('freight', 'customs', 'handling', 'insurance', 'other')),

    -- Cost description (e.g., "Ocean freight from Shanghai")
    description TEXT,

    -- Amount in cents (BIGINT for money)
    amount_cents BIGINT NOT NULL
        CHECK (amount_cents >= 0),

    -- Allocation method: by_value (MVP), future: by_quantity, by_weight, by_volume
    allocation_method TEXT NOT NULL DEFAULT 'by_value'
        CHECK (allocation_method IN ('by_value', 'by_quantity', 'by_weight', 'by_volume')),

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Composite primary key
    PRIMARY KEY (tenant_id, landed_cost_line_id),

    -- Composite FK to parent landed_costs
    CONSTRAINT landed_cost_lines_parent_fk
        FOREIGN KEY (tenant_id, landed_cost_id)
        REFERENCES landed_costs (tenant_id, landed_cost_id)
        ON DELETE CASCADE
);

-- Indexes for landed_cost_lines
CREATE INDEX idx_landed_cost_lines_tenant_parent
    ON landed_cost_lines(tenant_id, landed_cost_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_landed_cost_lines_tenant_cost_type
    ON landed_cost_lines(tenant_id, cost_type)
    WHERE deleted_at IS NULL;

-- ==================================
-- LANDED_COST_ALLOCATIONS TABLE
-- ==================================
-- Computed allocation results mapping cost lines to target items
-- Targets can be GRN items or stock moves

CREATE TABLE landed_cost_allocations (
    -- Multi-tenancy: composite PK
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    landed_cost_allocation_id UUID NOT NULL DEFAULT uuid_generate_v7(),

    -- Parent landed cost
    landed_cost_id UUID NOT NULL,

    -- Parent cost line
    landed_cost_line_id UUID NOT NULL,

    -- Target type and ID (polymorphic reference)
    target_type TEXT NOT NULL
        CHECK (target_type IN ('grn_item', 'stock_move')),
    target_id UUID NOT NULL,

    -- Allocated amount in cents
    allocated_amount_cents BIGINT NOT NULL
        CHECK (allocated_amount_cents >= 0),

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Composite primary key
    PRIMARY KEY (tenant_id, landed_cost_allocation_id),

    -- Composite FK to parent landed_costs
    CONSTRAINT landed_cost_allocations_parent_fk
        FOREIGN KEY (tenant_id, landed_cost_id)
        REFERENCES landed_costs (tenant_id, landed_cost_id)
        ON DELETE CASCADE,

    -- Composite FK to cost line
    CONSTRAINT landed_cost_allocations_line_fk
        FOREIGN KEY (tenant_id, landed_cost_line_id)
        REFERENCES landed_cost_lines (tenant_id, landed_cost_line_id)
        ON DELETE CASCADE
);

-- Indexes for landed_cost_allocations
CREATE INDEX idx_landed_cost_allocations_tenant_parent
    ON landed_cost_allocations(tenant_id, landed_cost_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_landed_cost_allocations_tenant_line
    ON landed_cost_allocations(tenant_id, landed_cost_line_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_landed_cost_allocations_tenant_target
    ON landed_cost_allocations(tenant_id, target_type, target_id)
    WHERE deleted_at IS NULL;

-- Unique constraint to prevent duplicate allocations for the same line+target
CREATE UNIQUE INDEX idx_landed_cost_allocations_unique
    ON landed_cost_allocations(tenant_id, landed_cost_line_id, target_type, target_id)
    WHERE deleted_at IS NULL;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp for landed_costs
CREATE TRIGGER update_landed_costs_updated_at
    BEFORE UPDATE ON landed_costs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auto-update updated_at timestamp for landed_cost_lines
CREATE TRIGGER update_landed_cost_lines_updated_at
    BEFORE UPDATE ON landed_cost_lines
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auto-update updated_at timestamp for landed_cost_allocations
CREATE TRIGGER update_landed_cost_allocations_updated_at
    BEFORE UPDATE ON landed_cost_allocations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE landed_costs IS 'Landed cost documents for allocating additional costs to goods receipts';
COMMENT ON COLUMN landed_costs.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN landed_costs.landed_cost_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN landed_costs.reference IS 'Human-readable reference number';
COMMENT ON COLUMN landed_costs.status IS 'Document status: draft/posted/cancelled';
COMMENT ON COLUMN landed_costs.grn_id IS 'Optional link to goods receipt';
COMMENT ON COLUMN landed_costs.posted_at IS 'Timestamp when document was posted';
COMMENT ON COLUMN landed_costs.posted_by IS 'User who posted the document';

COMMENT ON TABLE landed_cost_lines IS 'Individual cost lines within a landed cost document';
COMMENT ON COLUMN landed_cost_lines.cost_type IS 'Type of cost: freight/customs/handling/insurance/other';
COMMENT ON COLUMN landed_cost_lines.amount_cents IS 'Cost amount in smallest currency unit (cents)';
COMMENT ON COLUMN landed_cost_lines.allocation_method IS 'Method for allocating cost: by_value (MVP)';

COMMENT ON TABLE landed_cost_allocations IS 'Computed allocation of costs to target items';
COMMENT ON COLUMN landed_cost_allocations.target_type IS 'Type of target: grn_item or stock_move';
COMMENT ON COLUMN landed_cost_allocations.target_id IS 'ID of the target item';
COMMENT ON COLUMN landed_cost_allocations.allocated_amount_cents IS 'Allocated amount in cents';
