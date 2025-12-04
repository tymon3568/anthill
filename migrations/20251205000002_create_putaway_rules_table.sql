-- Migration: Create putaway_rules table
-- Description: Creates the putaway_rules table for automated storage location assignment rules
-- Dependencies: storage_locations table (20251205000001), products table (20250110000017), product_categories table (20250110000021), warehouses table (20250110000023)
-- Created: 2025-12-05

-- ==================================
-- PUTAWAY_RULES TABLE (Putaway Rules for Automated Location Assignment)
-- ==================================
-- This table defines rules for automatically assigning storage locations to incoming goods
-- Rules are evaluated in sequence order to determine optimal putaway locations
-- Supports product-specific, category-based, and attribute-based rules

CREATE TABLE putaway_rules (
    -- Primary key using UUID v7 (timestamp-based)
    rule_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),

    -- Multi-tenancy: All queries must filter by tenant_id
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),

    -- Rule metadata
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Evaluation order (lower sequence = higher priority)
    sequence INTEGER NOT NULL,

    -- Rule scope (optional filters)
    product_id UUID,                    -- Specific product rule
    product_category_id UUID,           -- Category-based rule
    warehouse_id UUID,                  -- Warehouse-specific rule

    -- Location preferences
    preferred_location_type VARCHAR(50), -- Preferred location type (e.g., 'picking', 'bulk')
    preferred_zone VARCHAR(50),         -- Preferred zone
    preferred_aisle VARCHAR(50),        -- Preferred aisle

    -- Rule conditions (JSON for flexible attribute matching)
    conditions JSONB,                   -- Conditions like {"weight_kg": {"lt": 10}, "fragile": true}

    -- Rule logic
    rule_type VARCHAR(50) NOT NULL DEFAULT 'product', -- 'product', 'category', 'attribute', 'fifo', 'fefo'
    match_mode VARCHAR(20) NOT NULL DEFAULT 'exact',  -- 'exact', 'contains', 'regex'

    -- Constraints and limits
    max_quantity BIGINT,                -- Maximum quantity per location
    min_quantity BIGINT,                -- Minimum quantity per location
    priority_score INTEGER DEFAULT 0,   -- Scoring for rule ranking

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    updated_by UUID,

    -- Soft delete
    deleted_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT putaway_rules_tenant_product_fk
        FOREIGN KEY (tenant_id, product_id)
        REFERENCES products (tenant_id, product_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT putaway_rules_category_fk
        FOREIGN KEY (tenant_id, product_category_id)
        REFERENCES product_categories (tenant_id, category_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT putaway_rules_tenant_warehouse_fk
        FOREIGN KEY (tenant_id, warehouse_id)
        REFERENCES warehouses (tenant_id, warehouse_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT putaway_rules_tenant_created_by_fk
        FOREIGN KEY (tenant_id, created_by)
        REFERENCES users (tenant_id, user_id),
    CONSTRAINT putaway_rules_tenant_updated_by_fk
        FOREIGN KEY (tenant_id, updated_by)
        REFERENCES users (tenant_id, user_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT putaway_rules_sequence_check
        CHECK (sequence > 0),
    CONSTRAINT putaway_rules_rule_type_check
        CHECK (rule_type IN ('product', 'category', 'attribute', 'fifo', 'fefo')),
    CONSTRAINT putaway_rules_match_mode_check
        CHECK (match_mode IN ('exact', 'contains', 'regex')),
    CONSTRAINT putaway_rules_quantity_check
        CHECK (
            (max_quantity IS NULL OR max_quantity > 0) AND
            (min_quantity IS NULL OR min_quantity >= 0) AND
            (max_quantity IS NULL OR min_quantity IS NULL OR max_quantity >= min_quantity)
        ),
    CONSTRAINT putaway_rules_scope_check
        CHECK (
            -- At least one scope must be defined
            product_id IS NOT NULL OR
            product_category_id IS NOT NULL OR
            warehouse_id IS NOT NULL OR
            conditions IS NOT NULL
        )
);

-- ==================================
-- INDEXES for Performance
-- ==================================
-- Critical indexes for rule evaluation and putaway operations

-- Primary lookup indexes
CREATE INDEX idx_putaway_rules_tenant_sequence
    ON putaway_rules(tenant_id, sequence)
    WHERE deleted_at IS NULL AND is_active = true;

CREATE INDEX idx_putaway_rules_tenant_product
    ON putaway_rules(tenant_id, product_id)
    WHERE deleted_at IS NULL AND is_active = true;

CREATE INDEX idx_putaway_rules_tenant_category
    ON putaway_rules(tenant_id, product_category_id)
    WHERE deleted_at IS NULL AND is_active = true;

CREATE INDEX idx_putaway_rules_tenant_warehouse
    ON putaway_rules(tenant_id, warehouse_id)
    WHERE deleted_at IS NULL AND is_active = true;

-- Rule evaluation indexes
CREATE INDEX idx_putaway_rules_tenant_type
    ON putaway_rules(tenant_id, rule_type, sequence)
    WHERE deleted_at IS NULL AND is_active = true;

-- JSON condition indexes for attribute-based rules
CREATE INDEX idx_putaway_rules_conditions
    ON putaway_rules USING GIN (conditions)
    WHERE deleted_at IS NULL AND is_active = true AND conditions IS NOT NULL;

-- Query optimization indexes
CREATE INDEX idx_putaway_rules_tenant_active
    ON putaway_rules(tenant_id, is_active)
    WHERE deleted_at IS NULL;

-- ==================================
-- FUNCTIONS
-- ==================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ==================================
-- TRIGGERS
-- ==================================

-- Auto-update updated_at timestamp
CREATE TRIGGER update_putaway_rules_updated_at
    BEFORE UPDATE ON putaway_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==================================
-- ROW LEVEL SECURITY (Future)
-- ==================================
-- Note: We use application-level filtering instead of RLS
-- All queries must include: WHERE tenant_id = $1 AND deleted_at IS NULL

-- ==================================
-- COMMENTS for Documentation
-- ==================================

COMMENT ON TABLE putaway_rules IS 'Putaway rules for automated storage location assignment based on product characteristics and business rules';
COMMENT ON COLUMN putaway_rules.rule_id IS 'UUID v7 primary key (timestamp-based)';
COMMENT ON COLUMN putaway_rules.tenant_id IS 'Multi-tenant isolation field';
COMMENT ON COLUMN putaway_rules.name IS 'Human-readable rule name';
COMMENT ON COLUMN putaway_rules.description IS 'Detailed rule description';
COMMENT ON COLUMN putaway_rules.sequence IS 'Rule evaluation order (lower = higher priority)';
COMMENT ON COLUMN putaway_rules.product_id IS 'Specific product this rule applies to';
COMMENT ON COLUMN putaway_rules.product_category_id IS 'Product category this rule applies to';
COMMENT ON COLUMN putaway_rules.warehouse_id IS 'Warehouse this rule applies to';
COMMENT ON COLUMN putaway_rules.preferred_location_type IS 'Preferred location type for putaway';
COMMENT ON COLUMN putaway_rules.preferred_zone IS 'Preferred zone for putaway';
COMMENT ON COLUMN putaway_rules.preferred_aisle IS 'Preferred aisle for putaway';
COMMENT ON COLUMN putaway_rules.conditions IS 'JSON conditions for attribute-based matching';
COMMENT ON COLUMN putaway_rules.rule_type IS 'Type of rule: product, category, attribute, fifo, fefo';
COMMENT ON COLUMN putaway_rules.match_mode IS 'How conditions are matched: exact, contains, regex';
COMMENT ON COLUMN putaway_rules.max_quantity IS 'Maximum quantity allowed per location';
COMMENT ON COLUMN putaway_rules.min_quantity IS 'Minimum quantity required per location';
COMMENT ON COLUMN putaway_rules.priority_score IS 'Additional scoring for rule ranking';
COMMENT ON COLUMN putaway_rules.is_active IS 'Whether this rule is active for evaluation';

-- ==================================
-- MIGRATION METADATA
-- ==================================

-- This migration creates the foundation for:
-- 1. Automated putaway rule definition and management
-- 2. Flexible rule conditions using JSON for attribute matching
-- 3. Rule prioritization and sequencing
-- 4. Integration with warehouse location hierarchy
-- 5. Product and category-based rule assignment

-- Key improvements from task requirements:
-- - Sequence-based rule evaluation for priority handling
-- - Multiple rule types (product, category, attribute, FIFO/FEFO)
-- - JSON conditions for flexible attribute matching
-- - Location preferences for zone/aisle targeting
-- - Quantity constraints for capacity management
-- - Soft delete support for rule lifecycle management

-- Future migrations will add:
-- - Rule evaluation engine implementation
-- - Putaway suggestion APIs
-- - Integration with GRN workflow
-- - Rule performance analytics
-- - Mobile putaway interfaces
