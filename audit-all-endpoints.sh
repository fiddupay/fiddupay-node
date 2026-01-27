#!/bin/bash

echo "🔍 COMPREHENSIVE ENDPOINT COVERAGE AUDIT"
echo "========================================"

# Extract all endpoints from routes.rs
echo "📋 Extracting all backend endpoints..."
grep -E "\.route\(" /home/vibes/crypto-payment-gateway/backend/src/api/routes.rs | \
  grep -v "//" | \
  sed 's/.*\.route("//' | \
  sed 's/", .*//' | \
  sort > /tmp/all_backend_endpoints.txt

echo "✅ Found $(wc -l < /tmp/all_backend_endpoints.txt) total backend endpoints"

# Categorize endpoints
echo ""
echo "📊 ENDPOINT CATEGORIZATION:"
echo "=========================="

# Public endpoints (no auth required)
echo "🌐 PUBLIC ENDPOINTS (9):"
grep -E "^(/health|/pay|/api/v1/merchants/(register|login)|/api/v1/currencies|/api/v1/(status|blog|careers))" /tmp/all_backend_endpoints.txt | nl

# Merchant endpoints (protected, non-admin)
echo ""
echo "👤 MERCHANT ENDPOINTS (43):"
grep -E "^/api/v1/(merchants|payments|refunds|analytics|withdrawals|wallets|security|audit-logs)" /tmp/all_backend_endpoints.txt | \
  grep -v "admin" | nl

# Sandbox endpoints
echo ""
echo "🏖️ SANDBOX ENDPOINTS (2):"
grep "^/api/v1/sandbox" /tmp/all_backend_endpoints.txt | nl

# Admin endpoints
echo ""
echo "🔧 ADMIN ENDPOINTS (5):"
grep "^/api/v1/admin" /tmp/all_backend_endpoints.txt | nl

echo ""
echo "📊 SUMMARY: $(wc -l < /tmp/all_backend_endpoints.txt) total endpoints"
echo "  - Public: $(grep -E "^(/health|/pay|/api/v1/merchants/(register|login)|/api/v1/currencies|/api/v1/(status|blog|careers))" /tmp/all_backend_endpoints.txt | wc -l)"
echo "  - Merchant: $(grep -E "^/api/v1/(merchants|payments|refunds|analytics|withdrawals|wallets|security|audit-logs)" /tmp/all_backend_endpoints.txt | grep -v "admin" | wc -l)"
echo "  - Sandbox: $(grep "^/api/v1/sandbox" /tmp/all_backend_endpoints.txt | wc -l)"
echo "  - Admin: $(grep "^/api/v1/admin" /tmp/all_backend_endpoints.txt | wc -l)"
