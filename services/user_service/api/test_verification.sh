#!/bin/bash
# Test verification script for tenant isolation security tests
# This script validates the test environment and fails CI if setup is incorrect

set -e  # Exit on any error

# Initialize error tracking
HAS_ERRORS=0

echo "🔍 Verifying tenant isolation security test setup..."
echo ""

# Function to increment error count and print error message
error() {
    echo "❌ ERROR: $1"
    HAS_ERRORS=$((HAS_ERRORS + 1))
}

# Function to print success message
success() {
    echo "✅ $1"
}

# Function to find the project root (repository root)
find_project_root() {
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/Cargo.toml" ]] && [[ -d "$dir/services/user_service/api" ]]; then
            echo "$dir"
            return 0
        fi
        dir=$(dirname "$dir")
    done
    return 1
}

# Function to find test file path robustly
find_test_file() {
    local project_root
    project_root=$(find_project_root)

    if [[ -n "$project_root" ]]; then
        local test_file="$project_root/services/user_service/api/tests/security.rs"
        if [[ -f "$test_file" ]]; then
            echo "$test_file"
            return 0
        fi
    fi

    # Fallback: try relative paths from current directory
    for test_path in "tests/security.rs" "../tests/security.rs" "../../tests/security.rs"; do
        if [[ -f "$test_path" ]]; then
            echo "$(realpath "$test_path")"
            return 0
        fi
    done

    return 1
}

# Check environment variables securely
echo "📋 Checking environment variables..."

if [ -z "$DATABASE_URL" ]; then
    error "DATABASE_URL environment variable is not set"
    echo "   Set with: export DATABASE_URL='postgres://user:pass@localhost:5432/db'"
else
    success "DATABASE_URL is set (credentials not displayed for security)"
fi

if [ -z "$JWT_SECRET" ]; then
    error "JWT_SECRET environment variable is not set"
    echo "   Set with: export JWT_SECRET='your-secret-key-at-least-32-characters'"
elif [ ${#JWT_SECRET} -lt 32 ]; then
    error "JWT_SECRET is too short (${#JWT_SECRET} characters). Must be at least 32 characters for security"
    echo "   Current length: ${#JWT_SECRET} characters (minimum: 32)"
else
    success "JWT_SECRET is set and meets length requirements (${#JWT_SECRET} characters)"
fi

# Check test file structure with validation
echo ""
echo "📁 Validating test file structure..."

TEST_FILE=$(find_test_file)

if [[ -z "$TEST_FILE" ]]; then
    error "Test file tests/security.rs not found"
    echo "   Searched from: $(find_project_root 2>/dev/null || echo "unknown")"
    echo "   Make sure you're running this script from the repository root or services/user_service/api directory"
else
    success "Test file found: $(basename "$TEST_FILE")"

    # Count and validate test functions
    TEST_COUNT=$(grep -c "#\[tokio::test\]" "$TEST_FILE")
    if [ "$TEST_COUNT" -eq 0 ]; then
        error "No test functions found in $TEST_FILE"
        echo "   Expected at least 1 test function with #[tokio::test]"
    elif [ "$TEST_COUNT" -lt 3 ]; then
        error "Insufficient test functions found ($TEST_COUNT). Expected at least 3 security tests"
        echo "   Found: $TEST_COUNT test functions"
    else
        success "Found $TEST_COUNT test functions"
    fi

    # Check ignore attributes (integration tests)
    IGNORED_COUNT=$(grep -c "#\[ignore\]" "$TEST_FILE")
    if [ "$IGNORED_COUNT" -eq 0 ]; then
        error "No integration tests found (missing #[ignore] attributes)"
        echo "   All tests should be marked with #[ignore] for CI integration"
    elif [ "$IGNORED_COUNT" -ne "$TEST_COUNT" ]; then
        error "Mismatch between test count and ignore count"
        echo "   Tests: $TEST_COUNT, Ignored: $IGNORED_COUNT (should be equal)"
    else
        success "$IGNORED_COUNT tests marked with #[ignore] (integration tests)"
    fi

    # Check environment variable usage
    ENV_USAGE=$(grep -c "std::env::var" "$TEST_FILE")
    if [ "$ENV_USAGE" -eq 0 ]; then
        error "No environment variable usage found in tests"
        echo "   Tests should use std::env::var for DATABASE_URL and JWT_SECRET"
    elif [ "$ENV_USAGE" -lt 2 ]; then
        error "Insufficient environment variable usage ($ENV_USAGE). Expected at least 2 usages"
        echo "   Found: $ENV_USAGE environment variable accesses"
    else
        success "Found $ENV_USAGE environment variable accesses"
    fi

    # Check helper functions
    HELPER_COUNT=$(grep -c "fn get_test_" "$TEST_FILE")
    if [ "$HELPER_COUNT" -eq 0 ]; then
        error "No helper functions found for environment access"
        echo "   Expected helper functions like get_test_database_url() and get_test_jwt_secret()"
    elif [ "$HELPER_COUNT" -lt 2 ]; then
        error "Insufficient helper functions ($HELPER_COUNT). Expected at least 2"
        echo "   Found: $HELPER_COUNT helper functions"
    else
        success "Found $HELPER_COUNT helper functions for environment access"
    fi
fi

# Final validation and exit
echo ""
if [ $HAS_ERRORS -gt 0 ]; then
    echo "🚨 VALIDATION FAILED: $HAS_ERRORS error(s) found!"
    echo ""
    echo "🔧 To fix the issues:"
    echo "   1. Set required environment variables:"
    echo "      export DATABASE_URL='postgres://user:pass@localhost:5432/db'"
    echo "      export JWT_SECRET='your-secret-key-at-least-32-characters-long'"
    echo "   2. Ensure test file exists and has proper structure"
    echo "   3. Run this script from the repository root or services/user_service/api directory"
    echo ""
    echo "❌ Test setup verification failed!"
    exit 1
else
    echo "🎉 VALIDATION SUCCESS: All checks passed!"
    echo ""
    echo "🚀 Ready to run tests:"
    echo "   cd services/user_service/api"
    echo "   cargo test --test security -- --ignored --test-threads=1"
    echo ""
    echo "✅ Test setup verification complete!"
    exit 0
fi
