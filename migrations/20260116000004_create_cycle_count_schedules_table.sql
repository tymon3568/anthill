-- Migration: Create cycle_count_schedules tables
-- Description: Cycle counting schedule tables for multi-tenant warehouse operations
-- Dependencies: warehouses, warehouse_locations tables
-- Created: 2026-01-16
-- Task: task_04.02.06_create_cycle_count_schedules_table.md

-- ==================================
-- CYCLE_COUNT_SCHEDULES TABLE (Header)
-- ==================================
-- Schedule configuration for recurring inventory cycle counts
-- Supports ABC classification, warehouse/location scoping, and auto stock take generation
--
-- Design decisions:
-- - Composite PK: (tenant_id, schedule_id) for tenant isolation
-- - Uses warehouse FK via composite unique constraint
-- - Soft delete: deleted_at TIMESTAMPTZ (project standard)
-- - Frequency types: daily, weekly, monthly, quarterly, yearly, custom

CREATE TABLE cycle_count_schedules (
    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Primary key using UUID v4
    schedule_id UUID NOT NULL DEFAULT gen_random_uuid(),

    -- Schedule identifiers
    name TEXT NOT NULL,                    -- Human-readable name (e.g., "Weekly A-Class Count")
    description TEXT,                      -- Optional description

    -- Warehouse scope (required)
    warehouse_id UUID NOT NULL,

    -- Scheduling configuration
    frequency TEXT NOT NULL DEFAULT 'monthly'
        CHECK (frequency IN ('daily', 'weekly', 'monthly', 'quarterly', 'yearly', 'custom')),
    interval_days INT,                     -- For 'custom' frequency or fine-grained control
    timezone TEXT NOT NULL DEFAULT 'UTC',  -- Timezone for scheduling
    start_at TIMESTAMPTZ NOT NULL,         -- When schedule becomes active
    next_run_at TIMESTAMPTZ NOT NULL,      -- Next scheduled execution
    end_at TIMESTAMPTZ,                    -- Optional end date (NULL = no end)

    -- ABC classification filtering (optional)
    abc_class TEXT                         -- 'A', 'B', 'C', or NULL for all
        CHECK (abc_class IS NULL OR abc_class IN ('A', 'B', 'C')),

    -- Value/quantity thresholds (optional)
    min_value_cents BIGINT,                -- Minimum inventory value in cents
    max_value_cents BIGINT,                -- Maximum inventory value in cents
    min_qty BIGINT,                        -- Minimum quantity threshold
    max_qty BIGINT,                        -- Maximum quantity threshold

    -- Operational flags
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    auto_create_stock_take BOOLEAN NOT NULL DEFAULT FALSE,  -- Auto-generate stock take records
    notes TEXT,                            -- Optional notes

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,                -- Soft delete

    -- Composite primary key for tenant isolation
    PRIMARY KEY (tenant_id, schedule_id),

    -- Composite FK to warehouses (via unique constraint)
    CONSTRAINT cycle_count_schedules_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id),

    -- Check constraints
    CONSTRAINT cycle_count_schedules_custom_interval
        CHECK (frequency != 'custom' OR interval_days IS NOT NULL),
    CONSTRAINT cycle_count_schedules_interval_positive
        CHECK (interval_days IS NULL OR interval_days > 0),
    CONSTRAINT cycle_count_schedules_value_range
        CHECK (min_value_cents IS NULL OR max_value_cents IS NULL OR min_value_cents <= max_value_cents),
    CONSTRAINT cycle_count_schedules_qty_range
        CHECK (min_qty IS NULL OR max_qty IS NULL OR min_qty <= max_qty)
);

-- ==================================
-- CYCLE_COUNT_SCHEDULE_LOCATIONS TABLE (Location Scope)
-- ==================================
-- Target locations for a cycle count schedule
-- Allows scoping schedules to specific warehouse locations

CREATE TABLE cycle_count_schedule_locations (
    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- References
    schedule_id UUID NOT NULL,
    location_id UUID NOT NULL,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key
    PRIMARY KEY (tenant_id, schedule_id, location_id),

    -- Composite FK to schedule
    CONSTRAINT cycle_count_schedule_locations_schedule_fk
        FOREIGN KEY (tenant_id, schedule_id)
        REFERENCES cycle_count_schedules (tenant_id, schedule_id)
        ON DELETE CASCADE,

    -- FK to location (simple FK since location_id is unique)
    CONSTRAINT cycle_count_schedule_locations_location_fk
        FOREIGN KEY (location_id)
        REFERENCES warehouse_locations (location_id)
        ON DELETE CASCADE
);

-- ==================================
-- CYCLE_COUNT_SCHEDULE_CATEGORIES TABLE (Product Category Scope)
-- ==================================
-- Target product categories for a cycle count schedule
-- Allows scoping schedules to specific product categories
-- Note: FK to product_categories is omitted since categories use simple PK
-- Application layer should validate category_id exists for tenant

CREATE TABLE cycle_count_schedule_categories (
    -- Multi-tenancy
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- References
    schedule_id UUID NOT NULL,
    category_id UUID NOT NULL,             -- Reference to product_categories.category_id

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key
    PRIMARY KEY (tenant_id, schedule_id, category_id),

    -- Composite FK to schedule
    CONSTRAINT cycle_count_schedule_categories_schedule_fk
        FOREIGN KEY (tenant_id, schedule_id)
        REFERENCES cycle_count_schedules (tenant_id, schedule_id)
        ON DELETE CASCADE

    -- Note: FK to product_categories omitted because:
    -- 1. product_categories uses simple PK (category_id), not composite
    -- 2. Cross-tenant protection is handled at application layer
    -- 3. Adding FK would require schema change to product_categories
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Next run lookup (for scheduler job)
CREATE INDEX idx_cycle_count_schedules_next_run
    ON cycle_count_schedules (tenant_id, next_run_at)
    WHERE deleted_at IS NULL AND is_active = TRUE;

-- Warehouse lookup
CREATE INDEX idx_cycle_count_schedules_warehouse
    ON cycle_count_schedules (tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

-- Active schedules list
CREATE INDEX idx_cycle_count_schedules_active
    ON cycle_count_schedules (tenant_id, is_active)
    WHERE deleted_at IS NULL;

-- ABC class filtering
CREATE INDEX idx_cycle_count_schedules_abc
    ON cycle_count_schedules (tenant_id, abc_class)
    WHERE deleted_at IS NULL AND abc_class IS NOT NULL;

-- Location scope lookup
CREATE INDEX idx_cycle_count_schedule_locations_location
    ON cycle_count_schedule_locations (tenant_id, location_id);

CREATE INDEX idx_cycle_count_schedule_locations_schedule
    ON cycle_count_schedule_locations (tenant_id, schedule_id);

-- Category scope lookup
CREATE INDEX idx_cycle_count_schedule_categories_category
    ON cycle_count_schedule_categories (tenant_id, category_id);

CREATE INDEX idx_cycle_count_schedule_categories_schedule
    ON cycle_count_schedule_categories (tenant_id, schedule_id);

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_cycle_count_schedules_updated_at
    BEFORE UPDATE ON cycle_count_schedules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE cycle_count_schedules IS 'Cycle counting schedule configuration for recurring inventory counts';
COMMENT ON COLUMN cycle_count_schedules.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN cycle_count_schedules.schedule_id IS 'UUID v4 primary key';
COMMENT ON COLUMN cycle_count_schedules.name IS 'Human-readable schedule name';
COMMENT ON COLUMN cycle_count_schedules.warehouse_id IS 'Target warehouse for cycle counting';
COMMENT ON COLUMN cycle_count_schedules.frequency IS 'Schedule frequency: daily/weekly/monthly/quarterly/yearly/custom';
COMMENT ON COLUMN cycle_count_schedules.interval_days IS 'Custom interval in days (required for custom frequency)';
COMMENT ON COLUMN cycle_count_schedules.timezone IS 'Timezone for schedule execution';
COMMENT ON COLUMN cycle_count_schedules.abc_class IS 'Optional ABC classification filter (A/B/C)';
COMMENT ON COLUMN cycle_count_schedules.min_value_cents IS 'Minimum inventory value threshold in cents';
COMMENT ON COLUMN cycle_count_schedules.auto_create_stock_take IS 'Automatically generate stock take records on schedule';
COMMENT ON COLUMN cycle_count_schedules.deleted_at IS 'Soft delete timestamp (NULL = active)';

COMMENT ON TABLE cycle_count_schedule_locations IS 'Location scope for cycle count schedules';
COMMENT ON COLUMN cycle_count_schedule_locations.location_id IS 'Target warehouse location';

COMMENT ON TABLE cycle_count_schedule_categories IS 'Product category scope for cycle count schedules';
COMMENT ON COLUMN cycle_count_schedule_categories.category_id IS 'Target product category (validated at application layer)';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Recurring cycle count scheduling
-- 2. ABC classification-based counting
-- 3. Location and category scoped counting
-- 4. Automated stock take generation (via auto_create_stock_take flag)

-- Next features will include:
-- - API endpoints for schedule CRUD
-- - Scheduler job to process next_run_at and generate stock takes
-- - Integration with existing stock_takes table
