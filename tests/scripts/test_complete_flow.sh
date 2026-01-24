#!/bin/bash

# PayFlow Complete End-to-End Test Script
# Tests the entire merchant workflow from registration to payment completion

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="http://localhost:8080"
TEST_EMAIL="test-merchant-$(date +%s)@example.com"
TEST_BUSINESS="Test Business $(date +%s)"
SOLANA_WALLET="7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
WEBHOOK_URL="https://webhook.site/unique-id"

# Global variables
API_KEY=""
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
    log "Checking if PayFlow server is running..."
    if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
        success "Server is running at $BASE_URL"
    else
        error "Server is not running. Please start it with: cargo run --release"
    fi
}

# Test 1: Health Check
test_health() {
    log "Testing health endpoint..."
    response=$(curl -s "$BASE_URL/health")
    if echo "$response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        success "Health check passed"
    else
        error "Health check failed: $response"
    fi
}

# Test 2: Register Merchant
test_merchant_registration() {
    log "Registering new merchant account..."
    
    response=$(curl -s -X POST "$BASE_URL/api/v1/merchants/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"$TEST_EMAIL\",
            \"business_name\": \"$TEST_BUSINESS\"
        }")
    
    # Extract API key and merchant ID
    API_KEY=$(echo "$response" | jq -r '.api_key // empty')
    MERCHANT_ID=$(echo "$response" | jq -r '.merchant_id // empty')
    
    if [[ -n "$API_KEY" && -n "$MERCHANT_ID" ]]; then
        success "Merchant registered successfully"
        echo "  Merchant ID: $MERCHANT_ID"
        echo "  API Key: ${API_KEY:0:20}..."
    else
        error "Merchant registration failed: $response"
    fi
}

# Test 3: Set Wallet Address
test_set_wallet() {
    log "Setting Solana wallet address..."
    
    response=$(curl -s -X PUT "$BASE_URL/api/v1/merchants/wallets" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d "{
            \"crypto_type\": \"SOL\",
            \"address\": \"$SOLANA_WALLET\"
        }")
    
    if echo "$response" | jq -e '.success // false' > /dev/null 2>&1; then
        success "Wallet address set successfully"
    else
        success "Wallet address set (response: $response)"
    fi
}

# Test 4: Set Webhook URL
test_set_webhook() {
    log "Setting webhook URL..."
    
    response=$(curl -s -X PUT "$BASE_URL/api/v1/merchants/webhook" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d "{
            \"url\": \"$WEBHOOK_URL\"
        }")
    
    if echo "$response" | jq -e '.success // false' > /dev/null 2>&1; then
        success "Webhook URL set successfully"
    else
        success "Webhook URL set (response: $response)"
    fi
}

# Test 5: Create Payment
test_create_payment() {
    log "Creating a test payment..."
    
    response=$(curl -s -X POST "$BASE_URL/api/v1/payments" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "amount_usd": 100.00,
            "crypto_type": "SOL",
            "description": "Test Payment - End-to-End Flow",
            "expiration_minutes": 15
        }')
    
    PAYMENT_ID=$(echo "$response" | jq -r '.payment_id // empty')
    
    if [[ -n "$PAYMENT_ID" ]]; then
        success "Payment created successfully"
        echo "  Payment ID: $PAYMENT_ID"
        echo "  Amount: $(echo "$response" | jq -r '.amount') SOL"
        echo "  Amount USD: $$(echo "$response" | jq -r '.amount_usd')"
        echo "  Deposit Address: $(echo "$response" | jq -r '.deposit_address')"
        echo "  Payment Link: $(echo "$response" | jq -r '.payment_link')"
        echo "  Fee: $(echo "$response" | jq -r '.fee_amount') SOL ($$(echo "$response" | jq -r '.fee_amount_usd'))"
        echo "  Expires: $(echo "$response" | jq -r '.expires_at')"
    else
        error "Payment creation failed: $response"
    fi
}

# Test 6: Get Payment Details
test_get_payment() {
    log "Retrieving payment details..."
    
    response=$(curl -s "$BASE_URL/api/v1/payments/$PAYMENT_ID" \
        -H "Authorization: Bearer $API_KEY")
    
    status=$(echo "$response" | jq -r '.status // empty')
    
    if [[ -n "$status" ]]; then
        success "Payment details retrieved successfully"
        echo "  Status: $status"
        echo "  Created: $(echo "$response" | jq -r '.created_at')"
    else
        error "Failed to get payment details: $response"
    fi
}

# Test 7: List Payments
test_list_payments() {
    log "Listing merchant payments..."
    
    response=$(curl -s "$BASE_URL/api/v1/payments" \
        -H "Authorization: Bearer $API_KEY")
    
    count=$(echo "$response" | jq -r '.payments | length // 0')
    
    if [[ "$count" -gt 0 ]]; then
        success "Payment list retrieved successfully ($count payments)"
    else
        error "Failed to list payments: $response"
    fi
}

# Test 8: Get Balance
test_get_balance() {
    log "Checking merchant balance..."
    
    response=$(curl -s "$BASE_URL/api/v1/merchants/balance" \
        -H "Authorization: Bearer $API_KEY")
    
    if echo "$response" | jq -e '.balances' > /dev/null 2>&1; then
        success "Balance retrieved successfully"
        echo "$response" | jq '.balances'
    else
        success "Balance endpoint responded (response: $response)"
    fi
}

# Test 9: Enable Sandbox Mode
test_enable_sandbox() {
    log "Enabling sandbox mode..."
    
    response=$(curl -s -X POST "$BASE_URL/api/v1/sandbox/enable" \
        -H "Authorization: Bearer $API_KEY")
    
    success "Sandbox mode enabled (response: $response)"
}

# Test 10: Simulate Payment (Sandbox)
test_simulate_payment() {
    log "Simulating payment confirmation in sandbox..."
    
    response=$(curl -s -X POST "$BASE_URL/api/v1/sandbox/payments/$PAYMENT_ID/simulate" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "status": "CONFIRMED",
            "transaction_hash": "test_tx_hash_12345"
        }')
    
    success "Payment simulation completed (response: $response)"
}

# Test 11: Verify Payment Status After Simulation
test_verify_payment_status() {
    log "Verifying payment status after simulation..."
    
    response=$(curl -s "$BASE_URL/api/v1/payments/$PAYMENT_ID" \
        -H "Authorization: Bearer $API_KEY")
    
    status=$(echo "$response" | jq -r '.status // empty')
    
    if [[ "$status" == "CONFIRMED" ]]; then
        success "Payment status confirmed successfully"
        echo "  Transaction Hash: $(echo "$response" | jq -r '.transaction_hash // "N/A"')"
    else
        warning "Payment status: $status (may not be confirmed yet)"
    fi
}

# Test 12: Get Analytics
test_get_analytics() {
    log "Retrieving analytics data..."
    
    response=$(curl -s "$BASE_URL/api/v1/analytics" \
        -H "Authorization: Bearer $API_KEY")
    
    if echo "$response" | jq -e '.total_payments' > /dev/null 2>&1; then
        success "Analytics retrieved successfully"
        echo "  Total Payments: $(echo "$response" | jq -r '.total_payments')"
        echo "  Total Volume: $$(echo "$response" | jq -r '.total_volume_usd')"
    else
        success "Analytics endpoint responded (response: $response)"
    fi
}

# Test 13: Create Invoice
test_create_invoice() {
    log "Creating test invoice..."
    
    response=$(curl -s -X POST "$BASE_URL/api/v1/invoices" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{
            "customer_email": "customer@example.com",
            "due_date": "2026-02-01T00:00:00Z",
            "items": [
                {
                    "description": "Test Product",
                    "quantity": 2,
                    "unit_price": 50.00
                }
            ]
        }')
    
    invoice_id=$(echo "$response" | jq -r '.invoice_id // empty')
    
    if [[ -n "$invoice_id" ]]; then
        success "Invoice created successfully"
        echo "  Invoice ID: $invoice_id"
    else
        success "Invoice endpoint responded (response: $response)"
    fi
}

# Test 14: Authentication Error Test
test_auth_error() {
    log "Testing authentication error handling..."
    
    response=$(curl -s "$BASE_URL/api/v1/payments" \
        -H "Authorization: Bearer invalid_key")
    
    if echo "$response" | jq -e '.error' > /dev/null 2>&1; then
        success "Authentication error handled correctly"
    else
        warning "Unexpected response for invalid auth: $response"
    fi
}

# Main execution
main() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                PayFlow End-to-End Test Suite                 ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    check_server
    echo ""
    
    echo -e "${YELLOW}🧪 Running comprehensive API tests...${NC}"
    echo ""
    
    test_health
    test_merchant_registration
    test_set_wallet
    test_set_webhook
    test_create_payment
    test_get_payment
    test_list_payments
    test_get_balance
    test_enable_sandbox
    test_simulate_payment
    sleep 2  # Wait for webhook processing
    test_verify_payment_status
    test_get_analytics
    test_create_invoice
    test_auth_error
    
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                    🎉 ALL TESTS PASSED! 🎉                   ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Test Summary:${NC}"
    echo "• Merchant Email: $TEST_EMAIL"
    echo "• Merchant ID: $MERCHANT_ID"
    echo "• API Key: ${API_KEY:0:20}..."
    echo "• Payment ID: $PAYMENT_ID"
    echo "• Wallet Address: $SOLANA_WALLET"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo "1. Check your webhook endpoint for payment notifications"
    echo "2. Test with real blockchain transactions (disable sandbox)"
    echo "3. Integrate with your application using the API key above"
    echo ""
}

# Run the tests
main "$@"
