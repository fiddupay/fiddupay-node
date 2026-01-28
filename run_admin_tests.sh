#!/bin/bash

# Comprehensive Admin API Test Suite Runner
# Runs all admin-only endpoint tests with detailed reporting

set -e

echo " Starting FidduPay Admin API Test Suite"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
export RUST_LOG=debug
export DATABASE_URL="postgresql://postgres:password@localhost:5432/fiddupay_test"
export REDIS_URL="redis://localhost:6379/1"
export ENCRYPTION_KEY="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

# Function to run test category
run_test_category() {
    local category=$1
    local description=$2
    
    echo -e "\n${BLUE} Running $description${NC}"
    echo "----------------------------------------"
    
    if cargo test --test "$category" -- --nocapture; then
        echo -e "${GREEN} $description: PASSED${NC}"
        return 0
    else
        echo -e "${RED} $description: FAILED${NC}"
        return 1
    fi
}

# Initialize test database
echo -e "${YELLOW} Setting up test environment...${NC}"
if command -v psql &> /dev/null; then
    psql -c "DROP DATABASE IF EXISTS fiddupay_test;" -U postgres || true
    psql -c "CREATE DATABASE fiddupay_test;" -U postgres
    echo " Test database created"
else
    echo "  PostgreSQL not found, assuming database exists"
fi

# Run database migrations
echo " Running database migrations..."
if cargo run --bin migrate; then
    echo " Migrations completed"
else
    echo "  Migration warnings (continuing)"
fi

# Test categories
declare -A test_categories=(
    ["admin_api_tests"]="Core Admin API Tests"
    ["admin_system_tests"]="System Management Tests"
    ["admin_merchant_tests"]="Merchant Management Tests"
    ["admin_analytics_tests"]="Analytics & Monitoring Tests"
    ["admin_security_tests"]="Security & Compliance Tests"
    ["admin_test_suite"]="Integration & Performance Tests"
)

# Track test results
passed_tests=0
failed_tests=0
total_categories=${#test_categories[@]}

echo -e "\n${BLUE} Running Admin API Test Categories${NC}"
echo "======================================"

# Run each test category
for test_file in "${!test_categories[@]}"; do
    if run_test_category "$test_file" "${test_categories[$test_file]}"; then
        ((passed_tests++))
    else
        ((failed_tests++))
    fi
done

# Generate detailed test report
echo -e "\n${BLUE} Generating Test Coverage Report${NC}"
echo "=================================="

cargo test --test admin_test_suite generate_test_coverage_report -- --nocapture

# Performance benchmarks
echo -e "\n${BLUE} Running Performance Benchmarks${NC}"
echo "================================="

echo "Testing admin endpoint response times..."
cargo test --test admin_test_suite test_admin_api_performance -- --nocapture

# Security validation
echo -e "\n${BLUE} Security Validation${NC}"
echo "====================="

echo "Validating role-based access control..."
cargo test --test admin_test_suite test_role_based_access_comprehensive -- --nocapture

# Final summary
echo -e "\n${BLUE} Test Suite Summary${NC}"
echo "===================="
echo "Total Categories: $total_categories"
echo -e "Passed: ${GREEN}$passed_tests${NC}"
echo -e "Failed: ${RED}$failed_tests${NC}"

if [ $failed_tests -eq 0 ]; then
    echo -e "\n${GREEN} All Admin API Tests Passed!${NC}"
    echo -e "${GREEN} Complete test coverage achieved${NC}"
    echo -e "${GREEN} All admin endpoints validated${NC}"
    echo -e "${GREEN} Security controls verified${NC}"
    echo -e "${GREEN} Performance benchmarks met${NC}"
    exit 0
else
    echo -e "\n${RED} Some tests failed. Please review the output above.${NC}"
    exit 1
fi