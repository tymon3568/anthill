#!/usr/bin/env bash
# Integration Test Database Setup Script
# Sets up a clean test database for integration tests
# Usage: ./scripts/setup-test-db.sh [options]
#
# Options:
#   --reset     Drop and recreate the database
#   --seed      Populate with test fixtures
#   --clean     Remove all test data (keep schema)
#   --help      Show this help message

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default options
RESET=false
SEED=false
CLEAN=false

# Database config
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-anthill}"
DB_PASSWORD="${DB_PASSWORD:-anthill}"
DB_NAME="${DB_NAME:-anthill_test}"
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --reset)
            RESET=true
            shift
            ;;
        --seed)
            SEED=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help)
            echo "Integration Test Database Setup"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --reset    Drop and recreate database"
            echo "  --seed     Populate with test fixtures"
            echo "  --clean    Remove all test data (keep schema)"
            echo "  --help     Show this help"
            echo ""
            echo "Environment Variables:"
            echo "  DB_HOST     Database host (default: localhost)"
            echo "  DB_PORT     Database port (default: 5432)"
            echo "  DB_USER     Database user (default: anthill)"
            echo "  DB_PASSWORD Database password (default: anthill)"
            echo "  DB_NAME     Database name (default: anthill_test)"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Integration Test Database Setup${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""
echo -e "${BLUE}Database: ${DB_NAME}${NC}"
echo -e "${BLUE}Host: ${DB_HOST}:${DB_PORT}${NC}"
echo ""

# Check if PostgreSQL is accessible
if ! command -v psql &> /dev/null; then
    echo -e "${YELLOW}Warning: psql not found. Install PostgreSQL client tools.${NC}"
    echo -e "${YELLOW}Continuing anyway (will use sqlx-cli)...${NC}"
fi

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}sqlx-cli not found. Installing...${NC}"
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Reset database if requested
if [[ "$RESET" == true ]]; then
    echo -e "${YELLOW}Dropping database ${DB_NAME}...${NC}"

    # Try with psql first, fallback to sqlx
    if command -v psql &> /dev/null; then
        PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d postgres -c "DROP DATABASE IF EXISTS ${DB_NAME};" || true
    else
        sqlx database drop -y --database-url "${DATABASE_URL}" || true
    fi

    echo -e "${GREEN}✓ Database dropped${NC}"
    echo ""
fi

# Create database if it doesn't exist
echo -e "${BLUE}Creating database if not exists...${NC}"

if command -v psql &> /dev/null; then
    PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d postgres -c "CREATE DATABASE ${DB_NAME};" 2>/dev/null || echo "Database already exists"
else
    sqlx database create --database-url "${DATABASE_URL}" || echo "Database already exists"
fi

echo -e "${GREEN}✓ Database ready${NC}"
echo ""

# Run migrations
echo -e "${BLUE}Running migrations...${NC}"

export DATABASE_URL="${DATABASE_URL}"
sqlx migrate run --source migrations

echo -e "${GREEN}✓ Migrations complete${NC}"
echo ""

# Clean test data if requested
if [[ "$CLEAN" == true ]]; then
    echo -e "${BLUE}Cleaning test data...${NC}"

    if command -v psql &> /dev/null; then
        PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -c "SELECT cleanup_test_data();"
    else
        sqlx query "SELECT cleanup_test_data();" --database-url "${DATABASE_URL}"
    fi

    echo -e "${GREEN}✓ Test data cleaned${NC}"
    echo ""
fi

# Seed test fixtures if requested
if [[ "$SEED" == true ]]; then
    echo -e "${BLUE}Seeding test fixtures...${NC}"

    # Create test tenant
    TENANT_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')

    if command -v psql &> /dev/null; then
        PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" <<SQL
-- Create test tenant
INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
VALUES (
    '${TENANT_ID}',
    'Test Corporation',
    'test-corp',
    'free',
    'active',
    '{}'::jsonb,
    NOW(),
    NOW()
) ON CONFLICT (slug) DO NOTHING;

-- Generate test users
SELECT generate_test_users('${TENANT_ID}'::uuid, 10);
SQL
    else
        # Fallback to sqlx-cli if psql not available
        echo -e "${YELLOW}psql not found, using sqlx-cli...${NC}"
        sqlx query --database-url "${DATABASE_URL}" \
            "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at) VALUES ('${TENANT_ID}', 'Test Corporation', 'test-corp', 'free', 'active', '{}'::jsonb, NOW(), NOW()) ON CONFLICT (slug) DO NOTHING;" 2>/dev/null || true
        sqlx query --database-url "${DATABASE_URL}" \
            "SELECT generate_test_users('${TENANT_ID}'::uuid, 10);" 2>/dev/null || true
    fi

    echo -e "${GREEN}✓ Test fixtures seeded${NC}"
    echo -e "${BLUE}Test Tenant ID: ${TENANT_ID}${NC}"
    echo ""
fi

# Verify setup
echo -e "${BLUE}Verifying database setup...${NC}"

if command -v psql &> /dev/null; then
    RESULT=$(PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -t -c "SELECT COUNT(*) FROM tenants;")
    echo -e "${GREEN}✓ Database accessible${NC}"
    echo -e "${BLUE}Total tenants: $(echo $RESULT | xargs)${NC}"
else
    echo -e "${GREEN}✓ Database setup complete${NC}"
fi

echo ""
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo -e "${BLUE}Connection String:${NC}"
echo -e "${DATABASE_URL}"
echo ""
echo -e "${BLUE}To run integration tests:${NC}"
echo -e "DATABASE_URL=${DATABASE_URL} cargo test --test integration"
echo ""
echo -e "${BLUE}To clean test data:${NC}"
echo -e "$0 --clean"
