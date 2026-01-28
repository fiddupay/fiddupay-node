#!/bin/bash

# FidduPay API Comprehensive Test Suite
# Tests all merchant and admin endpoints with new route organization

set -e

echo "🧪 FidduPay API Test Suite v2.5.0"
echo "=================================="

# Test configuration
MERCHANT_API_KEY="sk_test_merchant_key_placeholder"
BASE_URL="http://127.0.0.1:8080"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
test_endpoint() {
    local method=$1
    local endpoint=$2
    local auth_header=$3
    local data=$4
    local expected_status=${5:-200}
    
    echo -n "Testing $method $endpoint... "
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -H "$auth_header" \
            -d "$data" \
            "$BASE_URL$endpoint")
    else
        response=$(curl -s -w "%{http_code}" -X "$method" \
            -H "$auth_header" \
            "$BASE_URL$endpoint")
    fi
    
    status_code="${response: -3}"
    response_body="${response%???}"
    
    if [[ "$status_code" == "$expected_status"* ]]; then
        echo -e "${GREEN}✓ PASS${NC} ($status_code)"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC} ($status_code)"
        echo "Response: $response_body"
        ((TESTS_FAILED++))
    fi
}

echo ""
echo "📋 PUBLIC ENDPOINTS"
echo "==================="

test_endpoint "GET" "/health" "" ""
test_endpoint "GET" "/api/v1/status" "" ""
test_endpoint "GET" "/api/v1/currencies/supported" "" ""
test_endpoint "POST" "/api/v1/contact" "" '{"name":"Test","email":"test@example.com","subject":"Test","message":"Test message"}'

echo ""
echo "🔐 ADMIN AUTHENTICATION"
echo "======================="

# Admin login
echo -n "Admin login... "
admin_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"username": "admin", "password": "admin123"}' \
    "$BASE_URL/api/v1/admin/login")

if echo "$admin_response" | jq -e '.session_token' > /dev/null; then
    ADMIN_TOKEN=$(echo "$admin_response" | jq -r '.session_token')
    echo -e "${GREEN}✓ PASS${NC} (Token: ${ADMIN_TOKEN:0:20}...)"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "Response: $admin_response"
    ((TESTS_FAILED++))
    ADMIN_TOKEN="admin_session_placeholder"  # Fallback
fi

echo ""
echo "👑 ADMIN ENDPOINTS"
echo "=================="

test_endpoint "GET" "/api/v1/admin/security/events" "Authorization: Bearer $ADMIN_TOKEN"
test_endpoint "GET" "/api/v1/admin/security/alerts" "Authorization: Bearer $ADMIN_TOKEN"
test_endpoint "GET" "/api/v1/admin/dashboard" "Authorization: Bearer $ADMIN_TOKEN"
test_endpoint "GET" "/api/v1/admin/merchantss" "Authorization: Bearer $ADMIN_TOKEN"
test_endpoint "POST" "/api/v1/admin/logout" "Authorization: Bearer $ADMIN_TOKEN"

echo ""
echo "🏪 MERCHANT ENDPOINTS"
echo "===================="

# Merchant profile
test_endpoint "GET" "/api/v1/merchant/profile" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant payments
test_endpoint "GET" "/api/v1/merchant/payments" "Authorization: Bearer $MERCHANT_API_KEY"
test_endpoint "POST" "/api/v1/merchant/payments" "Authorization: Bearer $MERCHANT_API_KEY" \
    '{"amount_usd":"10.00","crypto_type":"ETH","description":"Test payment"}'

# Merchant analytics
test_endpoint "GET" "/api/v1/merchant/analytics" "Authorization: Bearer $MERCHANT_API_KEY"
test_endpoint "GET" "/api/v1/merchant/analytics?granularity=day" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant balance
test_endpoint "GET" "/api/v1/merchant/balance" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant invoices
test_endpoint "GET" "/api/v1/merchant/invoices" "Authorization: Bearer $MERCHANT_API_KEY"
test_endpoint "POST" "/api/v1/merchant/invoices" "Authorization: Bearer $MERCHANT_API_KEY" \
    '{"amount_usd":"50.00","description":"Test invoice","due_date":"2026-02-01T00:00:00Z"}'

# Merchant refunds
test_endpoint "GET" "/api/v1/merchant/refunds" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant withdrawals
test_endpoint "GET" "/api/v1/merchant/withdrawals" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant wallets
test_endpoint "GET" "/api/v1/merchant/wallets" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant security settings
test_endpoint "GET" "/api/v1/merchant/security/settings" "Authorization: Bearer $MERCHANT_API_KEY"

# Merchant IP whitelist
test_endpoint "GET" "/api/v1/merchant/ip-whitelist" "Authorization: Bearer $MERCHANT_API_KEY"

echo ""
echo "📊 TEST RESULTS"
echo "==============="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ SOME TESTS FAILED${NC}"
    exit 1
fi
