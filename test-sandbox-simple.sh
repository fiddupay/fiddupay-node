#!/bin/bash

# Test Sandbox Network Configuration (Simple)
echo "🧪 Testing Sandbox Network Configuration"
echo "========================================"

BASE_URL="http://localhost:8080"

echo "1. Creating a test payment directly in database..."

# Insert a test payment directly into the database
cd /home/vibes/crypto-payment-gateway/backend
source .env

PAYMENT_ID="pay_sandbox_test_$(date +%s)"
LINK_ID="link_$(date +%s)"

psql $DATABASE_URL -c "
INSERT INTO payment_transactions (
    payment_id, merchant_id, amount, amount_usd, crypto_type, network, 
    to_address, status, expires_at, fee_percentage, fee_amount, fee_amount_usd,
    required_confirmations, created_at
) VALUES (
    '$PAYMENT_ID', 33, 0.1, 10.00, 'SOL', 'Solana Devnet',
    'DevnetTestAddress123', 'PENDING', NOW() + INTERVAL '15 minutes', 1.5, 0.001, 0.15,
    32, NOW()
) RETURNING id;
"

# Get the inserted payment ID
PAYMENT_DB_ID=$(psql $DATABASE_URL -t -c "SELECT id FROM payment_transactions WHERE payment_id = '$PAYMENT_ID';")

psql $DATABASE_URL -c "
INSERT INTO payment_links (payment_id, link_id, merchant_id, created_at, expires_at)
VALUES ($PAYMENT_DB_ID, '$LINK_ID', 33, NOW(), NOW() + INTERVAL '15 minutes');
"

echo " Test payment created with ID: $PAYMENT_ID"
echo " Payment link ID: $LINK_ID"

echo ""
echo "2. Testing payment page..."
echo "URL: $BASE_URL/pay/$LINK_ID"

# Test payment page content
PAGE_CONTENT=$(curl -s "$BASE_URL/pay/$LINK_ID")

if echo "$PAGE_CONTENT" | grep -q "SANDBOX"; then
    echo " Payment page shows SANDBOX badge"
else
    echo " Payment page missing SANDBOX indicator"
fi

if echo "$PAGE_CONTENT" | grep -q "Devnet"; then
    echo " Payment page shows Devnet network"
else
    echo " Payment page missing Devnet indicator"
fi

if echo "$PAGE_CONTENT" | grep -q "Solana Devnet"; then
    echo " Payment page shows 'Solana Devnet' network"
else
    echo " Payment page missing 'Solana Devnet' text"
fi

echo ""
echo "3. Sample payment page content:"
echo "$PAGE_CONTENT" | grep -A5 -B5 "Network:\|SANDBOX\|Devnet"

echo ""
echo " Sandbox Test Complete!"
