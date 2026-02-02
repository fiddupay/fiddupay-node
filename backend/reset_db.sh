#!/bin/bash
set -e

# Configuration
DB_NAME="fiddupay"
DB_USER="fiddupay_user"
# Use hex to ensure alphanumeric characters only (safer for URLs/scripts)
DB_PASS=$(openssl rand -hex 16)

echo "🛑 Stopping backend service..."
pm2 stop fiddupay-backend || true

echo "🗑️  Dropping existing database and user..."
sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"
# Handle dependencies (e.g. ownership of other DBs like fiddupay_production)
sudo -u postgres psql -c "REASSIGN OWNED BY $DB_USER TO postgres;" || true
sudo -u postgres psql -c "DROP OWNED BY $DB_USER;" || true
sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;"

echo "✨ Creating new secure user and database..."
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"

# Update .env file with new password
echo "📝 Updating .env file..."
# Use | as delimiter for sed to avoid conflicts with / in URL
sed -i "s|DATABASE_URL=.*|DATABASE_URL=postgres://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME|g" .env

echo "🔄 Running migrations..."
export DATABASE_URL="postgres://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
# Use sqlx from cargo bin if not in path
if command -v sqlx &> /dev/null; then
    sqlx migrate run
else
    ~/.cargo/bin/sqlx migrate run
fi

echo "✅ Database reset complete!"
echo "➡️  New Database URL: postgres://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
