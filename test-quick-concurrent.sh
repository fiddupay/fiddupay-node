#!/bin/bash

# Quick Concurrent Test
echo "🔬 Quick Concurrent Test"
echo "======================="

BASE_URL="http://localhost:8080"

# Test 1: Single request (should work)
echo "1. Testing single request..."
# Don't restart backend, just test against running instance

RESPONSE1=$(curl -s -X POST $BASE_URL/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test1@test.com","business_name":"Test 1","password":"TestPassword123!"}')

KEY1=$(echo $RESPONSE1 | jq -r '.api_key // "FAILED"')
echo "Request 1: $KEY1"

# Test 2: Second request (should work now)
echo "2. Testing second request..."
RESPONSE2=$(curl -s -X POST $BASE_URL/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test2@test.com","business_name":"Test 2","password":"TestPassword123!"}')

KEY2=$(echo $RESPONSE2 | jq -r '.api_key // "TIMEOUT"')
echo "Request 2: $KEY2"

# Results
if [[ $KEY1 == sk_* ]]; then
    echo " First request: SUCCESS"
else
    echo " First request: FAILED"
fi

if [[ $KEY2 == sk_* ]]; then
    echo " Second request: SUCCESS - Backend handles concurrent requests!"
elif [[ $KEY2 == "TIMEOUT" ]]; then
    echo " Second request: TIMEOUT - Backend hangs after first request"
else
    echo " Second request: FAILED - $KEY2"
fi

echo ""
echo " DIAGNOSIS:"
if [[ $KEY1 == sk_* && $KEY2 == sk_* ]]; then
    echo " Backend handles multiple requests correctly"
elif [[ $KEY1 == sk_* && $KEY2 == "TIMEOUT" ]]; then
    echo " Backend has concurrency issue - hangs after first request"
    echo "   This is a critical production issue that needs fixing"
else
    echo " Backend has fundamental issues"
fi
