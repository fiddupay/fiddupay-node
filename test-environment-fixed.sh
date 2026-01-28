#!/bin/bash

# Fixed Environment Switching Test
echo "🧪 FidduPay Environment Switching Test (Fixed)"
echo "==============================================="

BASE_URL="http://localhost:8080"

# Check backend health
echo "Checking backend health..."
HEALTH_CHECK=$(curl -s $BASE_URL/health)
if [[ $HEALTH_CHECK != *"healthy"* ]]; then
    echo " Backend not healthy"
    exit 1
fi

# Test 1: Register merchant (defaults to sandbox)
echo "1. Registering merchant (default sandbox)..."
TIMESTAMP=$(date +%s)
EMAIL="env-test-${TIMESTAMP}@test.com"

REGISTER_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/merchant/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"business_name\":\"Env Test $TIMESTAMP\",\"password\":\"TestPassword123!\"}")

echo "Registration response: $REGISTER_RESPONSE"
SANDBOX_KEY=$(echo $REGISTER_RESPONSE | jq -r '.api_key // empty')
MERCHANT_ID=$(echo $REGISTER_RESPONSE | jq -r '.user.id // empty')

echo "Sandbox API Key: $SANDBOX_KEY"
echo "Merchant ID: $MERCHANT_ID"

# Validate sandbox key
if [[ -z "$SANDBOX_KEY" || "$SANDBOX_KEY" == "null" ]]; then
    echo " No sandbox API key received"
    exit 1
elif [[ $SANDBOX_KEY == sk_* ]]; then
    echo " Sandbox key has correct 'sk_' prefix"
else
    echo " Sandbox key missing 'sk_' prefix: $SANDBOX_KEY"
    exit 1
fi

# Test 2: Switch to live environment
echo "2. Switching to live environment..."
SWITCH_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/merchant/environment/switch \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $SANDBOX_KEY" \
  -d '{"to_live": true}')

echo "Switch response: $SWITCH_RESPONSE"
LIVE_KEY=$(echo $SWITCH_RESPONSE | jq -r '.api_key // empty')
echo "Live API Key: $LIVE_KEY"

# Validate live key
if [[ -z "$LIVE_KEY" || "$LIVE_KEY" == "null" ]]; then
    echo " No live API key received"
    exit 1
elif [[ $LIVE_KEY == live_* ]]; then
    echo " Live key has correct 'live_' prefix"
else
    echo " Live key missing 'live_' prefix: $LIVE_KEY"
    exit 1
fi

# Test 3: Switch back to sandbox
echo "3. Switching back to sandbox..."
SWITCH_BACK_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/merchant/environment/switch \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $LIVE_KEY" \
  -d '{"to_live": false}')

echo "Switch back response: $SWITCH_BACK_RESPONSE"
SANDBOX_KEY2=$(echo $SWITCH_BACK_RESPONSE | jq -r '.api_key // empty')
echo "New Sandbox API Key: $SANDBOX_KEY2"

# Validate new sandbox key
if [[ -z "$SANDBOX_KEY2" || "$SANDBOX_KEY2" == "null" ]]; then
    echo " No new sandbox API key received"
    exit 1
elif [[ $SANDBOX_KEY2 == sk_* ]]; then
    echo " New sandbox key has correct 'sk_' prefix"
else
    echo " New sandbox key missing 'sk_' prefix: $SANDBOX_KEY2"
    exit 1
fi

echo " Environment switching test passed!"
echo " All API keys generated with correct prefixes"
