-- Migration: Create Quality Control Tables
-- Description: Creates tables for comprehensive quality management system
-- Created: 2025-12-03

-- Create enum types for quality control
CREATE TYPE qc_point_type AS ENUM ('incoming', 'outgoing', 'internal');
CREATE TYPE qc_status AS ENUM ('pending', 'passed', 'failed');
CREATE TYPE qc_test_type AS ENUM ('pass_fail', 'measure', 'picture');
CREATE TYPE alert_priority AS ENUM ('low', 'medium', 'high');
CREATE TYPE alert_status AS ENUM ('open', 'in_progress', 'resolved');

-- Create quality_control_points table
CREATE TABLE quality_control_points (
    qc_point_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    type qc_point_type NOT NULL,
    product_id UUID,
    warehouse_id UUID,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id),
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id),
    FOREIGN KEY (tenant_id, warehouse_id) REFERENCES warehouses(tenant_id, warehouse_id)
);

-- Create quality_checks table
CREATE TABLE quality_checks (
    qc_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    qc_point_id UUID NOT NULL,
    reference_type TEXT NOT NULL, -- e.g., 'receipt', 'delivery', 'transfer'
    reference_id UUID NOT NULL,
    product_id UUID NOT NULL,
    lot_serial_id UUID,
    status qc_status NOT NULL DEFAULT 'pending',
    inspector_id UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id),
    FOREIGN KEY (tenant_id, qc_point_id) REFERENCES quality_control_points(tenant_id, qc_point_id),
    FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id),
    FOREIGN KEY (tenant_id, lot_serial_id) REFERENCES lot_serial_numbers(tenant_id, lot_serial_id)
);

-- Create quality_check_lines table
CREATE TABLE quality_check_lines (
    qc_line_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    qc_id UUID NOT NULL,
    test_type qc_test_type NOT NULL,
    name TEXT NOT NULL,
    value TEXT,
    min_value BIGINT,
    max_value BIGINT,
    uom_id UUID,
    result BOOLEAN,
    notes TEXT,

    FOREIGN KEY (qc_id) REFERENCES quality_checks(qc_id) ON DELETE CASCADE,
    FOREIGN KEY (uom_id) REFERENCES unit_of_measures(uom_id)
);

-- Create quality_alerts table
CREATE TABLE quality_alerts (
    alert_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    qc_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    priority alert_priority NOT NULL DEFAULT 'medium',
    status alert_status NOT NULL DEFAULT 'open',
    assigned_to UUID,
    resolution TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id),
    FOREIGN KEY (qc_id) REFERENCES quality_checks(qc_id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_to) REFERENCES users(user_id)
);

-- Create indexes for performance
CREATE INDEX idx_quality_control_points_tenant_active ON quality_control_points(tenant_id, active);
CREATE INDEX idx_quality_control_points_tenant_product ON quality_control_points(tenant_id, product_id);
CREATE INDEX idx_quality_control_points_tenant_warehouse ON quality_control_points(tenant_id, warehouse_id);

CREATE INDEX idx_quality_checks_tenant_qc_point ON quality_checks(tenant_id, qc_point_id);
CREATE INDEX idx_quality_checks_tenant_reference ON quality_checks(tenant_id, reference_type, reference_id);
CREATE INDEX idx_quality_checks_tenant_product ON quality_checks(tenant_id, product_id);
CREATE INDEX idx_quality_checks_tenant_status ON quality_checks(tenant_id, status);

CREATE INDEX idx_quality_alerts_tenant_status ON quality_alerts(tenant_id, status);
CREATE INDEX idx_quality_alerts_tenant_priority ON quality_alerts(tenant_id, priority);
CREATE INDEX idx_quality_alerts_assigned_to ON quality_alerts(assigned_to);
