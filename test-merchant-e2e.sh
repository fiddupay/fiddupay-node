#!/bin/bash

# End-to-End Merchant Test Script
echo "🧪 FidduPay End-to-End Merchant Test"
echo "===================================="

# Test 1: Register new merchant
echo "1. Registering new merchant..."
TIMESTAMP=$(date +%s)
EMAIL="e2e-${TIMESTAMP}@test.com"
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/merchant/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"business_name\":\"E2E Test Merchant $TIMESTAMP\",\"webhook_url\":\"https://example.com/webhook\",\"password\":\"testpass123\"}")

echo "Registration response: $RESPONSE"

# Extract API key
API_KEY=$(echo $RESPONSE | jq -r '.api_key')
echo "Generated API key: $API_KEY"

# Validate API key format
if [[ $API_KEY == sk_* ]]; then
    echo " API key has correct 'sk_' prefix"
else
    echo " API key missing 'sk_' prefix"
    exit 1
fi

# Test 2: Test authentication
echo ""
echo "2. Testing authentication..."
AUTH_RESPONSE=$(curl -s -H "Authorization: Bearer $API_KEY" http://localhost:8080/api/v1/status)
echo "Auth response: $AUTH_RESPONSE"

if [[ -z "$AUTH_RESPONSE" ]]; then
    echo " Authentication successful (empty response = success)"
else
    echo " Authentication failed: $AUTH_RESPONSE"
    exit 1
fi

# Test 3: Generate sandbox API key
echo ""
echo "3. Generating sandbox API key..."
SANDBOX_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/merchant/api-keys/generate \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"is_live": false}')

echo "Sandbox key response: $SANDBOX_RESPONSE"
SANDBOX_KEY=$(echo $SANDBOX_RESPONSE | jq -r '.api_key')
echo "Sandbox API key: $SANDBOX_KEY"

# Test 4: Test sandbox key authentication
echo ""
echo "4. Testing sandbox key authentication..."
SANDBOX_AUTH=$(curl -s -H "Authorization: Bearer $SANDBOX_KEY" http://localhost:8080/api/v1/status)

if [[ -z "$SANDBOX_AUTH" ]]; then
    echo " Sandbox authentication successful"
else
    echo " Sandbox authentication failed: $SANDBOX_AUTH"
    exit 1
fi

echo ""
echo " All merchant tests passed!"
echo "API Key: $API_KEY"
echo "Sandbox Key: $SANDBOX_KEY"
