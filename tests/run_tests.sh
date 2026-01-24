#!/bin/bash
# PayFlow - Streamlined Test Runner

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[$(date +'%H:%M:%S')] $1${NC}"; }
error() { echo -e "${RED}[$(date +'%H:%M:%S')] ERROR: $1${NC}"; }

run_unit_tests() {
    log "Running unit tests..."
    cargo test --test utils_test
    cargo test --test standalone_tests
}

run_integration_tests() {
    log "Running integration tests..."
    cargo test --test comprehensive_service_test
    cargo test --test payment_listing_tests
    cargo test --test workflows_test
    cargo test --test database_integration_test
    cargo test --test full_integration_test
    cargo test --test analytics_service_tests
    cargo test --test withdrawal_test
}

run_api_tests() {
    log "Running API tests..."
    cargo test --test complete_endpoint_test
}

run_test_scripts() {
    log "Running test scripts..."
    for script in tests/scripts/*.sh; do
        if [ -f "$script" ] && [ "$(basename "$script")" != "run_tests.sh" ]; then
            chmod +x "$script"
            log "Running $(basename "$script")..."
            "$script" || true
        fi
    done
}

main() {
    log "Starting PayFlow test suite..."
    
    case "${1:-all}" in
        "unit") run_unit_tests ;;
        "integration") run_integration_tests ;;
        "api") run_api_tests ;;
        "scripts") run_test_scripts ;;
        "all")
            run_unit_tests
            run_integration_tests
            run_api_tests
            run_test_scripts
            ;;
        *) echo "Usage: $0 [unit|integration|api|scripts|all]"; exit 1 ;;
    esac
    
    log "Test suite completed!"
}

main "$@"
