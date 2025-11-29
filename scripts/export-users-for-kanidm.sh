#!/bin/bash
# Export users from PostgreSQL for Kanidm migration
# Usage: ./export-users-for-kanidm.sh [output_file]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DATABASE_URL=${DATABASE_URL:-"postgresql://anthill:anthill@localhost:5432/anthill"}
OUTPUT_FILE=${1:-"users_export_$(date +%Y%m%d_%H%M%S).json"}

echo -e "${GREEN}📊 Exporting Users from PostgreSQL${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Database: $DATABASE_URL"
echo "Output:   $OUTPUT_FILE"
echo ""

# Check if database is accessible
if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
  echo -e "${RED}❌ Error: Cannot connect to database${NC}"
  echo "Check DATABASE_URL: $DATABASE_URL"
  exit 1
fi

# Export users as JSON
echo -e "${YELLOW}Querying database...${NC}"

psql "$DATABASE_URL" -t -A -o "$OUTPUT_FILE" <<'SQL'
SELECT json_agg(row_to_json(u))::text
FROM (
  SELECT
    u.user_id::text,
    u.tenant_id::text,
    t.slug AS tenant_slug,
    t.name AS tenant_name,
    u.email,
    u.full_name,
    u.phone,
    u.role,
    u.status,
    u.email_verified,
    u.last_login_at,
    u.created_at,
    (u.kanidm_user_id IS NOT NULL) AS has_kanidm,
    u.kanidm_user_id::text AS kanidm_user_id,
    u.auth_method
  FROM users u
  JOIN tenants t ON u.tenant_id = t.tenant_id
  WHERE u.deleted_at IS NULL
    AND u.status = 'active'
    AND u.kanidm_user_id IS NULL  -- Only export users NOT yet in Kanidm
  ORDER BY t.slug, u.email
) u;
SQL

# Check if export succeeded
if [ ! -f "$OUTPUT_FILE" ]; then
  echo -e "${RED}❌ Error: Export failed${NC}"
  exit 1
fi

# Validate JSON
if ! jq empty "$OUTPUT_FILE" 2>/dev/null; then
  echo -e "${RED}❌ Error: Invalid JSON output${NC}"
  exit 1
fi

# Count exported users
USER_COUNT=$(jq '. | length' "$OUTPUT_FILE")

if [ "$USER_COUNT" == "null" ] || [ -z "$USER_COUNT" ]; then
  echo -e "${YELLOW}⚠️  No users to export (all users already in Kanidm)${NC}"
  rm -f "$OUTPUT_FILE"
  exit 0
fi

# Statistics
echo ""
echo -e "${GREEN}✅ Export Complete!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Statistics:"
echo "  Total users exported: $USER_COUNT"
echo ""

# Breakdown by tenant
echo "📋 Users by tenant:"
jq -r '.[] | "\(.tenant_slug): \(.email)"' "$OUTPUT_FILE" | \
  awk -F: '{print $1}' | sort | uniq -c | \
  awk '{printf "  %-20s %3d users\n", $2, $1}'

echo ""
echo "📁 Output file: $OUTPUT_FILE"
echo ""

# Show sample (first 3 users)
echo "📄 Sample (first 3 users):"
jq -r '.[:3][] | "  - \(.email) (\(.tenant_slug)) - \(.role)"' "$OUTPUT_FILE"

if [ "$USER_COUNT" -gt 3 ]; then
  echo "  ... and $((USER_COUNT - 3)) more"
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Next steps:"
echo "  1. Review the exported file: jq . $OUTPUT_FILE"
echo "  2. Setup Kanidm groups: ./scripts/setup-kanidm-tenant-groups.sh"
echo "  3. Migrate users: ./scripts/migrate-users-to-kanidm.sh $OUTPUT_FILE"
echo ""
