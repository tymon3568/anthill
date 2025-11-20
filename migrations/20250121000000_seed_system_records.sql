-- Migration: Seed system warehouse and user records
-- This migration creates system warehouse and user records that are referenced
-- by delivery orders created from events. These records are needed to satisfy
-- foreign key constraints in delivery_orders table.

-- Note: In a real deployment, these would be created per-tenant or through
-- configuration. For now, we create them for a default tenant.

-- Insert system warehouse (used for automated delivery orders)
INSERT INTO warehouses (
    warehouse_id,
    tenant_id,
    warehouse_code,
    warehouse_name,
    address_line1,
    city,
    state,
    postal_code,
    country,
    is_active,
    created_by,
    created_at,
    updated_at
) VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid, -- Default tenant ID
    'SYS-WH',
    'System Warehouse',
    'System Address',
    'System City',
    'System State',
    '00000',
    'System Country',
    true,
    '00000000-0000-0000-0000-000000000002'::uuid,
    NOW(),
    NOW()
) ON CONFLICT (tenant_id, warehouse_id) DO NOTHING;

-- Insert system user (used for automated delivery orders)
INSERT INTO users (
    user_id,
    tenant_id,
    email,
    first_name,
    last_name,
    is_active,
    created_at,
    updated_at
) VALUES (
    '00000000-0000-0000-0000-000000000002'::uuid,
    '00000000-0000-0000-0000-000000000000'::uuid, -- Default tenant ID
    'system@anthill.local',
    'System',
    'User',
    true,
    NOW(),
    NOW()
) ON CONFLICT (tenant_id, user_id) DO NOTHING;
