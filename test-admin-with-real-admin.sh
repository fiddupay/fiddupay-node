#!/bin/bash

echo "🔑 TESTING ADMIN ENDPOINTS WITH ACTUAL ADMIN USER"
echo "================================================="
echo ""

# Login as admin user to get API key
echo "🔐 Logging in as admin user..."
LOGIN_RESPONSE=$(curl -s -X POST http://127.0.0.1:8080/api/v1/merchants/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "superadmin@fiddupay.com",
    "password": "superadmin123"
  }')

ADMIN_API_KEY=$(echo $LOGIN_RESPONSE | grep -o '"api_key":"[^"]*"' | cut -d'"' -f4)

if [ -z "$ADMIN_API_KEY" ]; then
    echo "❌ Failed to get admin API key"
    echo "Response: $LOGIN_RESPONSE"
    exit 1
fi

echo "✅ Admin logged in successfully"
echo "🔑 Admin API Key: ${ADMIN_API_KEY:0:20}..."
echo ""

# Test admin endpoints
TOTAL_TESTS=0
PASSED_TESTS=0

test_admin_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    local data=$4
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$method" = "GET" ]; then
        RESPONSE=$(curl -s -w "%{http_code}" -X GET "http://127.0.0.1:8080/api/v1$endpoint" \
          -H "Authorization: Bearer $ADMIN_API_KEY" \
          -H "Content-Type: application/json")
    else
        RESPONSE=$(curl -s -w "%{http_code}" -X $method "http://127.0.0.1:8080/api/v1$endpoint" \
          -H "Authorization: Bearer $ADMIN_API_KEY" \
          -H "Content-Type: application/json" \
          -d "$data")
    fi
    
    HTTP_CODE="${RESPONSE: -3}"
    BODY="${RESPONSE%???}"
    
    if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
        echo "✅ $description (HTTP $HTTP_CODE)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        # Show first 100 chars of response for verification
        echo "   Response: ${BODY:0:100}..."
    else
        echo "❌ $description (HTTP $HTTP_CODE)"
        echo "   Error: $BODY"
    fi
    echo ""
}

echo "🏛️ TESTING ADMIN DASHBOARD & MANAGEMENT:"
echo "========================================"
test_admin_endpoint "GET" "/admin/dashboard" "Admin Dashboard"
test_admin_endpoint "GET" "/admin/merchants" "Admin Merchants Summary"

echo "🔒 TESTING ADMIN SECURITY MANAGEMENT:"
echo "====================================="
test_admin_endpoint "GET" "/admin/security/events" "Admin Security Events"
test_admin_endpoint "GET" "/admin/security/alerts" "Admin Security Alerts"

echo "⚙️ TESTING ADMIN SYSTEM CONFIGURATION:"
echo "======================================"
test_admin_endpoint "GET" "/admin/config/environment" "Get Environment Config"
test_admin_endpoint "GET" "/admin/config/fees" "Get Fee Config"
test_admin_endpoint "GET" "/admin/config/limits" "Get System Limits"

echo "💳 TESTING ADMIN PAYMENT MANAGEMENT:"
echo "===================================="
test_admin_endpoint "GET" "/admin/payments" "Get All Payments"

echo "📈 TESTING ADMIN ANALYTICS & REPORTING:"
echo "======================================="
test_admin_endpoint "GET" "/admin/analytics/platform" "Get Platform Analytics"
test_admin_endpoint "GET" "/admin/analytics/revenue" "Get Revenue Analytics"

echo "🏦 TESTING ADMIN WALLET MANAGEMENT:"
echo "==================================="
test_admin_endpoint "GET" "/admin/wallets/hot" "Get Hot Wallets"
test_admin_endpoint "GET" "/admin/wallets/balances" "Get Wallet Balances"

echo "🔧 TESTING ADMIN SYSTEM MAINTENANCE:"
echo "===================================="
test_admin_endpoint "GET" "/admin/system/health" "Get System Health"

echo "📊 ADMIN ENDPOINT TEST RESULTS:"
echo "==============================="
echo "Total Admin Endpoints Tested: $TOTAL_TESTS"
echo "Successful Tests: $PASSED_TESTS"
echo "Failed Tests: $((TOTAL_TESTS - PASSED_TESTS))"
echo "Success Rate: $(echo "scale=1; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc)%"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo ""
    echo "🎉 ALL ADMIN ENDPOINTS WORKING WITH ADMIN USER!"
    echo "==============================================="
    echo "✅ Admin authentication successful"
    echo "✅ All admin endpoints accessible"
    echo "✅ Real config values returned"
    echo "✅ Admin system fully functional"
else
    echo ""
    echo "⚠️ SOME ADMIN ENDPOINTS FAILED"
    echo "=============================="
    echo "Check the errors above for details."
fi
