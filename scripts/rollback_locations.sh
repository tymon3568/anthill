#!/bin/bash
# ==================================
# Location Architecture Rollback Script
# ==================================
# Module: 4.5 - Location Architecture Fix
# Description: Rollback the location architecture migration
# Created: 2026-01-28
#
# This script reverts the location architecture changes:
# 1. Restores storage_locations table from backup
# 2. Reverts migrations
# 3. Verifies the rollback
#
# IMPORTANT: This script requires a backup file from migrate_locations.sh
# ==================================

set -e  # Exit on error
set -o pipefail  # Fail on pipe errors

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BACKUP_DIR="${BACKUP_DIR:-./backups}"
DATABASE_URL="${DATABASE_URL:-postgres://localhost/inventory_saas}"

# Migration version to revert to (before location architecture changes)
# This is the migration BEFORE 20260128300001_unify_location_tables_add_columns.sql
TARGET_MIGRATION_VERSION="20260128200002"

echo -e "${RED}========================================${NC}"
echo -e "${RED}Location Architecture ROLLBACK Script${NC}"
echo -e "${RED}========================================${NC}"
echo ""

# Function to log with timestamp
log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to check if a table exists
table_exists() {
    psql "$DATABASE_URL" -tAc "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '$1');" | grep -q 't'
}

# Check for backup file argument
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 <backup_file>${NC}"
    echo ""
    echo "Available backup files:"
    ls -la "$BACKUP_DIR"/backup_before_location_migration_*.sql 2>/dev/null || echo "No backup files found in $BACKUP_DIR"
    echo ""
    exit 1
fi

BACKUP_FILE="$1"

if [ ! -f "$BACKUP_FILE" ]; then
    echo -e "${RED}ERROR: Backup file not found: $BACKUP_FILE${NC}"
    exit 1
fi

log "Using backup file: $BACKUP_FILE"

# Confirm rollback
echo ""
echo -e "${RED}WARNING: This will revert the location architecture migration!${NC}"
echo -e "${RED}This action will:${NC}"
echo -e "${RED}  1. Revert database migrations${NC}"
echo -e "${RED}  2. Restore storage_locations table${NC}"
echo -e "${RED}  3. Remove new columns from warehouse_locations${NC}"
echo -e "${RED}  4. Remove zone/location columns from stock_transfer_items${NC}"
echo ""
read -p "Are you sure you want to proceed? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    log "Rollback cancelled by user"
    exit 0
fi

# Step 1: Pre-rollback snapshot
log "${YELLOW}Step 1: Taking pre-rollback snapshot...${NC}"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
ROLLBACK_SNAPSHOT="${BACKUP_DIR}/pre_rollback_snapshot_${TIMESTAMP}.sql"

pg_dump "$DATABASE_URL" -t warehouse_locations -t inventory_levels -t stock_moves -t stock_transfer_items > "$ROLLBACK_SNAPSHOT"
log "Pre-rollback snapshot saved to: ${ROLLBACK_SNAPSHOT}"

# Step 2: Revert migrations
log "${YELLOW}Step 2: Reverting migrations...${NC}"

# Check if sqlx is available
if ! command -v sqlx &> /dev/null; then
    echo -e "${RED}ERROR: sqlx CLI not found. Install with: cargo install sqlx-cli${NC}"
    exit 1
fi

# Revert migrations one by one (in reverse order)
MIGRATIONS_TO_REVERT=(
    "20260128300005_add_warehouse_zones_tenant_unique"
    "20260128300004_add_transfer_item_locations"
    "20260128300003_unify_location_tables_drop_old"
    "20260128300002_unify_location_tables_migrate_data"
    "20260128300001_unify_location_tables_add_columns"
)

for MIGRATION in "${MIGRATIONS_TO_REVERT[@]}"; do
    log "Reverting: $MIGRATION"
    sqlx migrate revert || {
        echo -e "${YELLOW}WARNING: Could not revert $MIGRATION (may already be reverted)${NC}"
    }
done

log "${GREEN}Migrations reverted${NC}"

# Step 3: Restore storage_locations from backup
log "${YELLOW}Step 3: Restoring storage_locations from backup...${NC}"

# Check if storage_locations needs to be restored
if ! table_exists "storage_locations"; then
    log "Restoring storage_locations table from backup..."
    psql "$DATABASE_URL" < "$BACKUP_FILE"
    log "${GREEN}✓ storage_locations restored${NC}"
else
    log "${YELLOW}storage_locations already exists, skipping restore${NC}"
fi

# Step 4: Verify rollback
log "${YELLOW}Step 4: Verifying rollback...${NC}"

# Check storage_locations exists
if table_exists "storage_locations"; then
    STORAGE_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM storage_locations WHERE deleted_at IS NULL;")
    log "${GREEN}✓ storage_locations restored with ${STORAGE_COUNT} records${NC}"
else
    echo -e "${RED}ERROR: storage_locations table not restored${NC}"
    exit 1
fi

# Check new columns are removed from warehouse_locations
NEW_COLUMNS_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'warehouse_locations' AND column_name IN ('aisle', 'rack', 'level', 'position', 'capacity', 'current_stock', 'is_quarantine', 'is_picking_location');")

if [ "$NEW_COLUMNS_COUNT" -eq 0 ]; then
    log "${GREEN}✓ New columns removed from warehouse_locations${NC}"
else
    log "${YELLOW}WARNING: $NEW_COLUMNS_COUNT new columns still exist in warehouse_locations${NC}"
fi

# Check zone/location columns are removed from stock_transfer_items
TRANSFER_COLUMNS_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'stock_transfer_items' AND column_name IN ('source_zone_id', 'source_location_id', 'destination_zone_id', 'destination_location_id');")

if [ "$TRANSFER_COLUMNS_COUNT" -eq 0 ]; then
    log "${GREEN}✓ Zone/location columns removed from stock_transfer_items${NC}"
else
    log "${YELLOW}WARNING: $TRANSFER_COLUMNS_COUNT zone/location columns still exist in stock_transfer_items${NC}"
fi

# Step 5: Summary
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Rollback Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
log "Summary:"
log "  - Pre-rollback snapshot: ${ROLLBACK_SNAPSHOT}"
log "  - Backup restored: ${BACKUP_FILE}"
log "  - storage_locations: RESTORED"
log "  - Migrations reverted: 5"
echo ""
echo -e "${YELLOW}IMPORTANT: Review the application and test thoroughly!${NC}"
echo -e "${GREEN}Rollback completed successfully!${NC}"
