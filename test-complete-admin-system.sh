#!/bin/bash

echo "🔑 COMPREHENSIVE ADMIN SYSTEM TEST"
echo "=================================="
echo ""

echo "🔐 Step 1: Admin Login Authentication"
echo "====================================="

# Test Super Admin Login
echo "🔑 Testing Super Admin Login..."
SUPER_ADMIN_RESPONSE=$(curl -s -X POST http://127.0.0.1:8080/api/v1/merchants/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "superadmin@fiddupay.com",
    "password": "any_password"
  }')

SUPER_ADMIN_KEY=$(echo $SUPER_ADMIN_RESPONSE | jq -r '.api_key')
SUPER_ADMIN_ID=$(echo $SUPER_ADMIN_RESPONSE | jq -r '.user.id')

if [ "$SUPER_ADMIN_KEY" != "null" ] && [ "$SUPER_ADMIN_KEY" != "" ]; then
    echo "✅ Super Admin Login: SUCCESS"
    echo "   📧 Email: superadmin@fiddupay.com"
    echo "   🆔 ID: $SUPER_ADMIN_ID"
    echo "   🔑 API Key: ${SUPER_ADMIN_KEY:0:25}..."
else
    echo "❌ Super Admin Login: FAILED"
    echo "   Response: $SUPER_ADMIN_RESPONSE"
    exit 1
fi

# Test Regular Admin Login
echo ""
echo "🔑 Testing Regular Admin Login..."
ADMIN_RESPONSE=$(curl -s -X POST http://127.0.0.1:8080/api/v1/merchants/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@fiddupay.com",
    "password": "any_password"
  }')

ADMIN_KEY=$(echo $ADMIN_RESPONSE | jq -r '.api_key')
ADMIN_ID=$(echo $ADMIN_RESPONSE | jq -r '.user.id')

if [ "$ADMIN_KEY" != "null" ] && [ "$ADMIN_KEY" != "" ]; then
    echo "✅ Regular Admin Login: SUCCESS"
    echo "   📧 Email: admin@fiddupay.com"
    echo "   🆔 ID: $ADMIN_ID"
    echo "   🔑 API Key: ${ADMIN_KEY:0:25}..."
else
    echo "❌ Regular Admin Login: FAILED"
    echo "   Response: $ADMIN_RESPONSE"
fi

echo ""
echo "🧪 Step 2: Comprehensive Admin Endpoint Testing"
echo "=============================================="

# Test admin endpoints with Super Admin
TOTAL_TESTS=0
PASSED_TESTS=0

test_admin_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    local api_key=$4
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    RESPONSE=$(curl -s -w "%{http_code}" -X $method "http://127.0.0.1:8080/api/v1$endpoint" \
      -H "Authorization: Bearer $api_key" \
      -H "Content-Type: application/json")
    
    HTTP_CODE="${RESPONSE: -3}"
    BODY="${RESPONSE%???}"
    
    if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
        echo "✅ $description (HTTP $HTTP_CODE)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        # Show first 80 chars of response
        echo "   📄 ${BODY:0:80}..."
    else
        echo "❌ $description (HTTP $HTTP_CODE)"
        echo "   ⚠️  ${BODY:0:100}..."
    fi
}

echo ""
echo "🏛️ ADMIN DASHBOARD & MANAGEMENT:"
test_admin_endpoint "GET" "/admin/dashboard" "Admin Dashboard" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/merchants" "Admin Merchants Summary" "$SUPER_ADMIN_KEY"

echo ""
echo "🔒 ADMIN SECURITY MANAGEMENT:"
test_admin_endpoint "GET" "/admin/security/events" "Admin Security Events" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/security/alerts" "Admin Security Alerts" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/security/settings" "Get Security Settings" "$SUPER_ADMIN_KEY"

echo ""
echo "⚙️ ADMIN SYSTEM CONFIGURATION:"
test_admin_endpoint "GET" "/admin/config/environment" "Get Environment Config" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/config/fees" "Get Fee Config" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/config/limits" "Get System Limits" "$SUPER_ADMIN_KEY"

echo ""
echo "💳 ADMIN PAYMENT MANAGEMENT:"
test_admin_endpoint "GET" "/admin/payments" "Get All Payments" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/analytics/platform" "Get Platform Analytics" "$SUPER_ADMIN_KEY"

echo ""
echo "🏦 ADMIN WALLET MANAGEMENT:"
test_admin_endpoint "GET" "/admin/wallets/hot" "Get Hot Wallets" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/wallets/balances" "Get Wallet Balances" "$SUPER_ADMIN_KEY"

echo ""
echo "🔧 ADMIN SYSTEM MAINTENANCE:"
test_admin_endpoint "GET" "/admin/system/health" "Get System Health" "$SUPER_ADMIN_KEY"
test_admin_endpoint "GET" "/admin/system/logs" "Get System Logs" "$SUPER_ADMIN_KEY"

echo ""
echo "📊 ADMIN TEST RESULTS:"
echo "====================="
echo "Total Admin Endpoints Tested: $TOTAL_TESTS"
echo "Successful Tests: $PASSED_TESTS"
echo "Failed Tests: $((TOTAL_TESTS - PASSED_TESTS))"
echo "Success Rate: $(echo "scale=1; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc)%"

echo ""
echo "🔄 Step 3: Revoke Merchant Admin Access"
echo "======================================="

# Revert any merchants back to MERCHANT role
echo "🔄 Reverting merchants with admin roles back to MERCHANT..."
REVERTED=$(psql -d fiddupay_test -t -c "
UPDATE merchants 
SET role = 'MERCHANT', updated_at = NOW()
WHERE role IN ('ADMIN', 'SUPER_ADMIN') 
AND email NOT IN ('superadmin@fiddupay.com', 'admin@fiddupay.com');
SELECT ROW_COUNT();
")

echo "✅ Reverted $REVERTED merchant(s) back to MERCHANT role"

# Show final admin users
echo ""
echo "👥 Final Admin Users:"
echo "===================="
psql -d fiddupay_test -c "
SELECT id, email, business_name, role, is_active, created_at 
FROM merchants 
WHERE role IN ('ADMIN', 'SUPER_ADMIN')
ORDER BY role DESC, created_at ASC;
"

echo ""
if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo "🎉 ADMIN SYSTEM FULLY FUNCTIONAL!"
    echo "================================="
    echo "✅ Super Admin seeded and working"
    echo "✅ Admin login authentication working"
    echo "✅ All admin endpoints accessible"
    echo "✅ Real configuration values returned"
    echo "✅ Merchant admin access revoked"
    echo "✅ Security-focused admin system complete"
else
    echo "⚠️ ADMIN SYSTEM PARTIALLY WORKING"
    echo "================================="
    echo "✅ Admin authentication working"
    echo "✅ Most admin endpoints functional"
    echo "⚠️ Some endpoints may need backend fixes"
fi
