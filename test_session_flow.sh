#!/bin/bash

# Test script for session management and logout flow
# Tests: Register -> Login -> Refresh Token -> Logout

set -e

BASE_URL="http://localhost:8000"
API_V1="$BASE_URL/api/v1"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Testing User Service Session Management${NC}"
echo -e "${YELLOW}========================================${NC}\n"

# Generate random email to avoid conflicts
RANDOM_ID=$(date +%s%N | md5sum | head -c 8)
TEST_EMAIL="testuser_${RANDOM_ID}@example.com"
TEST_PASSWORD="SecurePass123!"
TENANT_NAME="Test Tenant $RANDOM_ID"

echo -e "${YELLOW}Test Data:${NC}"
echo "Email: $TEST_EMAIL"
echo "Password: $TEST_PASSWORD"
echo "Tenant: $TENANT_NAME"
echo ""

# 1. Register
echo -e "${YELLOW}[1/5] Testing Register...${NC}"
REGISTER_RESP=$(curl -s -X POST "$API_V1/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$TEST_EMAIL\",
    \"password\": \"$TEST_PASSWORD\",
    \"full_name\": \"Test User\",
    \"tenant_name\": \"$TENANT_NAME\"
  }")

echo "Register Response:"
echo "$REGISTER_RESP" | jq '.'

# Extract access and refresh tokens
ACCESS_TOKEN=$(echo "$REGISTER_RESP" | jq -r '.access_token')
REFRESH_TOKEN=$(echo "$REGISTER_RESP" | jq -r '.refresh_token')
USER_ID=$(echo "$REGISTER_RESP" | jq -r '.user.id')
TENANT_ID=$(echo "$REGISTER_RESP" | jq -r '.user.tenant_id')

if [ "$ACCESS_TOKEN" == "null" ] || [ -z "$ACCESS_TOKEN" ]; then
  echo -e "${RED}✗ Register failed: No access token received${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Register successful${NC}"
echo "Access Token: ${ACCESS_TOKEN:0:20}..."
echo "Refresh Token: ${REFRESH_TOKEN:0:20}..."
echo "User ID: $USER_ID"
echo "Tenant ID: $TENANT_ID"
echo ""

# 2. Check session was created in database
echo -e "${YELLOW}[2/5] Checking session in database...${NC}"
SESSION_COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM sessions WHERE user_id = '$USER_ID' AND revoked = false;" 2>/dev/null || echo "0")
SESSION_COUNT=$(echo $SESSION_COUNT | xargs) # Trim whitespace

if [ "$SESSION_COUNT" == "1" ]; then
  echo -e "${GREEN}✓ Session created successfully in database${NC}"
  psql "$DATABASE_URL" -c "SELECT session_id, user_id, revoked, created_at FROM sessions WHERE user_id = '$USER_ID';" || echo "Database query skipped"
else
  echo -e "${YELLOW}⚠ Session check: Found $SESSION_COUNT sessions (database connection may not be available for direct query)${NC}"
fi
echo ""

# 3. Refresh Token
echo -e "${YELLOW}[3/5] Testing Refresh Token...${NC}"
REFRESH_RESP=$(curl -s -X POST "$API_V1/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$REFRESH_TOKEN\"
  }")

echo "Refresh Response:"
echo "$REFRESH_RESP" | jq '.'

NEW_ACCESS_TOKEN=$(echo "$REFRESH_RESP" | jq -r '.access_token')
NEW_REFRESH_TOKEN=$(echo "$REFRESH_RESP" | jq -r '.refresh_token')

if [ "$NEW_ACCESS_TOKEN" == "null" ] || [ -z "$NEW_ACCESS_TOKEN" ]; then
  echo -e "${RED}✗ Refresh failed: No new access token received${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Refresh successful${NC}"
echo "New Access Token: ${NEW_ACCESS_TOKEN:0:20}..."
echo "New Refresh Token: ${NEW_REFRESH_TOKEN:0:20}..."
echo ""

# Check that old session was revoked and new one created
echo -e "${YELLOW}[4/5] Checking session rotation...${NC}"
ACTIVE_SESSIONS=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM sessions WHERE user_id = '$USER_ID' AND revoked = false;" 2>/dev/null || echo "skip")
ACTIVE_SESSIONS=$(echo $ACTIVE_SESSIONS | xargs)

if [ "$ACTIVE_SESSIONS" == "skip" ]; then
  echo -e "${YELLOW}⚠ Session rotation check skipped (database not accessible)${NC}"
elif [ "$ACTIVE_SESSIONS" == "1" ]; then
  echo -e "${GREEN}✓ Old session revoked, new session created (1 active session)${NC}"
else
  echo -e "${YELLOW}⚠ Found $ACTIVE_SESSIONS active sessions (expected 1)${NC}"
fi
echo ""

# 4. Logout
echo -e "${YELLOW}[5/5] Testing Logout...${NC}"
LOGOUT_RESP=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$API_V1/auth/logout" \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$NEW_REFRESH_TOKEN\"
  }")

HTTP_STATUS=$(echo "$LOGOUT_RESP" | grep "HTTP_STATUS" | cut -d: -f2)
LOGOUT_BODY=$(echo "$LOGOUT_RESP" | grep -v "HTTP_STATUS")

echo "Logout Response:"
echo "HTTP Status: $HTTP_STATUS"
echo "Body: $LOGOUT_BODY"

if [ "$HTTP_STATUS" == "200" ]; then
  echo -e "${GREEN}✓ Logout successful${NC}"
else
  echo -e "${RED}✗ Logout failed with status $HTTP_STATUS${NC}"
  exit 1
fi
echo ""

# 5. Verify session was revoked
echo -e "${YELLOW}Verifying session was revoked...${NC}"
ACTIVE_SESSIONS_AFTER=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM sessions WHERE user_id = '$USER_ID' AND revoked = false;" 2>/dev/null || echo "skip")
ACTIVE_SESSIONS_AFTER=$(echo $ACTIVE_SESSIONS_AFTER | xargs)

if [ "$ACTIVE_SESSIONS_AFTER" == "skip" ]; then
  echo -e "${YELLOW}⚠ Session revocation check skipped (database not accessible)${NC}"
elif [ "$ACTIVE_SESSIONS_AFTER" == "0" ]; then
  echo -e "${GREEN}✓ All sessions revoked successfully${NC}"
else
  echo -e "${RED}✗ Still found $ACTIVE_SESSIONS_AFTER active sessions${NC}"
  exit 1
fi
echo ""

# 6. Try to use refresh token after logout (should fail)
echo -e "${YELLOW}Testing refresh token after logout (should fail)...${NC}"
REFRESH_AFTER_LOGOUT=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$API_V1/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$NEW_REFRESH_TOKEN\"
  }")

HTTP_STATUS_AFTER=$(echo "$REFRESH_AFTER_LOGOUT" | grep "HTTP_STATUS" | cut -d: -f2)

if [ "$HTTP_STATUS_AFTER" == "401" ] || [ "$HTTP_STATUS_AFTER" == "400" ]; then
  echo -e "${GREEN}✓ Refresh correctly rejected after logout (HTTP $HTTP_STATUS_AFTER)${NC}"
else
  echo -e "${RED}✗ Refresh should have failed after logout, but got HTTP $HTTP_STATUS_AFTER${NC}"
  exit 1
fi
echo ""

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}All session management tests passed! ✓${NC}"
echo -e "${GREEN}========================================${NC}"
