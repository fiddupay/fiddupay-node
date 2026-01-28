#!/bin/bash

# Quick Database Reset Script
# Drops and recreates the database completely

set -e

DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-fiddupay}"
DB_USER="${DB_USER:-postgres}"

echo " Quick Database Reset"
echo "======================"
echo ""

read -p "  This will DROP and RECREATE the database. Continue? (type 'RESET' to confirm): " confirmation

if [ "$confirmation" != "RESET" ]; then
    echo " Reset cancelled."
    exit 1
fi

echo ""
echo "  Dropping database..."
dropdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME" --if-exists

echo " Creating new database..."
createdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME"

echo " Running migrations..."
cd backend
sqlx migrate run --database-url "postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"

echo ""
echo " Database reset completed!"
echo " Fresh database ready for use!"
