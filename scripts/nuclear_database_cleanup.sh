#!/bin/bash

# Nuclear Database Cleanup - Deletes ALL databases except system ones

set -e

echo "💥 NUCLEAR DATABASE CLEANUP"
echo "==========================="
echo ""
echo "⚠️  WARNING: This will DELETE ALL user databases!"
echo "System databases (postgres, template0, template1) will be preserved."
echo ""

read -p "🚨 Type 'NUKE' to delete ALL databases: " confirmation

if [ "$confirmation" != "NUKE" ]; then
    echo "❌ Nuclear cleanup cancelled."
    exit 1
fi

echo ""
echo "🔍 Finding all databases..."

# Get list of all databases except system ones
DATABASES=$(sudo -u postgres psql -t -c "
SELECT datname 
FROM pg_database 
WHERE datname NOT IN ('postgres', 'template0', 'template1')
AND datistemplate = false;
" | grep -v '^$' | xargs)

if [ -z "$DATABASES" ]; then
    echo "✅ No user databases found to delete."
    exit 0
fi

echo "📋 Found databases to delete:"
for db in $DATABASES; do
    echo "  - $db"
done

echo ""
read -p "🚨 Confirm deletion of these databases? Type 'DELETE ALL': " final_confirm

if [ "$final_confirm" != "DELETE ALL" ]; then
    echo "❌ Deletion cancelled."
    exit 1
fi

echo ""
echo "💥 Deleting all user databases..."

# Terminate all connections first
for db in $DATABASES; do
    echo "🔌 Terminating connections to $db..."
    sudo -u postgres psql -c "
    SELECT pg_terminate_backend(pid)
    FROM pg_stat_activity
    WHERE datname = '$db' AND pid <> pg_backend_pid();
    " > /dev/null 2>&1 || true
done

# Drop all databases
for db in $DATABASES; do
    echo "🗑️  Dropping database: $db"
    sudo -u postgres dropdb "$db" || echo "⚠️  Failed to drop $db (might not exist)"
done

echo ""
echo "🧹 Cleaning up remaining artifacts..."

# Clean up any remaining roles/users (except system ones)
sudo -u postgres psql -c "
DO \$\$
DECLARE
    r RECORD;
BEGIN
    FOR r IN (SELECT rolname FROM pg_roles WHERE rolname NOT IN ('postgres', 'pg_monitor', 'pg_read_all_settings', 'pg_read_all_stats', 'pg_stat_scan_tables', 'pg_read_server_files', 'pg_write_server_files', 'pg_execute_server_program', 'pg_signal_backend') AND rolname NOT LIKE 'pg_%')
    LOOP
        EXECUTE 'DROP ROLE IF EXISTS ' || quote_ident(r.rolname);
    END LOOP;
END
\$\$;
" > /dev/null 2>&1 || true

echo ""
echo "📊 Remaining databases:"
sudo -u postgres psql -l

echo ""
echo "✅ NUCLEAR CLEANUP COMPLETED!"
echo "🎯 All user databases have been obliterated!"
echo "💾 System databases preserved for PostgreSQL functionality."
