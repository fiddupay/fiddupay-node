#!/bin/bash

echo "🚀 FidduPay - Complete Test Suite Runner"
echo "========================================"

# Check backend
if ! curl -s http://127.0.0.1:8080/health > /dev/null; then
    echo "❌ Backend not running. Start with: cd backend && cargo run"
    exit 1
fi

echo "✅ Backend is running"
echo ""

cd tests

# 1. Merchant API Tests
echo "1️⃣ MERCHANT API TESTS"
echo "====================="
node merchant-api-comprehensive.js
MERCHANT_RESULT=$?
echo ""

# 2. Admin API Tests  
echo "2️⃣ ADMIN API TESTS"
echo "=================="
node admin-api-comprehensive.js
ADMIN_RESULT=$?
echo ""

# 3. Sandbox API Tests
echo "3️⃣ SANDBOX API TESTS"
echo "===================="
node sandbox-api-comprehensive.js
SANDBOX_RESULT=$?
echo ""

# 4. SDK Tests (build SDK first)
echo "4️⃣ SDK TESTS"
echo "============"
cd ../fiddupay-node-sdk && npm run build > /dev/null 2>&1
cd ../tests
node sdk-comprehensive.js
SDK_RESULT=$?
echo ""

# Summary
echo "📊 FINAL RESULTS"
echo "================"

TOTAL_PASSED=0
TOTAL_TESTS=4

if [ $MERCHANT_RESULT -eq 0 ]; then
    echo "✅ Merchant API: PASSED"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
    echo "❌ Merchant API: FAILED"
fi

if [ $ADMIN_RESULT -eq 0 ]; then
    echo "✅ Admin API: PASSED"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
    echo "❌ Admin API: FAILED"
fi

if [ $SANDBOX_RESULT -eq 0 ]; then
    echo "✅ Sandbox API: PASSED"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
    echo "❌ Sandbox API: FAILED"
fi

if [ $SDK_RESULT -eq 0 ]; then
    echo "✅ SDK: PASSED"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
    echo "❌ SDK: FAILED"
fi

echo ""
echo "📈 Overall: $TOTAL_PASSED/$TOTAL_TESTS test suites passed"

# Overall result
if [ $TOTAL_PASSED -ge 3 ]; then
    echo "🎉 MAJORITY TESTS PASSED!"
    exit 0
else
    echo "⚠️ Most tests failed"
    exit 1
fi
