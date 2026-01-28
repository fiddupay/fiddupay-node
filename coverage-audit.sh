#!/bin/bash

echo " ENDPOINT COVERAGE AUDIT"
echo "=========================="

echo ""
echo " MERCHANT API TEST FILE COVERAGE:"
echo "===================================="

# Count merchant endpoints in test file
MERCHANT_TESTS=$(grep -c "axios\." /home/vibes/crypto-payment-gateway/tests/merchant-api-comprehensive.js)
echo "Merchant API tests: $MERCHANT_TESTS"

echo ""
echo " ADMIN API TEST FILE COVERAGE:"
echo "================================="

# Count admin endpoints in test file  
ADMIN_TESTS=$(grep -c "axios\." /home/vibes/crypto-payment-gateway/tests/admin-api-comprehensive.js)
echo "Admin API tests: $ADMIN_TESTS"

echo ""
echo " SANDBOX API TEST FILE COVERAGE:"
echo "==================================="

# Count sandbox endpoints in test file
SANDBOX_TESTS=$(grep -c "axios\." /home/vibes/crypto-payment-gateway/tests/sandbox-api-comprehensive.js)
echo "Sandbox API tests: $SANDBOX_TESTS"

echo ""
echo " SDK COMPREHENSIVE TEST COVERAGE:"
echo "===================================="

# Count SDK tests
SDK_TESTS=$(grep -c "Testing.*\.\.\." /home/vibes/crypto-payment-gateway/tests/sdk-comprehensive.js)
echo "SDK tests: $SDK_TESTS"

echo ""
echo " EXPECTED COVERAGE:"
echo "===================="
echo "Public endpoints: 9"
echo "Merchant endpoints: 43"
echo "Admin endpoints: 5"
echo "Sandbox endpoints: 2"
echo "Total: 57"

echo ""
echo " MISSING ENDPOINTS ANALYSIS:"
echo "=============================="

# Check for missing public endpoints
echo " PUBLIC ENDPOINTS (should be in merchant test):"
echo "- /health"
echo "- /pay/:link_id"
echo "- /pay/:link_id/status"
echo "- /api/v1/merchant/register"
echo "- /api/v1/merchant/login"
echo "- /api/v1/currencies/supported"
echo "- /api/v1/status"
echo "- /api/v1/blog"
echo "- /api/v1/careers"

echo ""
echo " ADMIN ENDPOINTS (should be in admin test):"
grep "^/api/v1/admin" /tmp/all_endpoints.txt

echo ""
echo " SANDBOX ENDPOINTS (should be in sandbox test):"
grep "^/api/v1/sandbox" /tmp/all_endpoints.txt
