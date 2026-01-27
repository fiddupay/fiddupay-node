#!/bin/bash

# FidduPay SDK - Complete Test Suite
# This script runs all tests to verify the SDK is working correctly

echo "🚀 FidduPay SDK - Complete Test Suite"
echo "======================================"

# Check if backend is running
echo "🔍 Checking backend server..."
if curl -s http://127.0.0.1:8080/health > /dev/null; then
    echo "✅ Backend server is running"
else
    echo "❌ Backend server is not running. Please start it first:"
    echo "   cd backend && cargo run"
    exit 1
fi

# Build SDK
echo ""
echo "🔨 Building SDK..."
cd fiddupay-node-sdk
npm run build
if [ $? -eq 0 ]; then
    echo "✅ SDK built successfully"
else
    echo "❌ SDK build failed"
    exit 1
fi

cd ../sandbox

# Run basic validation test
echo ""
echo "1️⃣ Running Basic Validation Test..."
echo "-----------------------------------"
node validated-test-v2.2.js
BASIC_RESULT=$?

# Run comprehensive test
echo ""
echo "2️⃣ Running Comprehensive Test..."
echo "--------------------------------"
node comprehensive-test.js
COMPREHENSIVE_RESULT=$?

# Run final validation test
echo ""
echo "3️⃣ Running Final Validation Test..."
echo "----------------------------------"
node final-validation.js
FINAL_RESULT=$?

# Summary
echo ""
echo "📊 Test Suite Summary"
echo "===================="

if [ $BASIC_RESULT -eq 0 ]; then
    echo "✅ Basic Validation: PASSED"
else
    echo "❌ Basic Validation: FAILED"
fi

if [ $COMPREHENSIVE_RESULT -eq 0 ]; then
    echo "✅ Comprehensive Test: PASSED"
else
    echo "❌ Comprehensive Test: FAILED"
fi

if [ $FINAL_RESULT -eq 0 ]; then
    echo "✅ Final Validation: PASSED"
else
    echo "❌ Final Validation: FAILED"
fi

# Overall result
if [ $BASIC_RESULT -eq 0 ] && [ $COMPREHENSIVE_RESULT -eq 0 ] && [ $FINAL_RESULT -eq 0 ]; then
    echo ""
    echo "🎉 ALL TESTS PASSED! FidduPay SDK is fully functional and production-ready."
    echo ""
    echo "✅ Features Verified:"
    echo "   • Merchant Registration"
    echo "   • Authentication"
    echo "   • Wallet Configuration"
    echo "   • Payment Creation"
    echo "   • Payment Retrieval"
    echo "   • Payment Listing"
    echo "   • Analytics"
    echo "   • Error Handling"
    echo "   • Multiple Crypto Types"
    echo ""
    exit 0
else
    echo ""
    echo "⚠️  Some tests failed. Please review the output above."
    exit 1
fi
