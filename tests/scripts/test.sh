#!/bin/bash
# PayFlow - Comprehensive Test Runner

set -e

echo "🧪 PayFlow Test Suite"
echo "===================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if test database exists
echo "📦 Checking test database..."
if ! psql -lqt | cut -d \| -f 1 | grep -qw payflow_test; then
    echo "${YELLOW}Creating test database...${NC}"
    createdb payflow_test
fi

# Run migrations on test database
echo "🔄 Running migrations..."
export DATABASE_URL="postgresql://vibes:Soledayo@2001@localhost:5432/payflow_test"

if command -v sqlx &> /dev/null; then
    sqlx migrate run
else
    echo "${YELLOW}⚠ sqlx-cli not installed. Running migrations manually...${NC}"
    PGPASSWORD="Soledayo@2001" psql -h localhost -U vibes -d payflow_test -c "SELECT 1" > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        for migration in migrations/*.sql; do
            echo "  Running $(basename $migration)..."
            PGPASSWORD="Soledayo@2001" psql -h localhost -U vibes -d payflow_test -f $migration 2>&1 | grep -v "already exists" || true
        done
    else
        echo "${RED}✗ Cannot connect to database${NC}"
        exit 1
    fi
fi

# Set test environment variables
export ENCRYPTION_KEY="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
export WEBHOOK_SIGNING_KEY="test_webhook_key_0123456789abcdef"
export RUST_LOG=error
export RUST_BACKTRACE=1

echo ""
echo "🧪 Running Unit Tests..."
echo "------------------------"
cargo test --lib -- --nocapture

echo ""
echo "🔗 Running Integration Tests..."
echo "--------------------------------"
cargo test --test '*' -- --nocapture

echo ""
echo "📊 Generating Coverage Report..."
echo "---------------------------------"
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Html --output-dir coverage --exclude-files 'tests/*'
    echo "${GREEN}✓ Coverage report generated: coverage/index.html${NC}"
else
    echo "${YELLOW}⚠ cargo-tarpaulin not installed. Skipping coverage.${NC}"
    echo "  Install with: cargo install cargo-tarpaulin"
fi

echo ""
echo "${GREEN}✅ All tests completed!${NC}"
echo ""
echo "Summary:"
echo "--------"
cargo test --lib 2>&1 | grep "test result:"
cargo test --test '*' 2>&1 | grep "test result:" || true

echo ""
echo "Next steps:"
echo "1. Review test results above"
echo "2. Check coverage report: open coverage/index.html"
echo "3. Fix any failing tests"
echo "4. Run: ./scripts/test.sh before each commit"
