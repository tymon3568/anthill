#!/bin/bash
# ==================================
# Location Architecture Migration Script
# ==================================
# Module: 4.5 - Location Architecture Fix
# Description: Migrate storage_locations to warehouse_locations
# Created: 2026-01-28
#
# This script performs the location architecture unification:
# 1. Backs up current data
# 2. Runs migrations in order
# 3. Verifies the migration
#
# IMPORTANT: Run this script on staging first before production!
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
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/backup_before_location_migration_${TIMESTAMP}.sql"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Location Architecture Migration Script${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Function to log with timestamp
log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to check if a table exists
table_exists() {
    psql "$DATABASE_URL" -tAc "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '$1');" | grep -q 't'
}

# Step 1: Pre-migration checks
log "${YELLOW}Step 1: Pre-migration checks...${NC}"

if ! table_exists "warehouse_locations"; then
    echo -e "${RED}ERROR: warehouse_locations table does not exist${NC}"
    exit 1
fi

if table_exists "storage_locations"; then
    STORAGE_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM storage_locations WHERE deleted_at IS NULL;")
    log "Found ${STORAGE_COUNT} active records in storage_locations"
else
    log "storage_locations table does not exist (already migrated?)"
    STORAGE_COUNT=0
fi

WAREHOUSE_LOCATIONS_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM warehouse_locations WHERE deleted_at IS NULL;")
log "Found ${WAREHOUSE_LOCATIONS_COUNT} active records in warehouse_locations"

# Step 2: Backup current data
log "${YELLOW}Step 2: Backing up current data...${NC}"

if table_exists "storage_locations"; then
    pg_dump "$DATABASE_URL" -t storage_locations -t warehouse_locations -t inventory_levels -t stock_moves > "$BACKUP_FILE"
    log "Backup saved to: ${BACKUP_FILE}"
else
    pg_dump "$DATABASE_URL" -t warehouse_locations -t inventory_levels -t stock_moves > "$BACKUP_FILE"
    log "Backup saved to: ${BACKUP_FILE} (without storage_locations)"
fi

# Step 3: Run migrations
log "${YELLOW}Step 3: Running migrations...${NC}"

# Check if sqlx is available
if ! command -v sqlx &> /dev/null; then
    echo -e "${RED}ERROR: sqlx CLI not found. Install with: cargo install sqlx-cli${NC}"
    exit 1
fi

# Run pending migrations
sqlx migrate run

log "${GREEN}Migrations completed successfully${NC}"

# Step 4: Verify migration
log "${YELLOW}Step 4: Verifying migration...${NC}"

# Check warehouse_locations has new columns
NEW_COLUMNS_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'warehouse_locations' AND column_name IN ('aisle', 'rack', 'level', 'position', 'capacity', 'current_stock', 'is_quarantine', 'is_picking_location');")

if [ "$NEW_COLUMNS_COUNT" -lt 8 ]; then
    echo -e "${RED}ERROR: warehouse_locations is missing new columns (found $NEW_COLUMNS_COUNT/8)${NC}"
    exit 1
fi
log "${GREEN}✓ warehouse_locations has all new columns${NC}"

# Check stock_transfer_items has zone/location columns
TRANSFER_COLUMNS_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'stock_transfer_items' AND column_name IN ('source_zone_id', 'source_location_id', 'destination_zone_id', 'destination_location_id');")

if [ "$TRANSFER_COLUMNS_COUNT" -lt 4 ]; then
    echo -e "${RED}ERROR: stock_transfer_items is missing zone/location columns (found $TRANSFER_COLUMNS_COUNT/4)${NC}"
    exit 1
fi
log "${GREEN}✓ stock_transfer_items has zone/location columns${NC}"

# Check storage_locations is dropped
if table_exists "storage_locations"; then
    echo -e "${YELLOW}WARNING: storage_locations table still exists${NC}"
else
    log "${GREEN}✓ storage_locations table has been removed${NC}"
fi

# Final count
FINAL_WAREHOUSE_LOCATIONS_COUNT=$(psql "$DATABASE_URL" -tAc "SELECT COUNT(*) FROM warehouse_locations WHERE deleted_at IS NULL;")
log "Final warehouse_locations count: ${FINAL_WAREHOUSE_LOCATIONS_COUNT}"

# Step 5: Summary
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Migration Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
log "Summary:"
log "  - Backup file: ${BACKUP_FILE}"
log "  - warehouse_locations: ${FINAL_WAREHOUSE_LOCATIONS_COUNT} records"
log "  - New columns added to warehouse_locations: 8"
log "  - New columns added to stock_transfer_items: 4"
if ! table_exists "storage_locations"; then
    log "  - storage_locations: REMOVED"
fi
echo ""
echo -e "${GREEN}Migration completed successfully!${NC}"
