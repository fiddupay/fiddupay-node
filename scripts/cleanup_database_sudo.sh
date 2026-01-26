#!/bin/bash

# Database cleanup using sudo (no password needed)

set -e

DB_NAME="${DB_NAME:-fiddupay}"

echo "🗑️  FidduPay Database Cleanup (Sudo Method)"
echo "==========================================="
echo ""

read -p "⚠️  This will DELETE ALL DATA. Are you sure? (type 'DELETE' to confirm): " confirmation

if [ "$confirmation" != "DELETE" ]; then
    echo " Cleanup cancelled."
    exit 1
fi

echo ""
echo " Starting database cleanup using sudo..."

# SQL cleanup commands
CLEANUP_SQL="
-- Disable foreign key checks temporarily
SET session_replication_role = replica;

-- Clear all payment-related data
TRUNCATE TABLE partial_payments CASCADE;
TRUNCATE TABLE payment_transactions CASCADE;
TRUNCATE TABLE webhook_events CASCADE;
TRUNCATE TABLE refunds CASCADE;

-- Clear merchant data (keep structure)
TRUNCATE TABLE merchant_balances CASCADE;
TRUNCATE TABLE merchant_wallets CASCADE;
TRUNCATE TABLE merchant_withdrawals CASCADE;
TRUNCATE TABLE merchant_invoices CASCADE;
TRUNCATE TABLE merchant_api_keys CASCADE;
TRUNCATE TABLE merchant_users CASCADE;
TRUNCATE TABLE merchants CASCADE;

-- Clear analytics and logs
TRUNCATE TABLE analytics_events CASCADE;
TRUNCATE TABLE audit_logs CASCADE;
TRUNCATE TABLE security_events CASCADE;

-- Clear session data
TRUNCATE TABLE user_sessions CASCADE;
TRUNCATE TABLE rate_limit_entries CASCADE;

-- Re-enable foreign key checks
SET session_replication_role = DEFAULT;

-- Reset sequences
SELECT setval(pg_get_serial_sequence('merchants', 'id'), 1, false);
SELECT setval(pg_get_serial_sequence('payment_transactions', 'id'), 1, false);
SELECT setval(pg_get_serial_sequence('refunds', 'id'), 1, false);
SELECT setval(pg_get_serial_sequence('webhook_events', 'id'), 1, false);

-- Vacuum to reclaim space
VACUUM ANALYZE;
"

# Execute cleanup using sudo
echo "🗑️  Truncating tables..."
echo "$CLEANUP_SQL" | sudo -u postgres psql -d "$DB_NAME"

echo ""
echo " Database cleanup completed!"
echo ""
echo " Verification:"
echo "================"

# Verify cleanup
VERIFY_SQL="
SELECT 
    tablename,
    n_live_tup as live_rows
FROM pg_stat_user_tables 
WHERE schemaname = 'public'
ORDER BY tablename;
"

echo "$VERIFY_SQL" | sudo -u postgres psql -d "$DB_NAME"

echo ""
echo " Database is now clean and ready for fresh data!"
