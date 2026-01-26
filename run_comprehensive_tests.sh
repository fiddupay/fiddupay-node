#!/bin/bash

# Comprehensive E2E Test Runner for FidduPay 3-Mode Wallet System
# Runs all integration tests with proper setup and teardown

set -e

echo " FidduPay Comprehensive E2E Test Suite"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
export RUST_LOG=info
export RUST_BACKTRACE=1
export TEST_MODE=true

# Database setup
echo -e "${BLUE} Setting up test database...${NC}"
export DATABASE_URL="postgresql://vibes:Soledayo%402001@localhost:5432/fiddupay_test"
export REDIS_URL="redis://localhost:6379"

# Security keys
export ENCRYPTION_KEY="fd4867a60ace984313bbeee057f586697f0f51063490c3b7d45536c83ee16525"
export JWT_SECRET="9c71f51199b7ea4b3e3f5a4c2f622260c41506b7f16c30f717bae5279f167c14"

# Change to backend directory for Rust tests
cd backend

# Working 2026 RPC endpoints
export ETHEREUM_RPC_URL="https://eth.llamarpc.com"
export BSC_RPC_URL="https://bsc-dataseed.binance.org"
export POLYGON_RPC_URL="https://polygon-rpc.com"
export ARBITRUM_RPC_URL="https://arb1.arbitrum.io/rpc"
export SOLANA_RPC_URL="https://api.mainnet-beta.solana.com"

echo -e "${GREEN} Environment configured${NC}"

# Function to run test with error handling
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -e "\n${BLUE} Running: $test_name${NC}"
    echo "----------------------------------------"
    
    if eval $test_command; then
        echo -e "${GREEN} $test_name PASSED${NC}"
        return 0
    else
        echo -e "${RED} $test_name FAILED${NC}"
        return 1
    fi
}

# Test suite execution
failed_tests=0
total_tests=0

echo -e "\n${YELLOW} Starting Comprehensive Test Suite${NC}"

# 1. Unit Tests
total_tests=$((total_tests + 1))
if ! run_test "Unit Tests" "cargo test --lib -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 2. RPC Gas Fee Tests
total_tests=$((total_tests + 1))
if ! run_test "RPC Gas Fee Tests" "cd .. && python3 test_rpc_gas_fees.py"; then
    failed_tests=$((failed_tests + 1))
fi

# 3. Address-Only Mode Tests
total_tests=$((total_tests + 1))
if ! run_test "Address-Only Mode E2E" "cargo test comprehensive_e2e_wallet_modes::test_mode_1_address_only_complete_flow -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 4. Gateway-Generated Mode Tests
total_tests=$((total_tests + 1))
if ! run_test "Gateway-Generated Mode E2E" "cargo test comprehensive_e2e_wallet_modes::test_mode_2_gateway_generated_complete_flow -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 5. Imported Key Mode Tests
total_tests=$((total_tests + 1))
if ! run_test "Imported Key Mode E2E" "cargo test comprehensive_e2e_wallet_modes::test_mode_3_imported_key_complete_flow -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 6. WebSocket Integration Tests
total_tests=$((total_tests + 1))
if ! run_test "WebSocket Integration" "cargo test websocket_integration_tests -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 7. API Integration Tests
total_tests=$((total_tests + 1))
if ! run_test "API Integration" "cargo test api_integration_tests -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 8. Error Handling Tests
total_tests=$((total_tests + 1))
if ! run_test "Comprehensive Error Handling" "cargo test comprehensive_e2e_wallet_modes::test_comprehensive_error_handling -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 9. Multi-Currency Support Tests
total_tests=$((total_tests + 1))
if ! run_test "Multi-Currency Support" "cargo test comprehensive_e2e_wallet_modes::test_multi_currency_support -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 10. Performance and Concurrency Tests
total_tests=$((total_tests + 1))
if ! run_test "Performance & Concurrency" "cargo test comprehensive_e2e_wallet_modes::test_performance_and_concurrency -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 11. Real-time Gas Monitoring Tests
total_tests=$((total_tests + 1))
if ! run_test "Real-time Gas Monitoring" "cargo test websocket_integration_tests::test_real_time_gas_price_monitoring -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# 12. Blockchain Transaction Tests
total_tests=$((total_tests + 1))
if ! run_test "Blockchain Transactions" "cargo test address_only_integration_test -- --nocapture"; then
    failed_tests=$((failed_tests + 1))
fi

# Test Results Summary
echo -e "\n${YELLOW} Test Suite Results${NC}"
echo "========================================"
echo -e "Total Tests: ${BLUE}$total_tests${NC}"
echo -e "Passed: ${GREEN}$((total_tests - failed_tests))${NC}"
echo -e "Failed: ${RED}$failed_tests${NC}"

if [ $failed_tests -eq 0 ]; then
    echo -e "\n${GREEN} ALL TESTS PASSED! ${NC}"
    echo -e "${GREEN} FidduPay 3-Mode Wallet System is ready for production!${NC}"
    
    echo -e "\n${BLUE} Test Coverage Summary:${NC}"
    echo " Mode 1: Address-Only with auto-forwarding"
    echo " Mode 2: Gateway-Generated wallets"
    echo " Mode 3: Imported private keys"
    echo " Payment creation and processing"
    echo " Fee collection and calculation"
    echo " Withdrawal processing"
    echo " Gas fee estimation (2026 RPC methods)"
    echo " WebSocket real-time updates"
    echo " API endpoints and authentication"
    echo " Error handling and validation"
    echo " Multi-currency support (5 networks)"
    echo " Performance and concurrency"
    echo " Blockchain transaction sending"
    
    exit 0
else
    echo -e "\n${RED} $failed_tests TEST(S) FAILED${NC}"
    echo -e "${YELLOW}⚠️ Please review failed tests before deployment${NC}"
    
    echo -e "\n${BLUE} Troubleshooting Tips:${NC}"
    echo "1. Check database connectivity"
    echo "2. Verify RPC endpoints are accessible"
    echo "3. Ensure Redis is running"
    echo "4. Check environment variables"
    echo "5. Review test logs for specific errors"
    
    exit 1
fi
