#!/usr/bin/env bash
# E2E Test Runner Script
# Runs Playwright E2E tests against the user service
# Usage: ./scripts/run-e2e-tests.sh [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
TENANT_ID="${TENANT_ID:-00000000-0000-0000-0000-000000000001}"
E2E_DIR="tests/e2e"

# Parse arguments
PROJECT=""
HEADED=false
DEBUG=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --api)
            PROJECT="api"
            shift
            ;;
        --flows)
            PROJECT="flows"
            shift
            ;;
        --security)
            PROJECT="security"
            shift
            ;;
        --headed)
            HEADED=true
            shift
            ;;
        --debug)
            DEBUG=true
            shift
            ;;
        --help)
            echo "E2E Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --api       Run API tests only"
            echo "  --flows     Run flow tests only"
            echo "  --security  Run security tests only"
            echo "  --headed    Run tests in headed mode (visible browser)"
            echo "  --debug     Run tests in debug mode"
            echo "  --help      Show this help"
            echo ""
            echo "Environment variables:"
            echo "  BASE_URL    Server URL (default: http://localhost:3000)"
            echo "  TENANT_ID   Tenant ID for tests"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       E2E Test Runner                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Check if npm dependencies are installed
if [ ! -d "$E2E_DIR/node_modules" ]; then
    echo -e "${YELLOW}Installing E2E test dependencies...${NC}"
    cd "$E2E_DIR"
    npm install
    npx playwright install
    cd - > /dev/null
    echo -e "${GREEN}✓ Dependencies installed${NC}"
    echo ""
fi

# Export environment variables
export BASE_URL
export TENANT_ID

echo -e "${BLUE}Configuration:${NC}"
echo "  Base URL: $BASE_URL"
echo "  Tenant ID: $TENANT_ID"
echo ""

# Build the command
CMD="npx playwright test"

if [ -n "$PROJECT" ]; then
    CMD="$CMD --project=$PROJECT"
    echo -e "${YELLOW}Running project: $PROJECT${NC}"
fi

if [ "$HEADED" = true ]; then
    CMD="$CMD --headed"
    echo -e "${YELLOW}Running in headed mode${NC}"
fi

if [ "$DEBUG" = true ]; then
    CMD="$CMD --debug"
    echo -e "${YELLOW}Running in debug mode${NC}"
fi

echo ""
echo -e "${BLUE}Starting E2E tests...${NC}"
echo ""

# Run tests
cd "$E2E_DIR"
EXIT_CODE=0
$CMD || EXIT_CODE=$?
cd - > /dev/null

echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}║   ✓ E2E tests passed!                 ║${NC}"
else
    echo -e "${RED}║   ✗ Some E2E tests failed             ║${NC}"
fi

echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Show next steps
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${BLUE}Next steps:${NC}"
    echo "  • View report: cd $E2E_DIR && npx playwright show-report"
    echo "  • Run specific test: $0 --api"
else
    echo -e "${YELLOW}To debug:${NC}"
    echo "  • Run in debug mode: $0 --debug"
    echo "  • Run in headed mode: $0 --headed"
    echo "  • View report: cd $E2E_DIR && npx playwright show-report"
fi

echo ""

exit $EXIT_CODE
