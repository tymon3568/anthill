-- Create reorder_rules table for automated stock replenishment
CREATE TABLE reorder_rules (
    rule_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    product_id UUID NOT NULL,
    warehouse_id UUID,
    reorder_point BIGINT NOT NULL CHECK (reorder_point >= 0),
    min_quantity BIGINT NOT NULL CHECK (min_quantity >= 0),
    max_quantity BIGINT NOT NULL CHECK (max_quantity >= min_quantity),
    lead_time_days INTEGER NOT NULL CHECK (lead_time_days > 0),
    safety_stock BIGINT NOT NULL DEFAULT 0 CHECK (safety_stock >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id),
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id),
    FOREIGN KEY (tenant_id, warehouse_id) REFERENCES warehouses(tenant_id, warehouse_id)
);

-- Create indexes for performance
CREATE INDEX idx_reorder_rules_tenant_product ON reorder_rules(tenant_id, product_id);
CREATE INDEX idx_reorder_rules_tenant_warehouse ON reorder_rules(tenant_id, warehouse_id);
CREATE UNIQUE INDEX idx_reorder_rules_active ON reorder_rules(tenant_id, product_id, warehouse_id) WHERE deleted_at IS NULL;
