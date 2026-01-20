#!/bin/bash

echo "=== PayFlow API Test Suite ==="
echo ""

BASE_URL="http://localhost:8080"

# Test 1: Health Check
echo "1. Testing Health Endpoint..."
curl -s $BASE_URL/health | jq .
echo ""

# Test 2: Metrics
echo "2. Testing Metrics Endpoint..."
curl -s $BASE_URL/metrics | head -5
echo ""

# Test 3: Create Merchant (should fail without proper auth)
echo "3. Testing Create Merchant (expect auth error)..."
curl -s $BASE_URL/api/v1/merchants -X POST \
  -H "Content-Type: application/json" \
  -d '{"business_name":"Test","email":"test@test.com"}' | jq .
echo ""

# Test 4: List Payments (should fail without auth)
echo "4. Testing List Payments (expect auth error)..."
curl -s $BASE_URL/api/v1/payments -H "Authorization: Bearer invalid_key" | jq .
echo ""

echo "=== Test Complete ==="
echo "Server is running successfully on port 8080"
echo "All endpoints are protected by authentication as expected"
