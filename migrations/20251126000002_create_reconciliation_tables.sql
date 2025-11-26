-- Create stock reconciliation tables for cycle counting and variance analysis
-- Migration: 20251126000002_create_reconciliation_tables.sql

-- Stock reconciliations table (reconciliation sessions)
CREATE TABLE stock_reconciliations (
    tenant_id UUID NOT NULL,
    reconciliation_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    reconciliation_number TEXT NOT NULL, -- Auto-generated sequence like REC-2025-001
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK (status IN ('draft', 'in_progress', 'completed', 'cancelled')) DEFAULT 'draft',
    cycle_type TEXT CHECK (cycle_type IN ('full', 'abc_a', 'abc_b', 'abc_c', 'location', 'random')) DEFAULT 'full',
    warehouse_id UUID,
    location_filter JSONB, -- For location-based filtering
    product_filter JSONB, -- For product category/ABC filtering
    total_items INTEGER DEFAULT 0,
    counted_items INTEGER DEFAULT 0,
    total_variance BIGINT DEFAULT 0, -- In base UOM
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,

    FOREIGN KEY (tenant_id, warehouse_id) REFERENCES warehouses(tenant_id, warehouse_id),
    FOREIGN KEY (tenant_id, created_by) REFERENCES users(tenant_id, user_id),
    FOREIGN KEY (tenant_id, approved_by) REFERENCES users(tenant_id, user_id),
    FOREIGN KEY (tenant_id, deleted_by) REFERENCES users(tenant_id, user_id)
);

-- Indexes for stock_reconciliations
CREATE INDEX idx_stock_reconciliations_tenant_status ON stock_reconciliations(tenant_id, status);
CREATE INDEX idx_stock_reconciliations_tenant_warehouse ON stock_reconciliations(tenant_id, warehouse_id);
CREATE INDEX idx_stock_reconciliations_tenant_created_at ON stock_reconciliations(tenant_id, created_at DESC);
CREATE INDEX idx_stock_reconciliations_tenant_cycle_type ON stock_reconciliations(tenant_id, cycle_type);
-- Filtered index for non-deleted reconciliations
CREATE INDEX idx_stock_reconciliations_tenant_number_not_deleted
    ON stock_reconciliations(tenant_id, reconciliation_number)
    WHERE deleted_at IS NULL;

-- Unique constraint for reconciliation numbers per tenant
ALTER TABLE stock_reconciliations
    ADD CONSTRAINT unique_tenant_reconciliation_number
    UNIQUE (tenant_id, reconciliation_number);

-- Ensure (tenant_id, reconciliation_id) is uniquely identifiable for FKs
ALTER TABLE stock_reconciliations
    ADD CONSTRAINT unique_tenant_reconciliation_id
    UNIQUE (tenant_id, reconciliation_id);

-- Stock reconciliation items table (item-level counts)
CREATE TABLE stock_reconciliation_items (
    tenant_id UUID NOT NULL,
    reconciliation_id UUID NOT NULL,
    product_id UUID NOT NULL,
    warehouse_id UUID NOT NULL,
    location_id UUID, -- Specific location within warehouse
    expected_quantity BIGINT NOT NULL DEFAULT 0, -- Expected stock level
    counted_quantity BIGINT, -- Actual counted quantity (NULL if not counted)
    variance BIGINT, -- counted_quantity - expected_quantity (computed)
    variance_percentage DECIMAL(10,4), -- (variance / expected_quantity) * 100
    unit_cost BIGINT NOT NULL, -- Cost per unit in cents (e.g., $10.50 = 1050)
    variance_value BIGINT, -- variance * unit_cost in cents
    notes TEXT,
    counted_by UUID,
    counted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,

    PRIMARY KEY (tenant_id, reconciliation_id, product_id, warehouse_id),
    FOREIGN KEY (tenant_id, reconciliation_id) REFERENCES stock_reconciliations(tenant_id, reconciliation_id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id),
    FOREIGN KEY (tenant_id, warehouse_id) REFERENCES warehouses(tenant_id, warehouse_id),
    FOREIGN KEY (tenant_id, counted_by) REFERENCES users(tenant_id, user_id),
    FOREIGN KEY (tenant_id, deleted_by) REFERENCES users(tenant_id, user_id)
);

-- Indexes for stock_reconciliation_items
CREATE INDEX idx_stock_reconciliation_items_tenant_reconciliation ON stock_reconciliation_items(tenant_id, reconciliation_id);
CREATE INDEX idx_stock_reconciliation_items_tenant_product ON stock_reconciliation_items(tenant_id, product_id);
CREATE INDEX idx_stock_reconciliation_items_tenant_warehouse ON stock_reconciliation_items(tenant_id, warehouse_id);
CREATE INDEX idx_stock_reconciliation_items_tenant_counted_at ON stock_reconciliation_items(tenant_id, counted_at);
CREATE INDEX idx_stock_reconciliation_items_variance ON stock_reconciliation_items(tenant_id, reconciliation_id, variance);
-- Filtered index for non-deleted items
CREATE INDEX idx_stock_reconciliation_items_not_deleted
    ON stock_reconciliation_items(tenant_id, reconciliation_id, product_id)
    WHERE deleted_at IS NULL;

-- Add reconciliation_number sequence (per tenant)
-- We'll use a function to generate REC-YYYY-NNN format
CREATE OR REPLACE FUNCTION generate_reconciliation_number(p_tenant_id UUID)
RETURNS TEXT AS $$
DECLARE
    current_year TEXT := EXTRACT(YEAR FROM NOW())::TEXT;
    next_number INTEGER;
    reconciliation_number TEXT;
    lock_key BIGINT;
BEGIN
    -- Create a unique lock key based on tenant and year to prevent race conditions
    lock_key := ('x' || substring(p_tenant_id::text, 1, 8))::bit(32)::int + EXTRACT(YEAR FROM NOW())::int;

    -- Acquire advisory lock to prevent concurrent number generation
    PERFORM pg_advisory_lock(lock_key);

    BEGIN
        -- Get next number for this tenant and year
        SELECT COALESCE(MAX(CAST(SPLIT_PART(reconciliation_number, '-', 3) AS INTEGER)), 0) + 1
        INTO next_number
        FROM stock_reconciliations
        WHERE tenant_id = p_tenant_id
          AND reconciliation_number LIKE 'REC-' || current_year || '-%';

        -- Format as REC-2025-001
        reconciliation_number := 'REC-' || current_year || '-' || LPAD(next_number::TEXT, 3, '0');

        -- Release the advisory lock
        PERFORM pg_advisory_unlock(lock_key);

        RETURN reconciliation_number;
    EXCEPTION
        WHEN OTHERS THEN
            -- Ensure lock is released even on error
            PERFORM pg_advisory_unlock(lock_key);
            RAISE;
    END;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-generate reconciliation_number
CREATE OR REPLACE FUNCTION set_reconciliation_number()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.reconciliation_number IS NULL OR NEW.reconciliation_number = '' THEN
        NEW.reconciliation_number := generate_reconciliation_number(NEW.tenant_id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_set_reconciliation_number
    BEFORE INSERT ON stock_reconciliations
    FOR EACH ROW
    EXECUTE FUNCTION set_reconciliation_number();

-- Update trigger for updated_at
CREATE TRIGGER trigger_stock_reconciliations_updated_at
    BEFORE UPDATE ON stock_reconciliations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_stock_reconciliation_items_updated_at
    BEFORE UPDATE ON stock_reconciliation_items
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to compute variance when counted_quantity is updated
CREATE OR REPLACE FUNCTION compute_reconciliation_variance()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.counted_quantity IS NOT NULL THEN
        NEW.variance := NEW.counted_quantity - NEW.expected_quantity;
        IF NEW.expected_quantity != 0 THEN
            NEW.variance_percentage := (NEW.variance::DECIMAL / NEW.expected_quantity) * 100;
        ELSE
            NEW.variance_percentage := NULL;
        END IF;
        -- Compute variance value in cents: variance * unit_cost_cents
        NEW.variance_value := NEW.variance * NEW.unit_cost;
    ELSE
        -- Reset derived fields when count is cleared
        NEW.variance := NULL;
        NEW.variance_percentage := NULL;
        NEW.variance_value := NULL;
    END IF;

    -- Reset derived fields when expected_quantity is zero to avoid division by zero
    IF NEW.expected_quantity = 0 THEN
        NEW.variance_percentage := NULL;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_compute_reconciliation_variance
    BEFORE INSERT OR UPDATE ON stock_reconciliation_items
    FOR EACH ROW
    EXECUTE FUNCTION compute_reconciliation_variance();

-- Function to update reconciliation summary incrementally when items are counted
CREATE OR REPLACE FUNCTION update_reconciliation_summary()
RETURNS TRIGGER AS $$
DECLARE
    tenant_id UUID;
    reconciliation_id UUID;
    delta_counted INTEGER := 0;
    delta_variance BIGINT := 0;
BEGIN
    -- Determine tenant and reconciliation ID
    tenant_id := COALESCE(NEW.tenant_id, OLD.tenant_id);
    reconciliation_id := COALESCE(NEW.reconciliation_id, OLD.reconciliation_id);

    -- Handle INSERT
    IF TG_OP = 'INSERT' THEN
        IF NEW.counted_quantity IS NOT NULL THEN
            delta_counted := 1;
            delta_variance := COALESCE(NEW.variance, 0);
        END IF;

    -- Handle UPDATE
    ELSIF TG_OP = 'UPDATE' THEN
        -- From not counted to counted
        IF OLD.counted_quantity IS NULL AND NEW.counted_quantity IS NOT NULL THEN
            delta_counted := 1;
            delta_variance := COALESCE(NEW.variance, 0);
        -- From counted to not counted
        ELSIF OLD.counted_quantity IS NOT NULL AND NEW.counted_quantity IS NULL THEN
            delta_counted := -1;
            delta_variance := -COALESCE(OLD.variance, 0);
        -- Both counted, variance changed
        ELSIF OLD.counted_quantity IS NOT NULL AND NEW.counted_quantity IS NOT NULL THEN
            delta_variance := COALESCE(NEW.variance, 0) - COALESCE(OLD.variance, 0);
        END IF;

    -- Handle DELETE
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.counted_quantity IS NOT NULL THEN
            delta_counted := -1;
            delta_variance := -COALESCE(OLD.variance, 0);
        END IF;
    END IF;

    -- Apply deltas if any change
    IF delta_counted != 0 OR delta_variance != 0 THEN
        UPDATE stock_reconciliations
        SET
            counted_items = counted_items + delta_counted,
            total_variance = total_variance + delta_variance,
            updated_at = NOW()
        WHERE stock_reconciliations.tenant_id = tenant_id
          AND stock_reconciliations.reconciliation_id = reconciliation_id;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_reconciliation_summary
    AFTER INSERT OR UPDATE OR DELETE ON stock_reconciliation_items
    FOR EACH ROW
    EXECUTE FUNCTION update_reconciliation_summary();
