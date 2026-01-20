#!/bin/bash
# Test Verification Script
# Run this script to verify all implementations

set -e

echo "=================================="
echo "Crypto Payment Gateway - Test Suite"
echo "=================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Step 1: Checking Prerequisites${NC}"
echo "-----------------------------------"

# Check Rust
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓${NC} Rust/Cargo installed: $(cargo --version)"
else
    echo -e "${RED}✗${NC} Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

# Check PostgreSQL
if command -v psql &> /dev/null; then
    echo -e "${GREEN}✓${NC} PostgreSQL installed: $(psql --version)"
else
    echo -e "${YELLOW}⚠${NC} PostgreSQL not found. Install it to run the application."
fi

# Check Redis
if command -v redis-cli &> /dev/null; then
    echo -e "${GREEN}✓${NC} Redis installed: $(redis-cli --version)"
else
    echo -e "${YELLOW}⚠${NC} Redis not found. Install it to run the application."
fi

echo ""
echo -e "${YELLOW}Step 2: Running Unit Tests${NC}"
echo "-----------------------------------"

# Run all library tests
echo "Running library tests..."
cargo test --lib --no-fail-fast 2>&1 | tee test_output.log

# Run integration tests
echo ""
echo "Running integration tests..."
cargo test --test '*' --no-fail-fast 2>&1 | tee -a test_output.log

echo ""
echo -e "${YELLOW}Step 3: Test Summary${NC}"
echo "-----------------------------------"

# Count test results
PASSED=$(grep -c "test result: ok" test_output.log || echo "0")
FAILED=$(grep -c "test result: FAILED" test_output.log || echo "0")

echo "Tests Passed: $PASSED"
echo "Tests Failed: $FAILED"

if [ "$FAILED" -eq "0" ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
else
    echo -e "${RED}✗ Some tests failed. Check test_output.log for details.${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 4: Checking Code Compilation${NC}"
echo "-----------------------------------"

echo "Building project..."
if cargo build 2>&1 | tee build_output.log; then
    echo -e "${GREEN}✓ Project builds successfully${NC}"
else
    echo -e "${RED}✗ Build failed. Check build_output.log for details.${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 5: Verifying Migrations${NC}"
echo "-----------------------------------"

if [ -d "migrations" ]; then
    MIGRATION_COUNT=$(ls -1 migrations/*.sql 2>/dev/null | wc -l)
    echo -e "${GREEN}✓${NC} Found $MIGRATION_COUNT migration files"
    ls -1 migrations/*.sql
else
    echo -e "${RED}✗${NC} Migrations directory not found"
fi

echo ""
echo -e "${YELLOW}Step 6: Checking Configuration${NC}"
echo "-----------------------------------"

if [ -f ".env" ]; then
    echo -e "${GREEN}✓${NC} .env file exists"
    echo "Configuration variables:"
    grep -E "^[A-Z_]+=" .env | cut -d'=' -f1 | sed 's/^/  - /'
else
    echo -e "${YELLOW}⚠${NC} .env file not found. Copy .env.example to .env"
fi

echo ""
echo "=================================="
echo -e "${GREEN}Test Verification Complete!${NC}"
echo "=================================="
echo ""
echo "Next steps:"
echo "1. Review test_output.log for detailed test results"
echo "2. Review build_output.log for build details"
echo "3. Follow SETUP_INSTRUCTIONS.md to run the application"
echo ""
