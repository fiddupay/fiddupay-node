#!/bin/bash

# FidduPay Comprehensive Test Runner
# Runs Admin API, Sandbox API, and SDK tests

set -e

echo "🚀 FidduPay Comprehensive Test Suite Runner"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test suite
run_test_suite() {
    local test_name="$1"
    local test_command="$2"
    local test_dir="$3"
    
    echo -e "\n${BLUE}🧪 Running $test_name...${NC}"
    echo "----------------------------------------"
    
    if [ -n "$test_dir" ]; then
        cd "$test_dir"
    fi
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ $test_name: PASSED${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}❌ $test_name: FAILED${NC}"
        ((FAILED_TESTS++))
    fi
    
    ((TOTAL_TESTS++))
    
    # Return to original directory
    cd - > /dev/null 2>&1 || true
}

# Check if backend is running
echo "🔍 Checking if FidduPay backend is running..."
if curl -s http://localhost:8080/health > /dev/null; then
    echo -e "${GREEN}✅ Backend server is running${NC}"
else
    echo -e "${RED}❌ Backend server is not running. Please start it first.${NC}"
    exit 1
fi

# Parse command line arguments
ADMIN_ONLY=false
SANDBOX_ONLY=false
SDK_ONLY=false
QUICK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --admin)
            ADMIN_ONLY=true
            shift
            ;;
        --sandbox)
            SANDBOX_ONLY=true
            shift
            ;;
        --sdk)
            SDK_ONLY=true
            shift
            ;;
        --quick)
            QUICK=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --admin     Run only Admin API tests"
            echo "  --sandbox   Run only Sandbox API tests"
            echo "  --sdk       Run only SDK tests"
            echo "  --quick     Run quick version of tests"
            echo "  --help      Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run tests based on arguments
if [ "$ADMIN_ONLY" = true ]; then
    run_test_suite "Admin API Tests" "node tests/admin-api-comprehensive.js" "/home/vibes/crypto-payment-gateway"
elif [ "$SANDBOX_ONLY" = true ]; then
    run_test_suite "Sandbox API Tests" "node tests/sandbox-api-comprehensive.js" "/home/vibes/crypto-payment-gateway"
elif [ "$SDK_ONLY" = true ]; then
    run_test_suite "SDK Tests" "npm test -- sdk-comprehensive.test.ts" "/home/vibes/crypto-payment-gateway/fiddupay-node-sdk"
else
    # Run all test suites
    echo -e "\n${YELLOW}📋 Running All Comprehensive Test Suites${NC}"
    
    # 1. Admin API Tests
    run_test_suite "Admin API Tests" "node tests/admin-api-comprehensive.js" "/home/vibes/crypto-payment-gateway"
    
    # 2. Sandbox API Tests  
    run_test_suite "Sandbox API Tests" "node tests/sandbox-api-comprehensive.js" "/home/vibes/crypto-payment-gateway"
    
    # 3. SDK Tests
    run_test_suite "SDK Comprehensive Tests" "npm test -- sdk-comprehensive.test.ts" "/home/vibes/crypto-payment-gateway/fiddupay-node-sdk"
    
    # 4. Existing Merchant Tests (for completeness)
    if [ "$QUICK" = false ]; then
        run_test_suite "Merchant API Tests" "./run-merchant-tests.sh" "/home/vibes/crypto-payment-gateway"
    fi
fi

# Final results
echo -e "\n${BLUE}=============================================="
echo "📊 COMPREHENSIVE TEST RESULTS SUMMARY"
echo -e "==============================================${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All test suites passed!${NC}"
    echo -e "${GREEN}✅ Passed: $PASSED_TESTS/$TOTAL_TESTS test suites${NC}"
    echo -e "${GREEN}📈 Success Rate: 100.0%${NC}"
else
    echo -e "${RED}❌ Some test suites failed${NC}"
    echo -e "${GREEN}✅ Passed: $PASSED_TESTS/$TOTAL_TESTS test suites${NC}"
    echo -e "${RED}❌ Failed: $FAILED_TESTS/$TOTAL_TESTS test suites${NC}"
    
    SUCCESS_RATE=$(echo "scale=1; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l)
    echo -e "${YELLOW}📈 Success Rate: ${SUCCESS_RATE}%${NC}"
fi

echo -e "\n${BLUE}🎯 Test suite execution completed!${NC}"

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    exit 0
else
    exit 1
fi
