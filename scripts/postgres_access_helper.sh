#!/bin/bash

# PostgreSQL Password Reset Helper
# Multiple methods to access PostgreSQL

echo "🔐 PostgreSQL Access Helper"
echo "==========================="
echo ""

echo "Method 1: Check if PostgreSQL is running"
echo "----------------------------------------"
sudo systemctl status postgresql || echo "PostgreSQL service not running"
echo ""

echo "Method 2: Try peer authentication (no password)"
echo "-----------------------------------------------"
echo "Attempting to connect as current user..."
if sudo -u postgres psql -c '\l' 2>/dev/null; then
    echo "✅ Peer authentication works!"
    echo ""
    echo "To reset postgres password:"
    echo "sudo -u postgres psql"
    echo "ALTER USER postgres PASSWORD 'newpassword';"
    echo "\q"
    exit 0
fi

echo "Method 3: Check PostgreSQL configuration"
echo "----------------------------------------"
PG_VERSION=$(sudo -u postgres psql --version | grep -oP '\d+\.\d+' | head -1)
PG_CONFIG="/etc/postgresql/$PG_VERSION/main/pg_hba.conf"

if [ -f "$PG_CONFIG" ]; then
    echo "PostgreSQL config found: $PG_CONFIG"
    echo ""
    echo "Current authentication methods:"
    grep -v '^#' "$PG_CONFIG" | grep -v '^$'
    echo ""
    echo "To enable password-less access temporarily:"
    echo "1. sudo nano $PG_CONFIG"
    echo "2. Change 'md5' to 'trust' for local connections"
    echo "3. sudo systemctl restart postgresql"
else
    echo "PostgreSQL config not found at expected location"
fi

echo ""
echo "Method 4: Reset password via sudo"
echo "---------------------------------"
echo "Run these commands:"
echo ""
echo "sudo -u postgres psql"
echo "ALTER USER postgres PASSWORD 'your_new_password';"
echo "\\q"
echo ""

echo "Method 5: Use environment variable"
echo "---------------------------------"
echo "export PGPASSWORD=your_password"
echo "Or create ~/.pgpass file:"
echo "echo 'localhost:5432:*:postgres:your_password' > ~/.pgpass"
echo "chmod 600 ~/.pgpass"
echo ""

echo "Method 6: Docker PostgreSQL (if using Docker)"
echo "---------------------------------------------"
if command -v docker &> /dev/null; then
    echo "Docker containers:"
    docker ps | grep postgres || echo "No PostgreSQL containers running"
    echo ""
    echo "To connect to Docker PostgreSQL:"
    echo "docker exec -it container_name psql -U postgres"
fi
