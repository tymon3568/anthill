#!/usr/bin/env bash
# Setup test data for Kanidm OAuth2 testing

set -e

echo "🔧 Setting up Kanidm OAuth2 test environment..."

# Database connection
export DATABASE_URL="${DATABASE_URL:-postgres://anthill_test:anthill_test_password@localhost:5433/anthill_test}"

echo ""
echo "📊 Step 1: Running database migrations..."
sqlx migrate run

echo ""
echo "🏢 Step 2: Creating test tenant (ACME Corporation)..."
psql "$DATABASE_URL" << EOF
-- Delete existing test data
DELETE FROM kanidm_tenant_groups WHERE kanidm_group_name LIKE 'tenant_acme%';
DELETE FROM users WHERE tenant_id = '018c3f1e-1234-7890-abcd-000000000001';
DELETE FROM tenants WHERE slug = 'acme';

-- Create test tenant
INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
VALUES (
  '018c3f1e-1234-7890-abcd-000000000001'::uuid,
  'ACME Corporation',
  'acme',
  'enterprise',
  'active',
  '{}'::jsonb,
  NOW(),
  NOW()
)
ON CONFLICT (tenant_id) DO UPDATE
SET name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    updated_at = NOW();

-- Map Kanidm groups to tenant
INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name, role, created_at, updated_at)
VALUES 
  (
    '018c3f1e-1234-7890-abcd-000000000001'::uuid,
    '00000000-0000-0000-0000-000000000001'::uuid,
    'tenant_acme_admins',
    'admin',
    NOW(),
    NOW()
  ),
  (
    '018c3f1e-1234-7890-abcd-000000000001'::uuid,
    '00000000-0000-0000-0000-000000000002'::uuid,
    'tenant_acme_users',
    'member',
    NOW(),
    NOW()
  )
ON CONFLICT (tenant_id, kanidm_group_uuid) DO UPDATE
SET kanidm_group_name = EXCLUDED.kanidm_group_name,
    role = EXCLUDED.role,
    updated_at = NOW();

-- Verify
SELECT 
  t.name as tenant, 
  ktg.kanidm_group_name as group, 
  ktg.role 
FROM tenants t
JOIN kanidm_tenant_groups ktg ON t.tenant_id = ktg.tenant_id
WHERE t.slug = 'acme';
EOF

echo ""
echo "✅ Database setup complete!"
echo ""
echo "📝 Next steps:"
echo "1. Start Kanidm: docker run -d -p 8300:8443 --name kanidm kanidm/server:latest"
echo "2. Setup Kanidm OAuth2 client (see docs/KANIDM_OAUTH2_TESTING.md)"
echo "3. Create test user in Kanidm and add to 'tenant_acme_admins' group"
echo "4. Run user service with Kanidm config"
echo "5. Test OAuth2 flow with curl or Postman"
echo ""
echo "📖 Full testing guide: docs/KANIDM_OAUTH2_TESTING.md"
