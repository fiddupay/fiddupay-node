#!/bin/bash

# FidduPay Merchant API Test Runner
# Comprehensive test suite for all merchant-related endpoints

set -e

echo "🚀 FidduPay Merchant API Test Runner"
echo "===================================="

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 16+ to run tests."
    exit 1
fi

# Check if backend server is running
echo "🔍 Checking if FidduPay backend is running..."
if ! curl -s http://127.0.0.1:8080/health > /dev/null; then
    echo "❌ FidduPay backend server is not running on http://127.0.0.1:8080"
    echo "   Please start the backend server first:"
    echo "   cd backend && cargo run"
    exit 1
fi

echo "✅ Backend server is running"

# Navigate to tests directory
cd tests

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing test dependencies..."
    npm install
fi

# Parse command line arguments
TEST_TYPE="comprehensive"
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --quick|-q)
            TEST_TYPE="quick"
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --quick, -q     Run quick test suite (faster)"
            echo "  --verbose, -v   Enable verbose output"
            echo "  --help, -h      Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Run comprehensive tests"
            echo "  $0 --quick           # Run quick tests"
            echo "  $0 --verbose         # Run with verbose output"
            echo "  $0 --quick --verbose # Run quick tests with verbose output"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set environment variables for verbose mode
if [ "$VERBOSE" = true ]; then
    export DEBUG="*"
fi

# Run the appropriate test suite
echo ""
if [ "$TEST_TYPE" = "quick" ]; then
    echo "⚡ Running Quick Merchant API Tests..."
    node merchant-api-quick.js
else
    echo "🧪 Running Comprehensive Merchant API Tests..."
    node merchant-api-comprehensive.js
fi

# Capture exit code
TEST_EXIT_CODE=$?

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "🎉 All tests completed successfully!"
else
    echo "💥 Some tests failed. Check the output above for details."
fi

exit $TEST_EXIT_CODE