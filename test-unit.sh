#!/bin/bash

# Unit Test for API Key Generation
echo "🔍 Testing API Key Generation"
echo "============================="

BASE_URL="http://localhost:8080/api/v1"

# Start backend if not running
if ! curl -s $BASE_URL/status > /dev/null 2>&1; then
    echo "Starting backend..."
    cd /home/vibes/crypto-payment-gateway/backend
    source .env
    ./target/release/fiddupay &
    sleep 3
fi

# Test registration and check key format
echo "Testing merchant registration..."
TIMESTAMP=$(date +%s)
EMAIL="unit-test-${TIMESTAMP}@test.com"

RESPONSE=$(curl -s -X POST $BASE_URL/merchants/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"business_name\":\"Unit Test $TIMESTAMP\",\"webhook_url\":\"https://example.com/webhook\",\"password\":\"testpass123\"}")

echo "Response: $RESPONSE"

API_KEY=$(echo $RESPONSE | jq -r '.api_key')
echo "Generated API Key: $API_KEY"

# Check key format
if [[ $API_KEY == sk_* ]]; then
    echo " API key has correct 'sk_' prefix"
    echo " Key generation working correctly"
else
    echo " API key missing 'sk_' prefix"
    echo "🔍 Investigating issue..."
    
    # Check if binary is using updated code
    echo "Binary timestamp:"
    ls -la /home/vibes/crypto-payment-gateway/backend/target/release/fiddupay
    
    echo "Source file timestamp:"
    ls -la /home/vibes/crypto-payment-gateway/backend/src/services/merchant_service.rs
    
    exit 1
fi

echo "Unit test passed!"
