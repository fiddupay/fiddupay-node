#!/bin/bash

# Fixed Unit Test for API Key Generation
echo "🔍 Testing API Key Generation (Fixed)"
echo "====================================="

BASE_URL="http://localhost:8080"

# Check if backend is running
echo "Checking backend health..."
HEALTH_CHECK=$(curl -s $BASE_URL/health)
echo "Health response: $HEALTH_CHECK"

if [[ $HEALTH_CHECK != *"healthy"* ]]; then
    echo " Backend not healthy, starting..."
    cd /home/vibes/crypto-payment-gateway/backend
    source .env
    ./target/release/fiddupay &
    sleep 5
    
    # Recheck health
    HEALTH_CHECK=$(curl -s $BASE_URL/health)
    echo "Health after start: $HEALTH_CHECK"
    
    if [[ $HEALTH_CHECK != *"healthy"* ]]; then
        echo " Backend failed to start properly"
        exit 1
    fi
fi

# Test registration with exact working format
echo "Testing merchant registration..."
TIMESTAMP=$(date +%s)
EMAIL="unit-test-fixed-${TIMESTAMP}@test.com"

echo "Making registration request..."
RESPONSE=$(curl -s -w "HTTP_CODE:%{http_code}" -X POST $BASE_URL/api/v1/merchant/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"business_name\":\"Unit Test Fixed $TIMESTAMP\",\"password\":\"TestPassword123!\"}")

echo "Full response: $RESPONSE"

# Extract HTTP code and body
HTTP_CODE=$(echo "$RESPONSE" | grep -o "HTTP_CODE:[0-9]*" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed 's/HTTP_CODE:[0-9]*$//')

echo "HTTP Code: $HTTP_CODE"
echo "Response Body: $BODY"

if [[ $HTTP_CODE != "201" && $HTTP_CODE != "200" ]]; then
    echo " Registration failed with HTTP code: $HTTP_CODE"
    echo "Response: $BODY"
    exit 1
fi

# Extract API key
API_KEY=$(echo $BODY | jq -r '.api_key // empty')
echo "Generated API Key: $API_KEY"

# Check key format
if [[ -z "$API_KEY" || "$API_KEY" == "null" ]]; then
    echo " No API key in response"
    echo "Full response: $BODY"
    exit 1
elif [[ $API_KEY == sk_* ]]; then
    echo " API key has correct 'sk_' prefix"
    echo " Key generation working correctly"
    echo " Unit test passed!"
else
    echo " API key missing 'sk_' prefix: $API_KEY"
    exit 1
fi
