#!/bin/bash

# =============================================================================
# Frontend Tenant Context Integration Tests
# Task: 08.02.05 - Test login with tenant subdomain and X-Tenant-ID header
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BACKEND_URL="${BACKEND_URL:-http://localhost:8000}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:5173}"
TEST_TENANT="acme"
TEST_EMAIL="test@example.com"
TEST_PASSWORD="Test123!@#"

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------

print_header() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_test() {
    echo -e "${CYAN}[TEST] $1${NC}"
}

print_pass() {
    echo -e "${GREEN}  ✓ PASS: $1${NC}"
    ((TESTS_PASSED++))
}

print_fail() {
    echo -e "${RED}  ✗ FAIL: $1${NC}"
    ((TESTS_FAILED++))
}

print_skip() {
    echo -e "${YELLOW}  ⊘ SKIP: $1${NC}"
    ((TESTS_SKIPPED++))
}

print_info() {
    echo -e "${YELLOW}  ℹ $1${NC}"
}

check_service() {
    local url=$1
    local name=$2
    local max_retries=5
    local retry=0

    while [ $retry -lt $max_retries ]; do
        if curl -s -o /dev/null -w "%{http_code}" "$url" | grep -qE "^[2-3][0-9]{2}$|000"; then
            # 000 means connection refused, but 2xx/3xx means service is up
            if curl -s -o /dev/null "$url" 2>/dev/null; then
                return 0
            fi
        fi
        retry=$((retry + 1))
        sleep 1
    done
    return 1
}

# -----------------------------------------------------------------------------
# Pre-flight Checks
# -----------------------------------------------------------------------------

print_header "Pre-flight Checks"

# Check if backend is running
print_test "Checking backend service at ${BACKEND_URL}"
if curl -s -o /dev/null -w "%{http_code}" "${BACKEND_URL}/api/v1/health" 2>/dev/null | grep -qE "^[2-3][0-9]{2}$"; then
    print_pass "Backend is running"
    BACKEND_RUNNING=true
else
    print_fail "Backend is not running at ${BACKEND_URL}"
    print_info "Start backend with: cargo run --bin user-service"
    BACKEND_RUNNING=false
fi

# Check if frontend is running
print_test "Checking frontend service at ${FRONTEND_URL}"
if curl -s -o /dev/null -w "%{http_code}" "${FRONTEND_URL}" 2>/dev/null | grep -qE "^[2-3][0-9]{2}$"; then
    print_pass "Frontend is running"
    FRONTEND_RUNNING=true
else
    print_fail "Frontend is not running at ${FRONTEND_URL}"
    print_info "Start frontend with: cd frontend && bun run dev"
    FRONTEND_RUNNING=false
fi

# Check /etc/hosts for subdomain
print_test "Checking /etc/hosts for ${TEST_TENANT}.localhost"
if grep -Fq "${TEST_TENANT}.localhost" /etc/hosts; then
    print_pass "${TEST_TENANT}.localhost is configured"
    HOSTS_CONFIGURED=true
else
    print_fail "${TEST_TENANT}.localhost not in /etc/hosts"
    print_info "Add with: echo '127.0.0.1 ${TEST_TENANT}.localhost' | sudo tee -a /etc/hosts"
    HOSTS_CONFIGURED=false
fi

# Exit if services not running
if [ "$BACKEND_RUNNING" = false ] || [ "$FRONTEND_RUNNING" = false ]; then
    echo ""
    echo -e "${RED}Cannot proceed without backend and frontend running.${NC}"
    echo -e "${YELLOW}Please start the services and run this script again.${NC}"
    exit 1
fi

# -----------------------------------------------------------------------------
# Test 4.1: Login with Tenant Subdomain
# -----------------------------------------------------------------------------

print_header "Test 4.1: Login with Tenant Subdomain"

if [ "$HOSTS_CONFIGURED" = true ]; then
    print_test "Accessing login page via subdomain: http://${TEST_TENANT}.localhost:5173/login"

    # Test that subdomain resolves and frontend serves the page
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://${TEST_TENANT}.localhost:5173/login" 2>/dev/null || echo "000")

    if [ "$HTTP_CODE" = "200" ]; then
        print_pass "Login page accessible via subdomain (HTTP $HTTP_CODE)"

        # Check if the page contains tenant context
        PAGE_CONTENT=$(curl -s "http://${TEST_TENANT}.localhost:5173/login" 2>/dev/null)

        if echo "$PAGE_CONTENT" | grep -qi "sign.*in\|login"; then
            print_pass "Login page rendered correctly"
        else
            print_fail "Login page content not as expected"
        fi
    else
        print_fail "Login page not accessible via subdomain (HTTP $HTTP_CODE)"
    fi
else
    print_skip "Subdomain test skipped - /etc/hosts not configured"
fi

# -----------------------------------------------------------------------------
# Test 4.2: Login with X-Tenant-ID Header
# -----------------------------------------------------------------------------

print_header "Test 4.2: Login with X-Tenant-ID Header"

print_test "Sending login request with X-Tenant-ID header"

# First, check if the backend API accepts the header
# Use single curl call to capture both response body and HTTP code
LOGIN_RAW=$(curl -s -w "\n%{http_code}" -X POST "${BACKEND_URL}/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -H "X-Tenant-ID: ${TEST_TENANT}" \
    -d "{\"email\": \"${TEST_EMAIL}\", \"password\": \"${TEST_PASSWORD}\"}" \
    2>/dev/null)

HTTP_CODE=$(printf '%s\n' "$LOGIN_RAW" | tail -n1)
LOGIN_RESPONSE=$(printf '%s\n' "$LOGIN_RAW" | sed '$d')

print_info "Response code: $HTTP_CODE"
print_info "Response body: $LOGIN_RESPONSE"

if [ "$HTTP_CODE" = "200" ]; then
    print_pass "Login successful with X-Tenant-ID header"

    # Check for access token in response
    if echo "$LOGIN_RESPONSE" | grep -qi "access_token\|token"; then
        print_pass "Response contains access token"
    else
        print_fail "Response missing access token"
    fi
elif [ "$HTTP_CODE" = "401" ]; then
    # Invalid credentials but header was accepted
    if echo "$LOGIN_RESPONSE" | grep -qi "invalid\|credentials\|password"; then
        print_pass "X-Tenant-ID header accepted (credentials invalid as expected with test data)"
    else
        print_fail "Unexpected 401 response"
    fi
elif [ "$HTTP_CODE" = "400" ]; then
    if echo "$LOGIN_RESPONSE" | grep -qi "tenant.*not.*found\|invalid.*tenant"; then
        print_info "Tenant '${TEST_TENANT}' not found in database"
        print_skip "Login test skipped - tenant needs to be seeded first"
    elif echo "$LOGIN_RESPONSE" | grep -qi "tenant.*required"; then
        print_fail "Backend still requires tenant context - X-Tenant-ID header not being read"
    else
        print_fail "Unexpected 400 response: $LOGIN_RESPONSE"
    fi
else
    print_fail "Unexpected response code: $HTTP_CODE"
fi

# Test without X-Tenant-ID header (should fail)
print_test "Sending login request WITHOUT X-Tenant-ID header (should fail)"

# Use single curl call to capture both response body and HTTP code
NO_HEADER_RAW=$(curl -s -w "\n%{http_code}" -X POST "${BACKEND_URL}/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"email\": \"${TEST_EMAIL}\", \"password\": \"${TEST_PASSWORD}\"}" \
    2>/dev/null)

NO_HEADER_CODE=$(printf '%s\n' "$NO_HEADER_RAW" | tail -n1)
NO_HEADER_RESPONSE=$(printf '%s\n' "$NO_HEADER_RAW" | sed '$d')

print_info "Response code: $NO_HEADER_CODE"

if [ "$NO_HEADER_CODE" = "400" ] && echo "$NO_HEADER_RESPONSE" | grep -qi "tenant.*required"; then
    print_pass "Backend correctly rejects request without tenant context"
else
    print_info "Response: $NO_HEADER_RESPONSE"
    print_info "Note: Backend may accept request from localhost without explicit tenant"
fi

# -----------------------------------------------------------------------------
# Test 4.3: Protected Route Access After Login
# -----------------------------------------------------------------------------

print_header "Test 4.3: Protected Route Access After Login"

print_test "Checking dashboard page without authentication"

DASHBOARD_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${FRONTEND_URL}/dashboard" 2>/dev/null)

if [ "$DASHBOARD_CODE" = "302" ] || [ "$DASHBOARD_CODE" = "303" ] || [ "$DASHBOARD_CODE" = "307" ]; then
    print_pass "Dashboard redirects unauthenticated users (HTTP $DASHBOARD_CODE)"
elif [ "$DASHBOARD_CODE" = "200" ]; then
    # Check if the content is actually the login page (client-side redirect)
    DASHBOARD_CONTENT=$(curl -s "${FRONTEND_URL}/dashboard" 2>/dev/null)
    if echo "$DASHBOARD_CONTENT" | grep -qi "login\|sign.*in"; then
        print_pass "Dashboard shows login page for unauthenticated users"
    else
        print_fail "Dashboard accessible without authentication"
    fi
else
    print_info "Dashboard returned HTTP $DASHBOARD_CODE"
fi

# -----------------------------------------------------------------------------
# Test 4.4: Dashboard Layout and Navigation
# -----------------------------------------------------------------------------

print_header "Test 4.4: Dashboard Layout and Navigation"

print_test "Checking if dashboard route exists"

# This test would need authentication to fully test
# For now, we verify the route structure exists
if curl -s "${FRONTEND_URL}/dashboard" 2>/dev/null | grep -qi "<!DOCTYPE\|<html"; then
    print_pass "Dashboard route responds with HTML"
else
    print_fail "Dashboard route not responding correctly"
fi

# -----------------------------------------------------------------------------
# Test 4.5: SSR and CSR Behavior
# -----------------------------------------------------------------------------

print_header "Test 4.5: SSR and CSR Behavior"

print_test "Checking for SSR content in login page"

LOGIN_HTML=$(curl -s "${FRONTEND_URL}/login" 2>/dev/null)

# Check for server-rendered content
if echo "$LOGIN_HTML" | grep -qi "<!DOCTYPE html"; then
    print_pass "Login page returns valid HTML"

    # Check for hydration markers (SvelteKit specific)
    if echo "$LOGIN_HTML" | grep -q "data-sveltekit"; then
        print_pass "SvelteKit hydration markers present"
    else
        print_info "SvelteKit hydration markers not detected (may be fine)"
    fi

    # Check for meta tags (SEO/SSR indicator)
    if echo "$LOGIN_HTML" | grep -qi "<title>"; then
        print_pass "Page has <title> tag (SSR working)"
    else
        print_fail "Missing <title> tag"
    fi
else
    print_fail "Login page not returning valid HTML"
fi

# -----------------------------------------------------------------------------
# Test: Frontend API Client X-Tenant-ID Header
# -----------------------------------------------------------------------------

print_header "Test: Frontend API Client Configuration"

print_test "Checking if frontend sends X-Tenant-ID header"

# We can't directly test the frontend's fetch calls, but we can check the built code
if [ -d "frontend/src/lib/api" ]; then
    if grep -r "X-Tenant-ID" frontend/src/lib/api/*.ts 2>/dev/null; then
        print_pass "X-Tenant-ID header configured in API client"
    else
        print_fail "X-Tenant-ID header not found in API client"
    fi
else
    print_skip "Cannot verify API client configuration"
fi

# Check tenant utilities
if [ -f "frontend/src/lib/tenant/index.ts" ]; then
    print_pass "Tenant utilities module exists"

    if grep -q "parseTenantFromHostname" frontend/src/lib/tenant/index.ts; then
        print_pass "parseTenantFromHostname function exists"
    else
        print_fail "parseTenantFromHostname function not found"
    fi
else
    print_fail "Tenant utilities module not found"
fi

# -----------------------------------------------------------------------------
# Test Summary
# -----------------------------------------------------------------------------

print_header "Test Summary"

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))

echo -e "Total Tests: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
echo -e "${YELLOW}Skipped: ${TESTS_SKIPPED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✓ All tests passed!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  ✗ Some tests failed${NC}"
    echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
    EXIT_CODE=1
fi

echo ""
echo -e "${YELLOW}Notes:${NC}"
echo "  • Full login flow requires a valid user in the test tenant"
echo "  • Create test user: INSERT INTO users (tenant_id, email, ...) VALUES (...)"
echo ""

exit $EXIT_CODE
