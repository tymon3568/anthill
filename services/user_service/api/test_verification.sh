#!/bin/bash
# Test verification script for tenant isolation security tests

echo "🔍 Verifying tenant isolation security test setup..."

# Check if environment variables are set
echo "📋 Checking environment variables..."
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL not set"
    echo "   Set with: export DATABASE_URL='postgres://user:pass@localhost:5432/db'"
else
    echo "✅ DATABASE_URL is set: $DATABASE_URL"
fi

if [ -z "$JWT_SECRET" ]; then
    echo "❌ JWT_SECRET not set"
    echo "   Set with: export JWT_SECRET='your-secret-key-at-least-32-characters'"
else
    echo "✅ JWT_SECRET is set (length: ${#JWT_SECRET})"
fi

# Check if test file exists and is valid
echo ""
echo "📁 Checking test file structure..."
if [ -f "tests/security.rs" ]; then
    echo "✅ Test file exists: tests/security.rs"
    
    # Count test functions
    TEST_COUNT=$(grep -c "#\[tokio::test\]" tests/security.rs)
    echo "✅ Found $TEST_COUNT test functions"
    
    # Check ignore attributes
    IGNORED_COUNT=$(grep -c "#\[ignore\]" tests/security.rs)
    echo "✅ $IGNORED_COUNT tests marked with #[ignore] (integration tests)"
    
    # Check environment variable usage
    ENV_USAGE=$(grep -c "std::env::var" tests/security.rs)
    echo "✅ Found $ENV_USAGE environment variable accesses"
    
    # Check helper functions
    HELPER_COUNT=$(grep -c "fn get_test_" tests/security.rs)
    echo "✅ Found $HELPER_COUNT helper functions for environment access"
    
else
    echo "❌ Test file not found: tests/security.rs"
fi

echo ""
echo "🚀 To run the tests:"
echo "   1. Set up PostgreSQL database"
echo "   2. Run migrations: sqlx migrate run"
echo "   3. Run tests: cargo test --test security -- --ignored --test-threads=1"
echo ""
echo "✅ Test setup verification complete!"
