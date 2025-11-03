#!/bin/bash
# Setup Kanidm groups for all tenants
# Creates user and admin groups for each tenant and maps them to PostgreSQL

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DATABASE_URL=${DATABASE_URL:-"postgresql://anthill:anthill@localhost:5432/anthill"}
KANIDM_URL=${KANIDM_URL:-"https://idm.example.com"}
DRY_RUN=${DRY_RUN:-"false"}

echo -e "${GREEN}🔧 Setting up Kanidm Tenant Groups${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Database:  $DATABASE_URL"
echo "Kanidm:    $KANIDM_URL"
echo "Dry Run:   $DRY_RUN"
echo ""

# Check database connection
if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
  echo -e "${RED}❌ Error: Cannot connect to database${NC}"
  exit 1
fi

# Check Kanidm connection (if not dry run)
if [ "$DRY_RUN" != "true" ]; then
  if ! kanidm self whoami > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Not logged in to Kanidm${NC}"
    echo "Run: kanidm login admin"
    exit 1
  fi
  echo -e "${GREEN}✓ Logged in to Kanidm as:${NC} $(kanidm self whoami)"
  echo ""
fi

# Fetch tenants from database
echo -e "${YELLOW}📋 Fetching tenants...${NC}"
TENANTS_JSON=$(psql "$DATABASE_URL" -t -A -c "
  SELECT json_agg(row_to_json(t))::text
  FROM (
    SELECT tenant_id::text, slug, name 
    FROM tenants 
    WHERE deleted_at IS NULL AND status = 'active'
    ORDER BY slug
  ) t;
")

# Parse tenant count
TENANT_COUNT=$(echo "$TENANTS_JSON" | jq '. | length')

if [ "$TENANT_COUNT" == "null" ] || [ "$TENANT_COUNT" == "0" ]; then
  echo -e "${YELLOW}⚠️  No active tenants found${NC}"
  exit 0
fi

echo -e "${GREEN}Found $TENANT_COUNT tenant(s)${NC}"
echo ""

# Process each tenant
for i in $(seq 0 $((TENANT_COUNT - 1))); do
  TENANT=$(echo "$TENANTS_JSON" | jq -r ".[$i]")
  
  TENANT_ID=$(echo "$TENANT" | jq -r '.tenant_id')
  SLUG=$(echo "$TENANT" | jq -r '.slug')
  NAME=$(echo "$TENANT" | jq -r '.name')
  
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BLUE}[$((i+1))/$TENANT_COUNT] Tenant: $SLUG ($NAME)${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  
  # Group names
  USERS_GROUP="tenant_${SLUG}_users"
  ADMINS_GROUP="tenant_${SLUG}_admins"
  
  if [ "$DRY_RUN" == "true" ]; then
    echo "  [DRY RUN] Would create group: $USERS_GROUP"
    echo "  [DRY RUN] Would create group: $ADMINS_GROUP"
    echo "  [DRY RUN] Would map to tenant_id: $TENANT_ID"
    echo ""
    continue
  fi
  
  # ============================================================================
  # Create users group
  # ============================================================================
  
  echo -e "${YELLOW}Creating users group: $USERS_GROUP${NC}"
  
  if kanidm group get "$USERS_GROUP" > /dev/null 2>&1; then
    echo "  ℹ️  Group already exists"
    USERS_GROUP_UUID=$(kanidm group get "$USERS_GROUP" --output json | jq -r '.uuid')
  else
    kanidm group create "$USERS_GROUP" "$NAME - Users"
    USERS_GROUP_UUID=$(kanidm group get "$USERS_GROUP" --output json | jq -r '.uuid')
    echo -e "  ${GREEN}✓ Created${NC}"
  fi
  
  echo "  UUID: $USERS_GROUP_UUID"
  
  # ============================================================================
  # Create admins group
  # ============================================================================
  
  echo -e "${YELLOW}Creating admins group: $ADMINS_GROUP${NC}"
  
  if kanidm group get "$ADMINS_GROUP" > /dev/null 2>&1; then
    echo "  ℹ️  Group already exists"
    ADMINS_GROUP_UUID=$(kanidm group get "$ADMINS_GROUP" --output json | jq -r '.uuid')
  else
    kanidm group create "$ADMINS_GROUP" "$NAME - Admins"
    ADMINS_GROUP_UUID=$(kanidm group get "$ADMINS_GROUP" --output json | jq -r '.uuid')
    echo -e "  ${GREEN}✓ Created${NC}"
  fi
  
  echo "  UUID: $ADMINS_GROUP_UUID"
  
  # ============================================================================
  # Map groups to tenant in PostgreSQL
  # ============================================================================
  
  echo -e "${YELLOW}Mapping groups to tenant in database...${NC}"
  
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<SQL
    INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name, role)
    VALUES 
      ('$TENANT_ID', '$USERS_GROUP_UUID', '$USERS_GROUP', 'member'),
      ('$TENANT_ID', '$ADMINS_GROUP_UUID', '$ADMINS_GROUP', 'admin')
    ON CONFLICT (tenant_id, kanidm_group_uuid) 
    DO UPDATE SET 
      kanidm_group_name = EXCLUDED.kanidm_group_name,
      role = EXCLUDED.role,
      updated_at = NOW();
SQL
  
  echo -e "  ${GREEN}✓ Mapped to database${NC}"
  echo ""
done

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All tenant groups set up!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Verification
echo "📊 Verification:"
echo ""
echo "Total groups in database:"
psql "$DATABASE_URL" -c "
  SELECT COUNT(*) AS group_count FROM kanidm_tenant_groups;
" -q

echo ""
echo "Groups by tenant:"
psql "$DATABASE_URL" -c "
  SELECT 
    t.slug AS tenant,
    COUNT(*) AS groups,
    string_agg(ktg.role, ', ' ORDER BY ktg.role) AS roles
  FROM kanidm_tenant_groups ktg
  JOIN tenants t ON ktg.tenant_id = t.tenant_id
  GROUP BY t.slug
  ORDER BY t.slug;
" -q

echo ""
echo "Next steps:"
echo "  1. Export users: ./scripts/export-users-for-kanidm.sh"
echo "  2. Migrate users: ./scripts/migrate-users-to-kanidm.sh <export_file.json>"
echo ""
