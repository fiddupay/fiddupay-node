#!/bin/bash
set -e

# Configuration
DB_NAME="fiddupay"
DB_USER="fiddupay_user"
# Generate a random secure password
DB_PASS=$(openssl rand -base64 12)

echo "🛑 Stopping backend service..."
pm2 stop fiddupay-backend || true

echo "🗑️  Dropping existing database and user..."
sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"
sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;"

echo "✨ Creating new secure user and database..."
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"

# Update .env file with new password
echo "📝 Updating .env file..."
# Escape special characters for sed
ESCAPED_PASS=$(printf '%s\n' "$DB_PASS" | sed -e 's/[\/&]/\\&/g')
sed -i "s/DATABASE_URL=.*/DATABASE_URL=postgres:\/\/$DB_USER:$ESCAPED_PASS@localhost:5432\/$DB_NAME/g" .env

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
