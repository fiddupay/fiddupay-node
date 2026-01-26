#!/bin/bash

# Merchant Registration and Environment Toggle Test
echo "🧪 FidduPay Merchant Environment Test"
echo "===================================="

BASE_URL="http://localhost:8080/api/v1"

# Test 1: Register merchant (defaults to sandbox)
echo "1. Registering merchant (default sandbox)..."
TIMESTAMP=$(date +%s)
EMAIL="env-test-${TIMESTAMP}@test.com"
REGISTER_RESPONSE=$(curl -s -X POST $BASE_URL/merchants/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"business_name\":\"Env Test Merchant $TIMESTAMP\",\"webhook_url\":\"https://example.com/webhook\",\"password\":\"testpass123\"}")

echo "Registration: $REGISTER_RESPONSE"
SANDBOX_KEY=$(echo $REGISTER_RESPONSE | jq -r '.api_key')
echo "Sandbox API Key: $SANDBOX_KEY"

# Validate sandbox key format
if [[ $SANDBOX_KEY == sk_* ]]; then
    echo " Sandbox key has correct 'sk_' prefix"
else
    echo " Sandbox key missing 'sk_' prefix"
    exit 1
fi

# Test 2: Test sandbox authentication
echo ""
echo "2. Testing sandbox authentication..."
SANDBOX_AUTH=$(curl -s -H "Authorization: Bearer $SANDBOX_KEY" $BASE_URL/status)
if [[ -z "$SANDBOX_AUTH" ]]; then
    echo " Sandbox authentication successful"
else
    echo " Sandbox authentication failed: $SANDBOX_AUTH"
    exit 1
fi

# Test 3: Switch to live environment
echo ""
echo "3. Switching to live environment..."
LIVE_RESPONSE=$(curl -s -X POST $BASE_URL/merchants/environment/switch \
  -H "Authorization: Bearer $SANDBOX_KEY" \
  -H "Content-Type: application/json" \
  -d '{"to_live": true}')

echo "Live switch: $LIVE_RESPONSE"
LIVE_KEY=$(echo $LIVE_RESPONSE | jq -r '.api_key')
echo "Live API Key: $LIVE_KEY"

# Validate live key format
if [[ $LIVE_KEY == live_* ]]; then
    echo " Live key has correct 'live_' prefix"
else
    echo " Live key missing 'live_' prefix"
    exit 1
fi

# Test 4: Test live authentication
echo ""
echo "4. Testing live authentication..."
LIVE_AUTH=$(curl -s -H "Authorization: Bearer $LIVE_KEY" $BASE_URL/status)
if [[ -z "$LIVE_AUTH" ]]; then
    echo " Live authentication successful"
else
    echo " Live authentication failed: $LIVE_AUTH"
    exit 1
fi

# Test 5: Switch back to sandbox
echo ""
echo "5. Switching back to sandbox..."
BACK_RESPONSE=$(curl -s -X POST $BASE_URL/merchants/environment/switch \
  -H "Authorization: Bearer $LIVE_KEY" \
  -H "Content-Type: application/json" \
  -d '{"to_live": false}')

echo "Back to sandbox: $BACK_RESPONSE"
NEW_SANDBOX_KEY=$(echo $BACK_RESPONSE | jq -r '.api_key')
echo "New Sandbox Key: $NEW_SANDBOX_KEY"

# Validate new sandbox key
if [[ $NEW_SANDBOX_KEY == sk_* ]]; then
    echo " New sandbox key has correct 'sk_' prefix"
else
    echo " New sandbox key missing 'sk_' prefix"
    exit 1
fi

echo ""
echo " All environment tests passed!"
echo "Final Sandbox Key: $NEW_SANDBOX_KEY"
