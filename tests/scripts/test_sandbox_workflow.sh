#!/bin/bash

# fiddupay Sandbox End-to-End Test
# Tests complete merchant workflow in sandbox mode

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

BASE_URL="http://localhost:8080"
TEST_EMAIL="sandbox-test-$(date +%s)@example.com"
TEST_BUSINESS="Sandbox Test Business $(date +%s)"

API_KEY="${API_KEY:-}"
MERCHANT_ID=""
PAYMENT_ID=""

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check if server is running
check_server() {
    log "Checking if fiddupay server is running..."
    if ! curl -s "$BASE_URL/health" > /dev/null; then
        error "fiddupay server not running at $BASE_URL"
    fi
    success "Server is running"
}

# Step 1: Register merchant
register_merchant() {
    log "Step 1: Registering merchant..."
    
    local response=$(curl -s -X POST "$BASE_URL/api/v1/merchants/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"business_name\": \"$TEST_BUSINESS\",
            \"email\": \"$TEST_EMAIL\",
            \"password\": \"password123\"
        }")
    
    if echo "$response" | grep -q "error"; then
        error "Failed to register merchant: $response"
    fi
    
    API_KEY=$(echo "$response" | jq -r '.api_key')
    MERCHANT_ID=$(echo "$response" | jq -r '.merchant_id')
    
    if [ "$API_KEY" = "null" ] || [ "$MERCHANT_ID" = "null" ]; then
        error "Invalid registration response: $response"
    fi
    
    success "Merchant registered - ID: $MERCHANT_ID"
}

# Step 2: Enable sandbox mode
enable_sandbox() {
    log "Step 2: Enabling sandbox mode..."
    
    local response=$(curl -s -X POST "$BASE_URL/api/v1/sandbox/enable" \
        -H "Authorization: Bearer $API_KEY")
    
    if echo "$response" | grep -q "error"; then
        error "Failed to enable sandbox: $response"
    fi
    
    # Extract new sandbox API key
    local sandbox_key=$(echo "$response" | jq -r '.sandbox_api_key')
    if [ "$sandbox_key" != "null" ] && [ "$sandbox_key" != "" ]; then
        API_KEY="$sandbox_key"
        success "Sandbox enabled - New API key: ${API_KEY:0:20}..."
    else
        success "Sandbox enabled - Using existing API key"
    fi
}

# Step 3: Configure wallet
configure_wallet() {
    log "Step 3: Configuring wallet..."
    
    local response=$(curl -s -X PUT "$BASE_URL/api/v1/merchants/wallets" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "crypto_type": "USDT_ETH",
            "address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87"
        }')
    
    if echo "$response" | grep -q "error"; then
        error "Failed to configure wallet: $response"
    fi
    
    success "Wallet configured for USDT_ETH"
}

# Step 4: Set webhook
set_webhook() {
    log "Step 4: Setting webhook..."
    
    local response=$(curl -s -X PUT "$BASE_URL/api/v1/merchants/webhook" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "url": "https://webhook.site/sandbox-test"
        }')
    
    if echo "$response" | grep -q "error"; then
        error "Failed to set webhook: $response"
    fi
    
    success "Webhook configured"
}

# Step 5: Create payment
create_payment() {
    log "Step 5: Creating payment..."
    
    local response=$(curl -s -X POST "$BASE_URL/api/v1/payments" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "amount_usd": "100.00",
            "crypto_type": "USDT_ETH",
            "description": "Sandbox test payment"
        }')
    
    if echo "$response" | grep -q "error"; then
        error "Failed to create payment: $response"
    fi
    
    PAYMENT_ID=$(echo "$response" | jq -r '.payment_id')
    local status=$(echo "$response" | jq -r '.status')
    
    if [ "$PAYMENT_ID" = "null" ]; then
        error "Invalid payment response: $response"
    fi
    
    success "Payment created - ID: $PAYMENT_ID, Status: $status"
}

# Step 6: Simulate payment confirmation
simulate_payment() {
    log "Step 6: Simulating payment confirmation..."
    
    local response=$(curl -s -X POST "$BASE_URL/api/v1/sandbox/payments/$PAYMENT_ID/simulate" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "success": true
        }')
    
    if echo "$response" | grep -q "error"; then
        error "Failed to simulate payment: $response"
    fi
    
    success "Payment simulation successful"
}

# Step 7: Verify payment status
verify_payment_status() {
    log "Step 7: Verifying payment status..."
    
    local response=$(curl -s -X GET "$BASE_URL/api/v1/payments/$PAYMENT_ID" \
        -H "Authorization: Bearer $API_KEY")
    
    if echo "$response" | grep -q "error"; then
        error "Failed to get payment: $response"
    fi
    
    local status=$(echo "$response" | jq -r '.status')
    local confirmed_at=$(echo "$response" | jq -r '.confirmed_at')
    
    if [ "$status" != "CONFIRMED" ]; then
        error "Payment status not confirmed: $status"
    fi
    
    if [ "$confirmed_at" = "null" ]; then
        error "Payment not marked as confirmed"
    fi
    
    success "Payment confirmed - Status: $status"
}

# Step 8: Test payment failure simulation
test_payment_failure() {
    log "Step 8: Testing payment failure simulation..."
    
    # Create another payment
    local response=$(curl -s -X POST "$BASE_URL/api/v1/payments" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "amount_usd": "50.00",
            "crypto_type": "USDT_ETH",
            "description": "Sandbox failure test"
        }')
    
    local fail_payment_id=$(echo "$response" | jq -r '.payment_id')
    
    # Simulate failure
    response=$(curl -s -X POST "$BASE_URL/api/v1/sandbox/payments/$fail_payment_id/simulate" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "success": false
        }')
    
    if echo "$response" | grep -q "error"; then
        error "Failed to simulate payment failure: $response"
    fi
    
    # Verify failed status
    response=$(curl -s -X GET "$BASE_URL/api/v1/payments/$fail_payment_id" \
        -H "Authorization: Bearer $API_KEY")
    
    local status=$(echo "$response" | jq -r '.status')
    
    if [ "$status" != "FAILED" ]; then
        error "Payment failure simulation failed: $status"
    fi
    
    success "Payment failure simulation successful - Status: $status"
}

# Step 9: Test analytics
test_analytics() {
    log "Step 9: Testing analytics..."
    
    local response=$(curl -s -X GET "$BASE_URL/api/v1/analytics" \
        -H "Authorization: Bearer $API_KEY")
    
    if echo "$response" | grep -q "error"; then
        error "Failed to get analytics: $response"
    fi
    
    local successful_payments=$(echo "$response" | jq -r '.successful_payments')
    local failed_payments=$(echo "$response" | jq -r '.failed_payments')
    
    success "Analytics retrieved - Successful: $successful_payments, Failed: $failed_payments"
}

# Step 10: Test balance
test_balance() {
    log "Step 10: Testing balance..."
    
    local response=$(curl -s -X GET "$BASE_URL/api/v1/balance" \
        -H "Authorization: Bearer $API_KEY")
    
    if echo "$response" | grep -q "error"; then
        error "Failed to get balance: $response"
    fi
    
    local total_usd=$(echo "$response" | jq -r '.total_usd')
    
    success "Balance retrieved - Total USD: $total_usd"
}

# Main execution
main() {
    echo " fiddupay Sandbox End-to-End Test"
    echo "=================================="
    echo ""
    
    check_server
    register_merchant
    enable_sandbox
    configure_wallet
    set_webhook
    create_payment
    simulate_payment
    verify_payment_status
    test_payment_failure
    test_analytics
    test_balance
    
    echo ""
    echo " Sandbox End-to-End Test Summary"
    echo "================================="
    success "All sandbox workflow steps completed successfully!"
    echo ""
    echo "Test Details:"
    echo "- Merchant ID: $MERCHANT_ID"
    echo "- Sandbox API Key: ${API_KEY:0:20}..."
    echo "- Test Payment ID: $PAYMENT_ID"
    echo "- Email: $TEST_EMAIL"
    echo ""
    success "Sandbox is fully functional for merchant workflows! ✨"
}

# Run main function
main "$@"
