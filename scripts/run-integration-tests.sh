#!/usr/bin/env bash
# Integration Test Runner
# Manages test database and runs integration tests
# Usage: ./scripts/run-integration-tests.sh [options]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default options
SETUP_DB=true
TEARDOWN_DB=false
CLEANUP_DATA=true
VERBOSE=false
TEST_FILTER=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-setup)
            SETUP_DB=false
            shift
            ;;
        --teardown)
            TEARDOWN_DB=true
            shift
            ;;
        --no-cleanup)
            CLEANUP_DATA=false
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --filter)
            TEST_FILTER="$2"
            shift 2
            ;;
        --help)
            echo "Integration Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --no-setup    Skip database setup"
            echo "  --teardown    Stop and remove test containers after tests"
            echo "  --no-cleanup  Don't cleanup test data after tests"
            echo "  --verbose     Show detailed test output"
            echo "  --filter      Run specific test (e.g., test_user_registration)"
            echo "  --help        Show this help"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Integration Test Runner             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Setup test database
if [[ "$SETUP_DB" == true ]]; then
    echo -e "${BLUE}[1/5] Setting up test database...${NC}"

    # Start Docker containers
    if ! docker-compose -f docker-compose.test.yml ps | grep -q "postgres-test.*Up"; then
        echo -e "${YELLOW}Starting test containers...${NC}"
        docker-compose -f docker-compose.test.yml up -d

        # Wait for PostgreSQL to be ready
        echo -e "${YELLOW}Waiting for PostgreSQL...${NC}"
        timeout 30 bash -c 'until docker-compose -f docker-compose.test.yml exec -T postgres-test pg_isready -U anthill -d anthill_test; do sleep 1; done'

        echo -e "${GREEN}✓ Test containers started${NC}"
    else
        echo -e "${GREEN}✓ Test containers already running${NC}"
    fi

    # Run migrations
    echo -e "${YELLOW}Running migrations...${NC}"
    export DATABASE_URL="postgres://anthill:anthill@localhost:5433/anthill_test"

    if ! command -v sqlx &> /dev/null; then
        echo -e "${YELLOW}Installing sqlx-cli...${NC}"
        cargo install sqlx-cli --no-default-features --features postgres
    fi

    sqlx migrate run --source migrations

    echo -e "${GREEN}✓ Database setup complete${NC}"
    echo ""
fi

# Set environment variables for tests
export DATABASE_URL="postgres://anthill:anthill@localhost:5433/anthill_test"
export TEST_DATABASE_URL="postgres://anthill:anthill@localhost:5433/anthill_test"
export JWT_SECRET="test-secret-key-at-least-32-characters-long"
export RUST_LOG="${RUST_LOG:-info}"
export RUSTFLAGS="--cfg uuid_unstable"

echo -e "${BLUE}[2/5] Building test binaries...${NC}"

# Build tests
if [[ "$VERBOSE" == true ]]; then
    cargo test --no-run --package user_service_api
else
    cargo test --no-run --package user_service_api --quiet
fi

echo -e "${GREEN}✓ Test binaries built${NC}"
echo ""

echo -e "${BLUE}[3/5] Running integration tests...${NC}"
echo ""

# Determine which tests to run
TEST_CMD="cargo test --package user_service_api --test"

if [[ -n "$TEST_FILTER" ]]; then
    echo -e "${YELLOW}Running filtered tests: ${TEST_FILTER}${NC}"
    TEST_CMD="$TEST_CMD -- --ignored $TEST_FILTER"
else
    echo -e "${YELLOW}Running all integration tests${NC}"
fi

# Run tests
TEST_EXIT_CODE=0

if [[ "$VERBOSE" == true ]]; then
    # Run individual test files with detailed output
    echo -e "${BLUE}Running API Endpoint Tests...${NC}"
    cargo test --package user_service_api --test api_endpoint_tests -- --ignored --nocapture || TEST_EXIT_CODE=$?

    echo ""
    echo -e "${BLUE}Running Auth Flow Tests...${NC}"
    cargo test --package user_service_api --test auth_flow_tests -- --ignored --nocapture || TEST_EXIT_CODE=$?

    echo ""
    echo -e "${BLUE}Running Error Handling Tests...${NC}"
    cargo test --package user_service_api --test error_handling_tests -- --ignored --nocapture || TEST_EXIT_CODE=$?

    echo ""
    echo -e "${BLUE}Running Integration Tests...${NC}"
    cargo test --package user_service_api --test integration_tests -- --ignored --nocapture || TEST_EXIT_CODE=$?
else
    # Run all tests quietly
    cargo test --package user_service_api --test api_endpoint_tests -- --ignored || TEST_EXIT_CODE=$?
    cargo test --package user_service_api --test auth_flow_tests -- --ignored || TEST_EXIT_CODE=$?
    cargo test --package user_service_api --test error_handling_tests -- --ignored || TEST_EXIT_CODE=$?
    cargo test --package user_service_api --test integration_tests -- --ignored || TEST_EXIT_CODE=$?
fi

echo ""

# Cleanup test data
if [[ "$CLEANUP_DATA" == true ]]; then
    echo -e "${BLUE}[4/5] Cleaning up test data...${NC}"

    docker-compose -f docker-compose.test.yml exec -T postgres-test psql -U anthill -d anthill_test -c "SELECT cleanup_test_data();" || true

    echo -e "${GREEN}✓ Test data cleaned${NC}"
    echo ""
fi

# Teardown if requested
if [[ "$TEARDOWN_DB" == true ]]; then
    echo -e "${BLUE}[5/5] Tearing down test environment...${NC}"

    docker-compose -f docker-compose.test.yml down -v

    echo -e "${GREEN}✓ Test environment removed${NC}"
    echo ""
fi

# Report results
echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"

if [[ $TEST_EXIT_CODE -eq 0 ]]; then
    echo -e "${GREEN}║   ✓ All tests passed!                 ║${NC}"
else
    echo -e "${RED}║   ✗ Some tests failed                 ║${NC}"
fi

echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Show next steps
if [[ $TEST_EXIT_CODE -eq 0 ]]; then
    echo -e "${BLUE}Next steps:${NC}"
    echo -e "  • Review test coverage: ${YELLOW}./scripts/coverage.sh${NC}"
    echo -e "  • Run security tests: ${YELLOW}./scripts/run-security-tests.sh${NC}"
    echo -e "  • Stop test containers: ${YELLOW}docker-compose -f docker-compose.test.yml down${NC}"
else
    echo -e "${YELLOW}To debug failing tests:${NC}"
    echo -e "  • Run with verbose output: ${YELLOW}$0 --verbose${NC}"
    echo -e "  • Run specific test: ${YELLOW}$0 --filter test_name${NC}"
    echo -e "  • Check logs: ${YELLOW}docker-compose -f docker-compose.test.yml logs${NC}"
fi

echo ""

exit $TEST_EXIT_CODE
