#!/bin/bash

echo "🔍 MERCHANT TEST COVERAGE AUDIT"
echo "==============================="

# Expected merchant endpoints (43 total)
cat > /tmp/expected_merchant_endpoints.txt << 'EOF'
/api/v1/analytics
/api/v1/analytics/export
/api/v1/audit-logs
/api/v1/merchant/api-keys/generate
/api/v1/merchant/api-keys/rotate
/api/v1/merchant/balance
/api/v1/merchant/balance/history
/api/v1/merchant/environment/switch
/api/v1/merchant/ip-whitelist
/api/v1/merchant/profile
/api/v1/merchant/wallets
/api/v1/merchant/webhook
/api/v1/payments
/api/v1/payments/:payment_id
/api/v1/payments/:payment_id/verify
/api/v1/refunds
/api/v1/refunds/:refund_id
/api/v1/refunds/:refund_id/complete
/api/v1/security/alerts
/api/v1/security/alerts/:alert_id/acknowledge
/api/v1/security/balance-alerts
/api/v1/security/balance-alerts/:alert_id/resolve
/api/v1/security/events
/api/v1/security/gas-check
/api/v1/security/settings
/api/v1/wallets
/api/v1/wallets/configure-address
/api/v1/wallets/export-key
/api/v1/wallets/gas-check
/api/v1/wallets/gas-estimates
/api/v1/wallets/generate
/api/v1/wallets/import
/api/v1/wallets/withdrawal-capability/:crypto_type
/api/v1/withdrawals
/api/v1/withdrawals/:withdrawal_id
/api/v1/withdrawals/:withdrawal_id/cancel
/api/v1/withdrawals/:withdrawal_id/process
/health
/pay/:link_id
/pay/:link_id/status
/api/v1/merchant/register
/api/v1/merchant/login
/api/v1/currencies/supported
/api/v1/status
/api/v1/blog
/api/v1/careers
EOF

# Extract endpoints from merchant test file
grep -o "axios\.[a-z]*(\`[^}]*" /home/vibes/crypto-payment-gateway/tests/merchant-api-comprehensive.js | \
  sed 's/axios\.[a-z]*(`[^/]*//g' | \
  sed 's/\${[^}]*}[^`]*/:/g' | \
  sed 's/`.*//g' | \
  sort | uniq > /tmp/merchant_test_endpoints.txt

echo "📊 MERCHANT TEST ANALYSIS:"
echo "========================="
echo "Expected endpoints: $(wc -l < /tmp/expected_merchant_endpoints.txt)"
echo "Test file endpoints: $(wc -l < /tmp/merchant_test_endpoints.txt)"

echo ""
echo "❌ MISSING FROM MERCHANT TEST:"
echo "=============================="
comm -23 /tmp/expected_merchant_endpoints.txt /tmp/merchant_test_endpoints.txt

echo ""
echo "✅ COVERED IN MERCHANT TEST:"
echo "============================"
comm -12 /tmp/expected_merchant_endpoints.txt /tmp/merchant_test_endpoints.txt | wc -l
echo "endpoints covered"
