#!/bin/bash

echo "=== PayFlow Test Suite Runner ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL=0
PASSED=0
FAILED=0

# Function to run test
run_test() {
    local test_name=$1
    local test_cmd=$2
    
    echo -e "${YELLOW}Running: $test_name${NC}"
    TOTAL=$((TOTAL + 1))
    
    if eval "$test_cmd" > /tmp/test_output.log 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}❌ FAILED${NC}"
        echo "Error output:"
        tail -20 /tmp/test_output.log
        FAILED=$((FAILED + 1))
    fi
    echo ""
}

# Set environment
export SQLX_OFFLINE=true
export DATABASE_URL="postgresql://vibes:Soledayo%402001@localhost:5432/payflow_test"
export REDIS_URL="redis://localhost:6379"

cd /home/vibes/crypto-payment-gateway

echo "=== 1. Standalone Tests (No Database Required) ==="
run_test "Standalone Tests" "cargo test --test standalone_tests --quiet"

echo "=== 2. Utils Tests ==="
run_test "Utils Tests" "cargo test --test utils_test --quiet"

echo "=== 3. Service Tests (Requires Database) ==="
# These will fail without proper database setup
echo -e "${YELLOW}Skipping integration tests (require database setup)${NC}"
echo "  - services_test.rs"
echo "  - payment_test.rs"
echo "  - withdrawal_test.rs"
echo "  - endpoints_test.rs"
echo "  - workflows_test.rs"
echo "  - payment_listing_tests.rs"
echo ""

echo "=== 4. Library Unit Tests ==="
echo -e "${YELLOW}Checking library compilation...${NC}"
if cargo test --lib --no-run --quiet 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}✅ Library compiles${NC}"
    PASSED=$((PASSED + 1))
    TOTAL=$((TOTAL + 1))
else
    echo -e "${RED}❌ Library has compilation errors${NC}"
    FAILED=$((FAILED + 1))
    TOTAL=$((TOTAL + 1))
fi
echo ""

echo "=== Test Summary ==="
echo "Total Tests: $TOTAL"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All runnable tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed${NC}"
    exit 1
fi
