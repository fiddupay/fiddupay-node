#!/bin/bash

# PayFlow - Master Test Runner
# Comprehensive test suite for all test types

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
BASE_URL="http://localhost:8080"
TEST_DB="payflow_test"

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo -e "${PURPLE}ℹ${NC} $1"
}

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

run_test_category() {
    local category=$1
    local command=$2
    
    log "Running $category tests..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$command" > /tmp/test_${category}.log 2>&1; then
        success "$category tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        error "$category tests failed - check /tmp/test_${category}.log"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Help function
show_help() {
    echo "PayFlow Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --unit          Run unit tests only"
    echo "  --integration   Run integration tests only"
    echo "  --api          Run API tests only"
    echo "  --scripts      Run test scripts only"
    echo "  --all          Run all tests (default)"
    echo "  --setup        Setup test environment"
    echo "  --cleanup      Cleanup test environment"
    echo "  --help         Show this help"
    echo ""
}

# Setup test environment
setup_test_env() {
    log "Setting up test environment..."
    
    # Check if test database exists
    if ! psql -lqt | cut -d \| -f 1 | grep -qw $TEST_DB; then
        log "Creating test database: $TEST_DB"
        createdb $TEST_DB
    fi
    
    # Set test database URL
    export DATABASE_URL="postgresql://localhost/$TEST_DB"
    
    # Run migrations
    if command -v sqlx &> /dev/null; then
        log "Running database migrations..."
        sqlx migrate run
    else
        warning "sqlx-cli not found, skipping migrations"
    fi
    
    # Check if server is running
    if ! curl -s $BASE_URL/health > /dev/null 2>&1; then
        warning "PayFlow server not running at $BASE_URL"
        warning "Start server with: cargo run --release"
    fi
    
    success "Test environment ready"
}

# Cleanup test environment
cleanup_test_env() {
    log "Cleaning up test environment..."
    
    # Remove test database
    if psql -lqt | cut -d \| -f 1 | grep -qw $TEST_DB; then
        log "Dropping test database: $TEST_DB"
        dropdb $TEST_DB
    fi
    
    # Clean temporary files
    rm -f /tmp/test_*.log
    
    success "Test environment cleaned"
}

# Run unit tests
run_unit_tests() {
    log "Running unit tests..."
    cargo test --lib --tests --test utils_test --test standalone_tests
}

# Run integration tests
run_integration_tests() {
    log "Running integration tests..."
    cargo test --test payment_test --test services_test --test workflows_test --test database_integration_test --test withdrawal_test --test comprehensive_service_test --test full_integration_test --test payment_listing_tests --test analytics_service_tests
}

# Run API tests
run_api_tests() {
    log "Running API tests..."
    cargo test --test complete_endpoint_test
}

# Run test scripts
run_test_scripts() {
    log "Running test scripts..."
    
    # Basic API test
    if [ -f "tests/scripts/test_basic_api.sh" ]; then
        log "Running basic API tests..."
        bash tests/scripts/test_basic_api.sh
    fi
    
    # Complete flow test
    if [ -f "tests/scripts/test_complete_flow.sh" ]; then
        log "Running complete flow tests..."
        bash tests/scripts/test_complete_flow.sh
    fi
    
    # Service layer test
    if [ -f "tests/scripts/test_service_layer.sh" ]; then
        log "Running service layer tests..."
        bash tests/scripts/test_service_layer.sh
    fi
    
    # Sandbox workflow test
    if [ -f "tests/scripts/test_sandbox_workflow.sh" ]; then
        log "Running sandbox workflow tests..."
        bash tests/scripts/test_sandbox_workflow.sh
    fi
}

# Main execution
main() {
    echo "🧪 PayFlow Test Suite"
    echo "===================="
    echo ""
    
    case "${1:-all}" in
        --setup)
            setup_test_env
            ;;
        --cleanup)
            cleanup_test_env
            ;;
        --unit)
            setup_test_env
            run_test_category "Unit" "run_unit_tests"
            ;;
        --integration)
            setup_test_env
            run_test_category "Integration" "run_integration_tests"
            ;;
        --api)
            setup_test_env
            run_test_category "API" "run_api_tests"
            ;;
        --scripts)
            run_test_category "Scripts" "run_test_scripts"
            ;;
        --all)
            setup_test_env
            run_test_category "Unit" "run_unit_tests"
            run_test_category "Integration" "run_integration_tests"
            run_test_category "API" "run_api_tests"
            run_test_category "Scripts" "run_test_scripts"
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            show_help
            exit 1
            ;;
    esac
    
    # Show summary
    echo ""
    echo "📊 Test Summary"
    echo "==============="
    echo "Total test categories: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        success "All tests passed! 🎉"
        exit 0
    else
        error "Some tests failed! ❌"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
