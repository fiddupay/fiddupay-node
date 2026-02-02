#!/bin/bash

# Create FidduPay Database and Run Migrations (Fixed Order)

set -e

DB_NAME="${DB_NAME:-fiddupay}"
DB_USER="${DB_USER:-postgres}"

echo " Creating FidduPay Database (Fixed)"
echo "===================================="
echo ""

# Drop existing database if it exists
echo "  Dropping existing database (if exists)..."
sudo -u postgres dropdb "$DB_NAME" --if-exists

echo " Creating database: $DB_NAME"
sudo -u postgres createdb "$DB_NAME"

echo " Creating database user (if needed)..."
sudo -u postgres psql -c "
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'fiddupay') THEN
        CREATE ROLE fiddupay WITH LOGIN PASSWORD ')h£,ZfI8T9-U1579<)';
    END IF;
END
\$\$;
" > /dev/null

echo " Granting permissions..."
sudo -u postgres psql -c "
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO fiddupay;
ALTER DATABASE $DB_NAME OWNER TO fiddupay;
" > /dev/null

echo " Running migrations in correct order..."
cd backend

# Run migrations manually in the correct order
echo "1⃣  Creating merchant tables..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000001_create_merchant_tables.sql"

echo "2⃣  Creating payment tables..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000002_create_payment_tables.sql"

echo "3⃣  Creating webhook and refund tables..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000003_create_webhook_refund_tables.sql"

echo "4⃣  Setting up balance management..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000004_balance_management.sql"

echo "5⃣  Adding withdrawals..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000005_withdrawals.sql"

echo "6⃣  Adding roles, invoices, and 2FA..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000006_roles_invoices_2fa.sql"

echo "7⃣  Adding merchant currencies..."
if [ -f "migrations/20240101000007_merchant_currencies.sql" ]; then
    sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240101000007_merchant_currencies.sql"
fi

echo "8⃣  Adding performance indexes..."
sudo -u postgres psql -d "$DB_NAME" -f "migrations/20240125000001_performance_indexes.sql"

cd ..

echo ""
echo " Database setup completed!"
echo ""
echo " Database Details:"
echo "==================="
echo "Database: $DB_NAME"
echo "User: fiddupay"
echo "Password: )h£,ZfI8T9-U1579<)"
echo "Connection: postgresql://fiddupay:)h£,ZfI8T9-U1579<)@localhost:5432/$DB_NAME"
echo ""

echo " Verifying tables..."
sudo -u postgres psql -d "$DB_NAME" -c "\dt" -P pager=off

echo ""
echo " Table counts:"
sudo -u postgres psql -d "$DB_NAME" -c "
SELECT schemaname, relname as table_name, n_live_tup as row_count
FROM pg_stat_user_tables 
WHERE schemaname = 'public'
ORDER BY relname;
" -P pager=off

echo ""
echo " FidduPay database is ready for use!"
