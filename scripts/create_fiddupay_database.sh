#!/bin/bash

# Create FidduPay Database and Run Migrations

set -e

DB_NAME="${DB_NAME:-fiddupay}"
DB_USER="${DB_USER:-postgres}"

echo " Creating FidduPay Database"
echo "============================="
echo ""

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

echo " Running migrations..."
cd backend

# Check if sqlx is available
if command -v sqlx &> /dev/null; then
    echo "Using sqlx migrate..."
    sqlx migrate run --database-url "postgresql://fiddupay:)h£,ZfI8T9-U1579<)@localhost:5432/$DB_NAME"
else
    echo "sqlx not found, running migrations manually..."
    for migration in migrations/*.sql; do
        if [ -f "$migration" ]; then
            echo "Running: $(basename "$migration")"
            sudo -u postgres psql -d "$DB_NAME" -f "$migration"
        fi
    done
fi

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
sudo -u postgres psql -d "$DB_NAME" -c "\dt"

echo ""
echo " FidduPay database is ready for use!"
