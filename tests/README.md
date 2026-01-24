# PayFlow Test Suite

Comprehensive test suite for PayFlow cryptocurrency payment gateway.

## 📁 Test Structure

```
tests/
├── run_tests.sh              # Master test runner
├── fixtures/                 # Test data and fixtures
│   └── test_data.sql         # Test database fixtures
├── unit/                     # Unit tests
│   ├── utils_test.rs         # Utility function tests
│   └── standalone_tests.rs   # Standalone component tests
├── integration/              # Integration tests
│   ├── payment_test.rs       # Payment flow tests
│   ├── services_test.rs      # Service layer tests
│   ├── workflows_test.rs     # End-to-end workflow tests
│   ├── database_integration_test.rs # Database tests
│   ├── withdrawal_test.rs    # Withdrawal tests
│   ├── comprehensive_service_test.rs # Service integration
│   ├── full_integration_test.rs # Full system integration
│   ├── payment_listing_tests.rs # Payment listing tests
│   └── analytics_service_tests.rs # Analytics tests
├── api/                      # API endpoint tests
│   └── complete_endpoint_test.rs # Complete API test suite
└── scripts/                  # Test scripts
    ├── test.sh               # Main test script
    ├── test_basic_api.sh     # Basic API functionality
    ├── test_complete_flow.sh # End-to-end flow
    ├── test_service_layer.sh # Service layer testing
    ├── test_redis.sh         # Redis functionality
    └── run_tests.sh          # Legacy test runner
```

## 🚀 Quick Start

### Run All Tests
```bash
# Run complete test suite
./tests/run_tests.sh

# Or use the --all flag explicitly
./tests/run_tests.sh --all
```

### Run Specific Test Categories
```bash
# Unit tests only
./tests/run_tests.sh --unit

# Integration tests only
./tests/run_tests.sh --integration

# API tests only
./tests/run_tests.sh --api

# Test scripts only
./tests/run_tests.sh --scripts
```

### Environment Management
```bash
# Setup test environment
./tests/run_tests.sh --setup

# Cleanup test environment
./tests/run_tests.sh --cleanup
```

## 🧪 Test Categories

### Unit Tests
- **utils_test.rs**: Utility functions (encryption, key generation)
- **standalone_tests.rs**: Individual component tests

### Integration Tests
- **payment_test.rs**: Payment creation and processing
- **services_test.rs**: Service layer interactions
- **workflows_test.rs**: Complete business workflows
- **database_integration_test.rs**: Database operations
- **withdrawal_test.rs**: Withdrawal processing
- **comprehensive_service_test.rs**: Multi-service integration
- **full_integration_test.rs**: System-wide integration
- **payment_listing_tests.rs**: Payment listing and filtering
- **analytics_service_tests.rs**: Analytics and reporting

### API Tests
- **complete_endpoint_test.rs**: All API endpoints with authentication

### Test Scripts
- **test_basic_api.sh**: Basic API functionality verification
- **test_complete_flow.sh**: End-to-end merchant workflow
- **test_service_layer.sh**: Service layer testing
- **test_redis.sh**: Redis functionality testing

## 🔧 Test Environment

### Prerequisites
- PostgreSQL running
- Redis running (for some tests)
- PayFlow server running on localhost:8080

### Environment Variables
```bash
export DATABASE_URL="postgresql://localhost/payflow_test"
export REDIS_URL="redis://localhost:6379"
export ENCRYPTION_KEY="test_key_32_bytes_long_for_testing"
export WEBHOOK_SIGNING_KEY="test_webhook_key_32_bytes_long"
```

### Test Database
The test suite automatically creates and manages a test database:
- **Database**: `payflow_test`
- **Auto-created**: Yes
- **Auto-migrated**: Yes
- **Auto-cleaned**: Optional

## 📊 Test Coverage

### Current Coverage
- **Unit Tests**: Core utilities and components
- **Integration Tests**: Service interactions and workflows
- **API Tests**: All HTTP endpoints
- **End-to-End Tests**: Complete user journeys

### Test Metrics
- **Total Test Files**: 15
- **Test Categories**: 4 (Unit, Integration, API, Scripts)
- **API Endpoints Covered**: 28+
- **Service Methods Covered**: 100+

## 🛠️ Running Individual Tests

### Cargo Tests
```bash
# Run specific test file
cargo test --test payment_test

# Run specific test function
cargo test test_create_payment

# Run with output
cargo test -- --nocapture

# Run with specific database
DATABASE_URL=postgresql://localhost/payflow_test cargo test
```

### Script Tests
```bash
# Run specific script
bash tests/scripts/test_basic_api.sh

# Run with debug output
DEBUG=1 bash tests/scripts/test_complete_flow.sh
```

## 🔍 Debugging Tests

### Test Logs
Test logs are saved to `/tmp/test_*.log` files:
```bash
# View unit test logs
cat /tmp/test_Unit.log

# View integration test logs
cat /tmp/test_Integration.log

# View API test logs
cat /tmp/test_API.log
```

### Common Issues

#### Database Connection
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Create test database manually
createdb payflow_test

# Run migrations manually
DATABASE_URL=postgresql://localhost/payflow_test sqlx migrate run
```

#### Server Not Running
```bash
# Start PayFlow server
cargo run --release

# Check server health
curl http://localhost:8080/health
```

#### Redis Connection
```bash
# Check Redis status
redis-cli ping

# Start Redis if needed
redis-server --daemonize yes
```

## 📈 Performance Testing

### Load Testing
```bash
# Install wrk
sudo apt install wrk

# Run load test
wrk -t12 -c400 -d30s http://localhost:8080/health
```

### Memory Testing
```bash
# Run with memory profiling
cargo test --release -- --nocapture

# Check for memory leaks
valgrind --tool=memcheck cargo test
```

## 🎯 Best Practices

### Writing Tests
- Use descriptive test names
- Include setup and teardown
- Test both success and failure cases
- Use realistic test data
- Mock external dependencies

### Test Data
- Use unique identifiers (timestamps, UUIDs)
- Clean up after tests
- Use test fixtures for complex data
- Avoid hardcoded values

### Assertions
- Test expected behavior
- Verify error conditions
- Check side effects
- Validate data integrity

## 🔄 Continuous Integration

### GitHub Actions
The test suite is designed to work with CI/CD:
```yaml
- name: Run tests
  run: ./tests/run_tests.sh --all
```

### Test Reports
- Test results logged to files
- Exit codes indicate success/failure
- Summary statistics provided

## 📞 Support

For test-related issues:
- Check test logs in `/tmp/test_*.log`
- Verify environment setup
- Ensure all prerequisites are running
- Review test documentation

---

**Last Updated**: 2026-01-24  
**Test Suite Version**: 1.0.0
