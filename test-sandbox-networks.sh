#!/bin/bash

# Test Sandbox Network Configuration
echo "🧪 Testing Sandbox Network Configuration"
echo "========================================"

API_KEY="${API_KEY:-test_key}"
BASE_URL="http://localhost:8080"

echo "1. Testing payment creation in sandbox mode..."

# Create a test payment
PAYMENT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/payments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "amount_usd": 10.00,
    "crypto_type": "SOL",
    "description": "Test sandbox payment"
  }')

echo "Payment Response:"
echo $PAYMENT_RESPONSE | jq .

# Extract payment ID and link
PAYMENT_ID=$(echo $PAYMENT_RESPONSE | jq -r '.payment_id // "FAILED"')
PAYMENT_LINK=$(echo $PAYMENT_RESPONSE | jq -r '.payment_link // "FAILED"')

if [ "$PAYMENT_ID" != "FAILED" ] && [ "$PAYMENT_LINK" != "FAILED" ]; then
    echo " Payment created successfully!"
    echo "   Payment ID: $PAYMENT_ID"
    echo "   Payment Link: $PAYMENT_LINK"
    
    # Test payment page
    echo ""
    echo "2. Testing payment page..."
    LINK_ID=$(basename $PAYMENT_LINK)
    curl -s "$BASE_URL/pay/$LINK_ID" | grep -o "SANDBOX\|Testnet\|Devnet" | head -3
    
    if curl -s "$BASE_URL/pay/$LINK_ID" | grep -q "SANDBOX"; then
        echo " Payment page shows SANDBOX badge"
    else
        echo " Payment page missing SANDBOX indicator"
    fi
    
    if curl -s "$BASE_URL/pay/$LINK_ID" | grep -q "Devnet"; then
        echo " Payment page shows Devnet network"
    else
        echo " Payment page missing Devnet indicator"
    fi
else
    echo " Payment creation failed"
fi

echo ""
echo " Sandbox Test Complete!"
