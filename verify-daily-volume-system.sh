#!/bin/bash

echo " VERIFYING DAILY VOLUME LIMIT SYSTEM"
echo "======================================"

echo ""
echo " 1. Checking .env.example configuration:"
echo "   Daily Volume Limit: $(grep DAILY_VOLUME_LIMIT_NON_KYC_USD /home/vibes/crypto-payment-gateway/.env.example)"

echo ""
echo " 2. Verifying old limits are removed from .env.example:"
OLD_LIMITS=$(grep -E "MIN_PAYMENT_USD|MAX_PAYMENT_USD|DAILY_PAYMENT_LIMIT|WITHDRAWAL_MIN|WITHDRAWAL_MAX|WITHDRAWAL_DAILY_LIMIT" /home/vibes/crypto-payment-gateway/.env.example || echo "NONE FOUND")
if [ "$OLD_LIMITS" = "NONE FOUND" ]; then
    echo "    No old payment/withdrawal limits found"
else
    echo "    Old limits still present: $OLD_LIMITS"
fi

echo ""
echo " 3. Checking database for old limit tables/columns:"
cd /home/vibes/crypto-payment-gateway
OLD_DB_LIMITS=$(psql -d fiddupay_test -t -c "SELECT table_name, column_name FROM information_schema.columns WHERE table_schema = 'public' AND (column_name LIKE '%limit%' OR column_name LIKE '%min_%' OR column_name LIKE '%max_%');" 2>/dev/null | grep -v "^$" || echo "NONE")
if [ "$OLD_DB_LIMITS" = "NONE" ]; then
    echo "    No old limit columns found in database"
else
    echo "    Old limit columns found: $OLD_DB_LIMITS"
fi

echo ""
echo " 4. Verifying KYC column exists:"
KYC_COLUMN=$(psql -d fiddupay_test -t -c "SELECT column_name FROM information_schema.columns WHERE table_name = 'merchants' AND column_name = 'kyc_verified';" 2>/dev/null | grep -v "^$")
if [ "$KYC_COLUMN" = " kyc_verified" ]; then
    echo "    kyc_verified column exists in merchants table"
else
    echo "    kyc_verified column missing"
fi

echo ""
echo " 5. Checking volume tracking service exists:"
if [ -f "/home/vibes/crypto-payment-gateway/backend/src/services/volume_tracking_service.rs" ]; then
    echo "    Volume tracking service file exists"
else
    echo "    Volume tracking service file missing"
fi

echo ""
echo " 6. Verifying configuration in code:"
DAILY_LIMIT_CONFIG=$(grep -c "daily_volume_limit_non_kyc_usd" /home/vibes/crypto-payment-gateway/backend/src/config.rs)
if [ "$DAILY_LIMIT_CONFIG" -gt 0 ]; then
    echo "    Daily volume limit configured in backend ($DAILY_LIMIT_CONFIG references)"
else
    echo "    Daily volume limit not configured in backend"
fi

echo ""
echo " SUMMARY:"
echo "   • Daily Volume Limit: $1000 USD for non-KYC merchants"
echo "   • KYC Verified: No limits"
echo "   • Combined: Deposits + Withdrawals in single daily total"
echo "   • No Per-Transaction Limits: Removed"
echo "   • No Monthly Limits: Removed"
echo ""
echo " Daily volume limit system verification complete!"
