#!/bin/bash

# =============================================================================
# Integration Test Setup Script for Anthill Frontend
# Task: 08.02.05 - Tenant Context & OIDC Flow Testing
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKEND_PORT=8000
FRONTEND_PORT=5173
POSTGRES_PORT=5432
TEST_TENANTS=("acme" "demo")

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Anthill Integration Test Setup${NC}"
echo -e "${BLUE}  Task: 08.02.05 - Tenant Context Testing${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# -----------------------------------------------------------------------------
# Step 1: Check Prerequisites
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[Step 1/6] Checking prerequisites...${NC}"

check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed${NC}"
        return 1
    else
        echo -e "${GREEN}✓ $1 found${NC}"
        return 0
    fi
}

MISSING_DEPS=0
check_command "docker" || MISSING_DEPS=1
check_command "docker-compose" || check_command "docker" || MISSING_DEPS=1
check_command "cargo" || MISSING_DEPS=1
check_command "bun" || MISSING_DEPS=1
check_command "curl" || MISSING_DEPS=1

if [ $MISSING_DEPS -eq 1 ]; then
    echo -e "${RED}Please install missing dependencies and try again.${NC}"
    exit 1
fi

echo ""

# -----------------------------------------------------------------------------
# Step 2: Setup /etc/hosts for subdomain testing
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[Step 2/6] Setting up /etc/hosts for subdomain testing...${NC}"

HOSTS_UPDATED=0
for tenant in "${TEST_TENANTS[@]}"; do
    if ! grep -q "${tenant}.localhost" /etc/hosts; then
        echo -e "${YELLOW}Adding ${tenant}.localhost to /etc/hosts (requires sudo)${NC}"
        echo "127.0.0.1 ${tenant}.localhost" | sudo tee -a /etc/hosts > /dev/null
        HOSTS_UPDATED=1
    else
        echo -e "${GREEN}✓ ${tenant}.localhost already in /etc/hosts${NC}"
    fi
done

if [ $HOSTS_UPDATED -eq 1 ]; then
    echo -e "${GREEN}✓ /etc/hosts updated${NC}"
fi

echo ""

# -----------------------------------------------------------------------------
# Step 3: Update .env with CORS for tenant subdomains
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[Step 3/6] Configuring CORS for tenant subdomains...${NC}"

cd "$PROJECT_ROOT"

# Build CORS origins string
CORS_ORIGINS="http://localhost:${FRONTEND_PORT}"
for tenant in "${TEST_TENANTS[@]}"; do
    CORS_ORIGINS="${CORS_ORIGINS},http://${tenant}.localhost:${FRONTEND_PORT}"
done

# Update .env file
if [ -f .env ]; then
    # Check if CORS_ORIGINS exists
    if grep -q "^CORS_ORIGINS=" .env; then
        # Update existing CORS_ORIGINS
        sed -i "s|^CORS_ORIGINS=.*|CORS_ORIGINS=${CORS_ORIGINS}|" .env
        echo -e "${GREEN}✓ Updated CORS_ORIGINS in .env${NC}"
    else
        # Add CORS_ORIGINS
        echo "CORS_ORIGINS=${CORS_ORIGINS}" >> .env
        echo -e "${GREEN}✓ Added CORS_ORIGINS to .env${NC}"
    fi
else
    echo -e "${RED}❌ .env file not found. Creating from example...${NC}"
    if [ -f .env.example ]; then
        cp .env.example .env
        echo "CORS_ORIGINS=${CORS_ORIGINS}" >> .env
    else
        echo "DATABASE_URL=postgres://user:password@localhost:5432/inventory_db" > .env
        echo "PORT=8000" >> .env
        echo "CORS_ORIGINS=${CORS_ORIGINS}" >> .env
    fi
    echo -e "${GREEN}✓ Created .env with CORS_ORIGINS${NC}"
fi

echo -e "${BLUE}CORS_ORIGINS=${CORS_ORIGINS}${NC}"
echo ""

# -----------------------------------------------------------------------------
# Step 4: Start Docker services (PostgreSQL, Redis, etc.)
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[Step 4/6] Starting Docker services...${NC}"

cd "$PROJECT_ROOT/infra/docker_compose"

# Check if services are already running
if docker ps | grep -q postgres_db; then
    echo -e "${GREEN}✓ PostgreSQL already running${NC}"
else
    echo "Starting PostgreSQL, Redis, NATS, MinIO..."
    docker-compose up -d postgres redis nats minio

    # Wait for PostgreSQL to be ready
    echo "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if docker exec postgres_db pg_isready -U user -d inventory_db > /dev/null 2>&1; then
            echo -e "${GREEN}✓ PostgreSQL is ready${NC}"
            break
        fi
        sleep 1
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ PostgreSQL failed to start${NC}"
            exit 1
        fi
    done
fi

echo ""

# -----------------------------------------------------------------------------
# Step 5: Run database migrations and seed test tenant
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[Step 5/6] Running migrations and seeding test data...${NC}"

cd "$PROJECT_ROOT"

# Run migrations
echo "Running database migrations..."
if command -v sqlx &> /dev/null; then
    sqlx migrate run 2>/dev/null || echo -e "${YELLOW}⚠ sqlx migrate failed (may already be up to date)${NC}"
else
    echo -e "${YELLOW}⚠ sqlx-cli not installed. Migrations may need to be run manually.${NC}"
fi

# Seed test tenants
echo "Seeding test tenants..."
for tenant in "${TEST_TENANTS[@]}"; do
    # Check if tenant exists
    TENANT_EXISTS=$(docker exec postgres_db psql -U user -d inventory_db -t -c "SELECT COUNT(*) FROM tenants WHERE slug = '${tenant}';" 2>/dev/null | tr -d ' ' || echo "0")

    if [ "$TENANT_EXISTS" = "0" ] || [ -z "$TENANT_EXISTS" ]; then
        echo "Creating tenant: ${tenant}"
        docker exec postgres_db psql -U user -d inventory_db -c "
            INSERT INTO tenants (tenant_id, name, slug, created_at, updated_at)
            VALUES (gen_random_uuid(), '${tenant^} Corp', '${tenant}', NOW(), NOW())
            ON CONFLICT (slug) DO NOTHING;
        " 2>/dev/null || echo -e "${YELLOW}⚠ Could not create tenant ${tenant} (table may not exist yet)${NC}"
    else
        echo -e "${GREEN}✓ Tenant ${tenant} already exists${NC}"
    fi
done

echo ""

# -----------------------------------------------------------------------------
# Step 6: Print test instructions
# -----------------------------------------------------------------------------
echo -e "${BLUE}============================================${NC}"
echo -e "${GREEN}  Setup Complete! ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""
echo -e "${YELLOW}Next steps to run integration tests:${NC}"
echo ""
echo -e "${BLUE}Terminal 1 - Start Backend:${NC}"
echo "  cd $PROJECT_ROOT"
echo "  cargo run --bin user-service"
echo ""
echo -e "${BLUE}Terminal 2 - Start Frontend:${NC}"
echo "  cd $PROJECT_ROOT/frontend"
echo "  bun run dev"
echo ""
echo -e "${BLUE}Terminal 3 - Run Tests:${NC}"
echo "  # Wait for both services to start, then run:"
echo "  cd $PROJECT_ROOT"
echo "  ./scripts/run-integration-tests.sh"
echo ""
echo -e "${YELLOW}Manual Test URLs:${NC}"
echo "  • http://localhost:${FRONTEND_PORT}/login"
for tenant in "${TEST_TENANTS[@]}"; do
    echo "  • http://${tenant}.localhost:${FRONTEND_PORT}/login"
done
echo ""
echo -e "${YELLOW}Test Scenarios:${NC}"
echo "  1. Login at localhost:${FRONTEND_PORT} → Should show Organization input"
echo "  2. Login at acme.localhost:${FRONTEND_PORT} → Should auto-detect 'acme' tenant"
echo "  3. Click 'Switch organization' → Should allow changing tenant"
echo ""
echo -e "${GREEN}Ready for testing!${NC}"
