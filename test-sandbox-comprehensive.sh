#!/bin/bash

# Comprehensive Sandbox Test Suite
echo " FidduPay Sandbox Test Suite"
echo "=============================="

BASE_URL="http://localhost:8080"

# Function to restart backend if needed
restart_backend() {
    echo "Restarting backend..."
    pkill -f "fiddupay" 2>/dev/null
    sleep 2
    cd /home/vibes/crypto-payment-gateway/backend
    source .env
    ./target/release/fiddupay &
    sleep 3
    
    # Wait for health check
    for i in {1..10}; do
        HEALTH=$(curl -s $BASE_URL/health 2>/dev/null)
        if [[ $HEALTH == *"healthy"* ]]; then
            echo " Backend restarted successfully"
            return 0
        fi
        sleep 1
    done
    echo " Backend failed to restart"
    return 1
}

# Function to test registration with retries
test_registration() {
    local email=$1
    local business_name=$2
    local max_retries=3
    
    for i in $(seq 1 $max_retries); do
        echo "Registration attempt $i/$max_retries..."
        
        RESPONSE=$(timeout 10s curl -s -X POST $BASE_URL/api/v1/merchant/register \
          -H "Content-Type: application/json" \
          -d "{\"email\":\"$email\",\"business_name\":\"$business_name\",\"password\":\"TestPassword123!\"}")
        
        if [[ -n "$RESPONSE" && "$RESPONSE" != *"error"* ]]; then
            echo "$RESPONSE"
            return 0
        fi
        
        echo "Attempt $i failed, response: $RESPONSE"
        if [[ $i -lt $max_retries ]]; then
            echo "Restarting backend and retrying..."
            restart_backend || return 1
        fi
    done
    
    echo " Registration failed after $max_retries attempts"
    return 1
}

# Start tests
echo "Starting comprehensive sandbox tests..."

# Test 1: Basic API Key Generation
echo "1. Testing basic API key generation..."
TIMESTAMP=$(date +%s)
EMAIL="sandbox-test-${TIMESTAMP}@test.com"

REGISTER_RESPONSE=$(test_registration "$EMAIL" "Sandbox Test $TIMESTAMP")
if [[ $? -ne 0 ]]; then
    echo " Registration test failed"
    exit 1
fi

echo "Registration successful: $REGISTER_RESPONSE"
API_KEY=$(echo $REGISTER_RESPONSE | jq -r '.api_key // empty')
echo "Generated API Key: $API_KEY"

# Validate API key format
if [[ -z "$API_KEY" || "$API_KEY" == "null" ]]; then
    echo " No API key in response"
    exit 1
elif [[ $API_KEY == sk_* ]]; then
    echo " API key has correct 'sk_' prefix"
else
    echo " API key missing 'sk_' prefix: $API_KEY"
    exit 1
fi

# Test 2: API Key Authentication
echo "2. Testing API key authentication..."
STATUS_RESPONSE=$(curl -s -H "Authorization: Bearer $API_KEY" $BASE_URL/api/v1/status)
echo "Status response: $STATUS_RESPONSE"

if [[ $STATUS_RESPONSE == *"merchant_id"* ]]; then
    echo " API key authentication working"
else
    echo " API key authentication failed"
    exit 1
fi

# Test 3: Multiple registrations
echo "3. Testing multiple registrations..."
for i in {1..3}; do
    TIMESTAMP=$(date +%s)
    EMAIL="multi-test-${i}-${TIMESTAMP}@test.com"
    
    RESPONSE=$(test_registration "$EMAIL" "Multi Test $i")
    if [[ $? -ne 0 ]]; then
        echo " Multiple registration test failed at iteration $i"
        exit 1
    fi
    
    KEY=$(echo $RESPONSE | jq -r '.api_key // empty')
    if [[ $KEY == sk_* ]]; then
        echo " Registration $i successful with key: $KEY"
    else
        echo " Registration $i failed, invalid key: $KEY"
        exit 1
    fi
done

echo " All sandbox tests passed!"
echo " API key generation working correctly"
echo " Sandbox keys have proper 'sk_' prefixes"
echo " Authentication working"
echo " Multiple registrations working"
