-- Migration: Create Cycle Count Schedules Tables
-- Description: Schema for recurring inventory counts (cycle counting schedules)
-- Purpose: Enable planned counting by location/category with ABC classification
-- Dependencies: warehouses, warehouse_locations, product_categories tables
-- Task: PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.06_create_cycle_count_schedules_table.md
-- Created: 2026-01-16

-- ==================================
-- CYCLE_COUNT_SCHEDULES TABLE (Header)
-- ==================================
-- Defines recurring schedule for cycle counting inventory
-- Each schedule targets a specific warehouse and can be scoped to locations/categories

CREATE TABLE cycle_count_schedules (
    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Primary key using UUID v4 (standard random UUID)
    schedule_id UUID NOT NULL DEFAULT gen_random_uuid(),

    -- Schedule identification
    name TEXT NOT NULL,
    description TEXT,

    -- Warehouse target (required)
    warehouse_id UUID NOT NULL,

    -- Scheduling configuration
    frequency TEXT NOT NULL DEFAULT 'monthly',  -- 'daily', 'weekly', 'monthly', 'quarterly', 'yearly', 'custom'
    interval_days INT,                          -- For 'custom' frequency or specific intervals
    timezone TEXT NOT NULL DEFAULT 'UTC',
    start_at TIMESTAMPTZ NOT NULL,              -- When the schedule starts
    next_run_at TIMESTAMPTZ NOT NULL,           -- Next scheduled execution
    end_at TIMESTAMPTZ,                         -- Optional end date

    -- ABC classification (optional)
    abc_class TEXT,                             -- 'A', 'B', 'C', or NULL for all
    min_value_cents BIGINT,                     -- Minimum inventory value threshold (in cents)
    min_qty BIGINT,                             -- Minimum quantity threshold

    -- Operational flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    auto_create_stock_take BOOLEAN NOT NULL DEFAULT false,  -- Auto-generate stock take on schedule
    notes TEXT,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- Soft delete (project standard)

    -- Composite primary key for multi-tenant isolation
    PRIMARY KEY (tenant_id, schedule_id),

    -- Unique schedule name per tenant
    CONSTRAINT cycle_count_schedules_name_unique_per_tenant
        UNIQUE (tenant_id, name),

    -- Composite FK to warehouses (tenant-scoped)
    CONSTRAINT cycle_count_schedules_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses(tenant_id, warehouse_id),

    -- Check constraints for scheduling logic
    CONSTRAINT cycle_count_schedules_frequency_check
        CHECK (frequency IN ('daily', 'weekly', 'monthly', 'quarterly', 'yearly', 'custom')),

    CONSTRAINT cycle_count_schedules_custom_interval_check
        CHECK (frequency != 'custom' OR interval_days IS NOT NULL),

    CONSTRAINT cycle_count_schedules_interval_positive_check
        CHECK (interval_days IS NULL OR interval_days > 0),

    CONSTRAINT cycle_count_schedules_abc_class_check
        CHECK (abc_class IS NULL OR abc_class IN ('A', 'B', 'C'))
);

-- ==================================
-- CYCLE_COUNT_SCHEDULE_LOCATIONS TABLE (Location Scope)
-- ==================================
-- Join table linking schedules to specific locations within the warehouse
-- If empty, the schedule applies to all locations in the warehouse

CREATE TABLE cycle_count_schedule_locations (
    -- Multi-tenancy
    tenant_id UUID NOT NULL,
    schedule_id UUID NOT NULL,
    location_id UUID NOT NULL,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key
    PRIMARY KEY (tenant_id, schedule_id, location_id),

    -- Composite FK to schedules
    CONSTRAINT ccsl_schedule_fk
        FOREIGN KEY (tenant_id, schedule_id)
        REFERENCES cycle_count_schedules(tenant_id, schedule_id)
        ON DELETE CASCADE,

    -- Composite FK to locations (uses tenant_location unique constraint)
    CONSTRAINT ccsl_location_fk
        FOREIGN KEY (tenant_id, location_id)
        REFERENCES warehouse_locations(tenant_id, location_id)
        ON DELETE CASCADE
);

-- ==================================
-- CYCLE_COUNT_SCHEDULE_CATEGORIES TABLE (Product Category Scope)
-- ==================================
-- Join table linking schedules to specific product categories
-- If empty, the schedule applies to all product categories

CREATE TABLE cycle_count_schedule_categories (
    -- Multi-tenancy
    tenant_id UUID NOT NULL,
    schedule_id UUID NOT NULL,
    category_id UUID NOT NULL,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key
    PRIMARY KEY (tenant_id, schedule_id, category_id),

    -- Composite FK to schedules
    CONSTRAINT ccsc_schedule_fk
        FOREIGN KEY (tenant_id, schedule_id)
        REFERENCES cycle_count_schedules(tenant_id, schedule_id)
        ON DELETE CASCADE,

    -- Composite FK to product categories
    CONSTRAINT ccsc_category_fk
        FOREIGN KEY (tenant_id, category_id)
        REFERENCES product_categories(tenant_id, category_id)
        ON DELETE CASCADE
);

-- ==================================
-- INDEXES for Performance
-- ==================================

-- Schedule lookup by next run time (for scheduler/cron jobs)
CREATE INDEX idx_cycle_count_schedules_next_run
    ON cycle_count_schedules(tenant_id, next_run_at)
    WHERE deleted_at IS NULL AND is_active = true;

-- Schedule lookup by warehouse
CREATE INDEX idx_cycle_count_schedules_warehouse
    ON cycle_count_schedules(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL;

-- Schedule lookup by ABC class
CREATE INDEX idx_cycle_count_schedules_abc_class
    ON cycle_count_schedules(tenant_id, abc_class)
    WHERE deleted_at IS NULL AND abc_class IS NOT NULL;

-- Active schedules for tenant
CREATE INDEX idx_cycle_count_schedules_active
    ON cycle_count_schedules(tenant_id, schedule_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- Location scopes by location
CREATE INDEX idx_cycle_count_schedule_locations_location
    ON cycle_count_schedule_locations(tenant_id, location_id);

-- Category scopes by category
CREATE INDEX idx_cycle_count_schedule_categories_category
    ON cycle_count_schedule_categories(tenant_id, category_id);

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

COMMENT ON TABLE cycle_count_schedules IS 'Recurring cycle counting schedules for warehouse inventory auditing';
COMMENT ON COLUMN cycle_count_schedules.schedule_id IS 'UUID v4 primary key (random)';
COMMENT ON COLUMN cycle_count_schedules.tenant_id IS 'Multi-tenant isolation field - all queries must filter by this';
COMMENT ON COLUMN cycle_count_schedules.warehouse_id IS 'Target warehouse for this schedule';
COMMENT ON COLUMN cycle_count_schedules.frequency IS 'Schedule frequency: daily, weekly, monthly, quarterly, yearly, or custom';
COMMENT ON COLUMN cycle_count_schedules.interval_days IS 'Custom interval in days (required when frequency=custom)';
COMMENT ON COLUMN cycle_count_schedules.abc_class IS 'Optional ABC classification filter (A/B/C)';
COMMENT ON COLUMN cycle_count_schedules.min_value_cents IS 'Minimum inventory value threshold in cents';
COMMENT ON COLUMN cycle_count_schedules.auto_create_stock_take IS 'If true, automatically creates stock take records on schedule trigger';

COMMENT ON TABLE cycle_count_schedule_locations IS 'Location scope for cycle count schedules (empty = all locations)';
COMMENT ON COLUMN cycle_count_schedule_locations.location_id IS 'Target warehouse location for counting';

COMMENT ON TABLE cycle_count_schedule_categories IS 'Product category scope for cycle count schedules (empty = all categories)';
COMMENT ON COLUMN cycle_count_schedule_categories.category_id IS 'Target product category for counting';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Recurring cycle count schedules (daily/weekly/monthly/custom)
-- 2. Location-based counting scopes
-- 3. Product category-based counting scopes
-- 4. ABC classification filtering
--
-- Follow-up tasks will add:
-- - API endpoints for CRUD operations (task 04.14.01)
-- - Scheduler/cron job to auto-generate stock takes
-- - Execution and reconciliation workflows
