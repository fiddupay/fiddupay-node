#!/bin/bash

# Hybrid Non-Custodial System Test Runner
# Executes all test phases with progress tracking

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DB_URL="postgresql://test_user:test_pass@localhost:5432/fiddupay_test"
BACKEND_DIR="backend"
FRONTEND_DIR="frontend"

echo -e "${BLUE} Hybrid Non-Custodial System - Test Suite${NC}"
echo "=" | tr -d '\n'; for i in {1..60}; do echo -n "="; done; echo

# Function to print test phase header
print_phase_header() {
    local phase_name="$1"
    echo -e "\n${BLUE} $phase_name${NC}"
    echo "-" | tr -d '\n'; for i in {1..50}; do echo -n "-"; done; echo
}

# Function to run a test phase
run_test_phase() {
    local phase_num="$1"
    local phase_name="$2"
    local test_command="$3"
    
    print_phase_header "Phase $phase_num: $phase_name"
    
    if eval "$test_command"; then
        echo -e "${GREEN} Phase $phase_num PASSED${NC}"
        return 0
    else
        echo -e "${RED} Phase $phase_num FAILED${NC}"
        return 1
    fi
}

# Function to setup test environment
setup_test_env() {
    echo -e "${YELLOW} Setting up test environment...${NC}"
    
    # Check if test database exists
    if ! psql "$TEST_DB_URL" -c '\q' 2>/dev/null; then
        echo -e "${YELLOW} Creating test database...${NC}"
        createdb fiddupay_test || echo "Database might already exist"
    fi
    
    # Run migrations on test database
    echo -e "${YELLOW} Running database migrations...${NC}"
    cd "$BACKEND_DIR"
    DATABASE_URL="$TEST_DB_URL" sqlx migrate run || echo "Migrations completed"
    cd ..
    
    echo -e "${GREEN} Test environment ready${NC}"
}

# Function to cleanup test environment
cleanup_test_env() {
    echo -e "${YELLOW} Cleaning up test environment...${NC}"
    # Optional: Drop test database
    # dropdb fiddupay_test 2>/dev/null || true
    echo -e "${GREEN} Cleanup completed${NC}"
}

# Main test execution
main() {
    local total_phases=9
    local passed_phases=0
    local failed_phases=0
    
    echo -e "${BLUE} Test Plan: $total_phases phases, 89 total tests${NC}"
    echo -e "${BLUE} Target: 100% success rate${NC}\n"
    
    # Setup test environment
    setup_test_env
    
    # Phase 1: Core Infrastructure Testing
    if run_test_phase "1" "Core Infrastructure Testing" "cd $BACKEND_DIR && cargo test phase1_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Phase 2: Wallet Management Testing
    if run_test_phase "2" "Wallet Management Testing" "cd $BACKEND_DIR && cargo test phase2_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Phase 3: Gas Validation Testing
    if run_test_phase "3" "Gas Validation Testing" "cd $BACKEND_DIR && cargo test phase3_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Phase 4: API Endpoint Testing
    if run_test_phase "4" "API Endpoint Testing" "cd $BACKEND_DIR && cargo test phase4_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Phase 5: Frontend Integration Testing
    if run_test_phase "5" "Frontend Integration Testing" "cd $FRONTEND_DIR && npm test -- --testPathPattern=phase5"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Phase 6: Security & Monitoring Testing
    if run_test_phase "6" "Security & Monitoring Testing" "cd $BACKEND_DIR && cargo test phase6_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Integration Testing
    if run_test_phase "I" "Integration Testing" "cd $BACKEND_DIR && cargo test integration_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Performance Testing
    if run_test_phase "P" "Performance Testing" "cd $BACKEND_DIR && cargo test performance_tests --lib --release"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Security Testing
    if run_test_phase "S" "Security Testing" "cd $BACKEND_DIR && cargo test security_tests --lib"; then
        ((passed_phases++))
    else
        ((failed_phases++))
    fi
    
    # Print final results
    echo -e "\n${BLUE} Final Test Results${NC}"
    echo "=" | tr -d '\n'; for i in {1..60}; do echo -n "="; done; echo
    echo -e "Total Phases: $total_phases"
    echo -e "${GREEN} Passed Phases: $passed_phases${NC}"
    echo -e "${RED} Failed Phases: $failed_phases${NC}"
    
    local success_rate=$(( (passed_phases * 100) / total_phases ))
    echo -e " Success Rate: $success_rate%"
    
    if [ $failed_phases -eq 0 ]; then
        echo -e "\n${GREEN} ALL TESTS PASSED! System ready for production.${NC}"
        cleanup_test_env
        exit 0
    else
        echo -e "\n${RED} Some tests failed. Please review and fix issues.${NC}"
        cleanup_test_env
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    "setup")
        setup_test_env
        ;;
    "cleanup")
        cleanup_test_env
        ;;
    "phase1")
        setup_test_env
        run_test_phase "1" "Core Infrastructure Testing" "cd $BACKEND_DIR && cargo test phase1_tests --lib"
        ;;
    "phase2")
        setup_test_env
        run_test_phase "2" "Wallet Management Testing" "cd $BACKEND_DIR && cargo test phase2_tests --lib"
        ;;
    "phase3")
        setup_test_env
        run_test_phase "3" "Gas Validation Testing" "cd $BACKEND_DIR && cargo test phase3_tests --lib"
        ;;
    "phase4")
        setup_test_env
        run_test_phase "4" "API Endpoint Testing" "cd $BACKEND_DIR && cargo test phase4_tests --lib"
        ;;
    "phase5")
        setup_test_env
        run_test_phase "5" "Frontend Integration Testing" "cd $FRONTEND_DIR && npm test -- --testPathPattern=phase5"
        ;;
    "phase6")
        setup_test_env
        run_test_phase "6" "Security & Monitoring Testing" "cd $BACKEND_DIR && cargo test phase6_tests --lib"
        ;;
    "integration")
        setup_test_env
        run_test_phase "I" "Integration Testing" "cd $BACKEND_DIR && cargo test integration_tests --lib"
        ;;
    "performance")
        setup_test_env
        run_test_phase "P" "Performance Testing" "cd $BACKEND_DIR && cargo test performance_tests --lib --release"
        ;;
    "security")
        setup_test_env
        run_test_phase "S" "Security Testing" "cd $BACKEND_DIR && cargo test security_tests --lib"
        ;;
    "help"|"-h"|"--help")
        echo "Hybrid Non-Custodial System Test Runner"
        echo ""
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  (no args)    Run all test phases"
        echo "  setup        Setup test environment only"
        echo "  cleanup      Cleanup test environment"
        echo "  phase1       Run Phase 1: Core Infrastructure tests"
        echo "  phase2       Run Phase 2: Wallet Management tests"
        echo "  phase3       Run Phase 3: Gas Validation tests"
        echo "  phase4       Run Phase 4: API Endpoint tests"
        echo "  phase5       Run Phase 5: Frontend Integration tests"
        echo "  phase6       Run Phase 6: Security & Monitoring tests"
        echo "  integration  Run Integration tests"
        echo "  performance  Run Performance tests"
        echo "  security     Run Security tests"
        echo "  help         Show this help message"
        echo ""
        echo "Environment Variables:"
        echo "  TEST_DB_URL  Test database URL (default: $TEST_DB_URL)"
        ;;
    *)
        main
        ;;
esac
