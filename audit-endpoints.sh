#!/bin/bash

echo "🔍 FidduPay Backend Endpoint Audit"
echo "=================================="

echo ""
echo "📋 EXTRACTING ALL ENDPOINTS FROM BACKEND..."

# Extract all endpoints from routes.rs
grep "\.route(" /home/vibes/crypto-payment-gateway/backend/src/api/routes.rs | grep -v "//" | sed 's/.*\.route("//' | sed 's/", .*//' | sort > /tmp/all_endpoints.txt

echo "✅ Found $(wc -l < /tmp/all_endpoints.txt) total endpoints"

echo ""
echo "📊 CATEGORIZING ENDPOINTS..."

# Public endpoints
echo "🌐 PUBLIC ENDPOINTS:"
grep -E "^(/health|/pay|/api/v1/merchants/(register|login)|/api/v1/currencies|/api/v1/(status|blog|careers))" /tmp/all_endpoints.txt | nl

# Merchant endpoints  
echo ""
echo "👤 MERCHANT ENDPOINTS:"
grep -E "^/api/v1/(merchants|payments|refunds|analytics|withdrawals|wallets|security|audit-logs)" /tmp/all_endpoints.txt | grep -v "admin" | nl

# Sandbox endpoints
echo ""
echo "🏖️ SANDBOX ENDPOINTS:"
grep "^/api/v1/sandbox" /tmp/all_endpoints.txt | nl

# Admin endpoints
echo ""
echo "🔧 ADMIN ENDPOINTS:"
grep "^/api/v1/admin" /tmp/all_endpoints.txt | nl

echo ""
echo "📊 ENDPOINT SUMMARY:"
echo "==================="
echo "Public: $(grep -E "^(/health|/pay|/api/v1/merchants/(register|login)|/api/v1/currencies|/api/v1/(status|blog|careers))" /tmp/all_endpoints.txt | wc -l)"
echo "Merchant: $(grep -E "^/api/v1/(merchants|payments|refunds|analytics|withdrawals|wallets|security|audit-logs)" /tmp/all_endpoints.txt | grep -v "admin" | wc -l)"
echo "Sandbox: $(grep "^/api/v1/sandbox" /tmp/all_endpoints.txt | wc -l)"
echo "Admin: $(grep "^/api/v1/admin" /tmp/all_endpoints.txt | wc -l)"
echo "Total: $(wc -l < /tmp/all_endpoints.txt)"
