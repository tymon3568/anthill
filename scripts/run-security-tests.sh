#!/bin/bash
# Security Testing Script for User Service
# This script runs all comprehensive security tests

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  User Service Security Testing${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Check for required environment variables
if [ -z "$DATABASE_URL" ]; then
    echo -e "${YELLOW}WARNING: DATABASE_URL not set. Using default.${NC}"
    export DATABASE_URL="postgres://anthill:anthill@localhost:5432/anthill_test"
fi

if [ -z "$JWT_SECRET" ]; then
    echo -e "${YELLOW}WARNING: JWT_SECRET not set. Using default.${NC}"
    export JWT_SECRET="test-secret-key-at-least-32-characters-long"
fi

echo -e "${GREEN}Environment Variables:${NC}"
echo "  DATABASE_URL: $DATABASE_URL"
echo "  JWT_SECRET: [HIDDEN]"
echo ""

# Check if database is accessible
echo -e "${BLUE}Checking database connection...${NC}"
if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${RED}ERROR: Cannot connect to database${NC}"
    echo -e "${YELLOW}Make sure PostgreSQL is running:${NC}"
    echo "  docker-compose -f infra/docker_compose/docker-compose.yml up -d postgres"
    exit 1
fi
echo -e "${GREEN}✓ Database connection successful${NC}"
echo ""

# Function to run a test suite
run_test_suite() {
    local test_name=$1
    local test_file=$2

    echo -e "${BLUE}Running: ${test_name}${NC}"
    echo "=========================================="

    if cargo test --package user_service_api --test "$test_file" -- --ignored --test-threads=1 --nocapture 2>&1; then
        echo -e "${GREEN}✓ ${test_name} PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ ${test_name} FAILED${NC}"
        return 1
    fi
    echo ""
}

# Track results
total_suites=0
passed_suites=0
failed_suites=0

# Test Suite 1: Tenant Isolation
total_suites=$((total_suites + 1))
if run_test_suite "Tenant Isolation Tests" "tenant_isolation_tests"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites=$((failed_suites + 1))
fi

# Test Suite 2: RBAC Security
total_suites=$((total_suites + 1))
if run_test_suite "RBAC Security Tests" "rbac_security_tests"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites=$((failed_suites + 1))
fi

# Test Suite 3: JWT & Session Security
total_suites=$((total_suites + 1))
if run_test_suite "JWT & Session Security Tests" "jwt_session_security_tests"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites=$((failed_suites + 1))
fi

# Test Suite 4: SQL Injection Prevention
total_suites=$((total_suites + 1))
if run_test_suite "SQL Injection Prevention Tests" "sql_injection_tests"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites=$((failed_suites + 1))
fi

# Print summary
echo ""
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  Security Test Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo "Total Test Suites: $total_suites"
echo -e "${GREEN}Passed: $passed_suites${NC}"
if [ $failed_suites -gt 0 ]; then
    echo -e "${RED}Failed: $failed_suites${NC}"
else
    echo -e "Failed: $failed_suites"
fi
echo ""

# Calculate success rate
success_rate=$((passed_suites * 100 / total_suites))
echo "Success Rate: ${success_rate}%"
echo ""

# Exit with appropriate code
if [ $failed_suites -eq 0 ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  ALL SECURITY TESTS PASSED! ✓${NC}"
    echo -e "${GREEN}========================================${NC}"
    exit 0
else
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}  SOME SECURITY TESTS FAILED! ✗${NC}"
    echo -e "${RED}========================================${NC}"
    exit 1
fi
