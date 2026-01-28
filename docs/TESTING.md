# fiddupay Testing Guide

Complete testing guide for fiddupay cryptocurrency payment gateway.

## Test Structure

```
tests/
 api_endpoints_test.rs        # API endpoint tests
 payment_test.rs              # Payment flow tests
 withdrawal_test.rs           # Withdrawal tests
 services_test.rs             # Service layer tests
 utils_test.rs                # Utility tests
 workflows_test.rs            # End-to-end workflows
 complete_endpoint_test.rs    # Complete API tests
 comprehensive_service_test.rs # Service integration
 database_integration_test.rs # Database tests
 endpoints_test.rs            # Endpoint validation
 full_integration_test.rs     # Full integration
 payment_listing_tests.rs     # Payment listing
 analytics_service_tests.rs   # Analytics tests
 standalone_tests.rs          # Standalone tests
```

## Running Tests

### Unit Tests
```bash
# All tests
cargo test

# Specific test file
cargo test --test payment_test

# Specific test function
cargo test test_create_payment
```

### Integration Tests
```bash
# All integration tests
cargo test --test '*'

# Service layer tests
cargo test --test services_test

# Database tests
cargo test --test database_integration_test
```

### API Tests
```bash
# Complete endpoint tests
cargo test --test complete_endpoint_test

# API endpoint tests
cargo test --test api_endpoints_test
```

### Test Scripts
```bash
# Make scripts executable
chmod +x test_*.sh

# Basic API functionality
./test_basic_api.sh

# Complete payment flow
./test_complete_flow.sh

# Service layer testing
./test_service_layer.sh

# Final comprehensive test
./test_final_complete.sh
```

## Test Categories

### 1. Unit Tests
- Individual function testing
- Service method validation
- Utility function verification
- Model serialization/deserialization

### 2. Integration Tests
- Service interaction testing
- Database integration
- External API mocking
- Error handling validation

### 3. API Tests
- HTTP endpoint testing
- Authentication validation
- Request/response validation
- Error response testing

### 4. End-to-End Tests
- Complete workflow testing
- Multi-service integration
- Real-world scenario simulation
- Performance validation

## Test Environment Setup

### Database Setup
```bash
# Create test database
createdb fiddupay_test

# Run migrations
DATABASE_URL=postgresql://localhost/fiddupay_test sqlx migrate run
```

### Redis Setup
```bash
# Start Redis for testing
redis-server --port 6380 --daemonize yes
```

### Environment Variables
```bash
# Test environment
export DATABASE_URL=postgresql://localhost/fiddupay_test
export REDIS_URL=redis://localhost:6380
export ENCRYPTION_KEY=test_key_32_bytes_long_for_testing
export WEBHOOK_SIGNING_KEY=test_webhook_key_32_bytes_long
```

## Test Data

### Test Addresses
```bash
# Ethereum (USDT_ETH)
0x742d35Cc6634C0532925a3b8D4C9db96590c6C87

# Solana (SOL/USDT_SOL)
9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM

# BSC (USDT_BSC)
0x742d35Cc6634C0532925a3b8D4C9db96590c6C87
```

### Test Merchants
```json
{
  "business_name": "Test Business",
  "email": "test@example.com",
  "password": "password123"
}
```

## Continuous Integration

### GitHub Actions
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
```

## Performance Testing

### Load Testing
```bash
# Install wrk
sudo apt install wrk

# Test payment creation
wrk -t12 -c400 -d30s \
  --header "Authorization: Bearer test_api_key" \
  --header "Content-Type: application/json" \
  --body '{"amount_usd":"100.00","crypto_type":"USDT_ETH","description":"Load test"}' \
  -s post.lua \
  http://localhost:8080/api/v1/payments
```

### Memory Testing
```bash
# Run with memory profiling
cargo test --release -- --nocapture

# Check for memory leaks
valgrind --tool=memcheck --leak-check=full cargo test
```

## Test Coverage

### Generate Coverage Report
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

### Coverage Targets
- **Unit Tests**: >90% coverage
- **Integration Tests**: >80% coverage
- **API Tests**: 100% endpoint coverage
- **Critical Paths**: 100% coverage

## Troubleshooting Tests

### Common Issues

#### Database Connection
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Reset test database
dropdb fiddupay_test && createdb fiddupay_test
DATABASE_URL=postgresql://localhost/fiddupay_test sqlx migrate run
```

#### Redis Connection
```bash
# Check Redis status
redis-cli ping

# Start Redis if not running
redis-server --daemonize yes
```

#### Test Failures
```bash
# Run with verbose output
cargo test -- --nocapture

# Run single test with debug
RUST_LOG=debug cargo test test_name -- --nocapture

# Clean and rebuild
cargo clean && cargo build --tests
```

## Best Practices

### Test Organization
- Group related tests in modules
- Use descriptive test names
- Include setup and teardown
- Mock external dependencies

### Test Data
- Use realistic test data
- Clean up after tests
- Avoid hardcoded values
- Use test fixtures

### Assertions
- Test both success and failure cases
- Validate error messages
- Check edge cases
- Verify side effects

### Performance
- Keep tests fast
- Run tests in parallel
- Use test databases
- Mock slow operations

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Parallel execution
cargo test -- --test-threads=4
```

## Test Coverage

### 1. Core Services (Unit Tests)

**MerchantService:**
- [ ] Register merchant
- [ ] Generate API key
- [ ] Rotate API key
- [ ] Authenticate
- [ ] Set wallet address
- [ ] Validate wallet address

**PaymentService:**
- [ ] Create payment
- [ ] Calculate fees
- [ ] Verify payment
- [ ] List payments
- [ ] Handle expiration

**DepositAddressService:**
- [ ] Generate Solana address
- [ ] Generate EVM address
- [ ] Encrypt private key
- [ ] Decrypt private key
- [ ] Mark as used
- [ ] Expire old addresses

**BalanceService:**
- [ ] Credit available balance
- [ ] Debit available balance
- [ ] Reserve balance
- [ ] Release reserve
- [ ] Get balance
- [ ] Balance history

**WithdrawalService:**
- [ ] Create withdrawal
- [ ] Validate minimum amount
- [ ] Auto-approve < $1000
- [ ] Manual approve >= $1000
- [ ] Cancel withdrawal
- [ ] Complete withdrawal

**InvoiceService:**
- [ ] Create invoice
- [ ] Calculate totals
- [ ] Mark as paid
- [ ] List invoices

**EmailService:**
- [ ] Send payment confirmation
- [ ] Send withdrawal notification
- [ ] Send invoice
- [ ] Send 2FA alert
- [ ] Handle SMTP errors

**TwoFactorService:**
- [ ] Setup 2FA
- [ ] Generate secret
- [ ] Generate recovery codes
- [ ] Verify TOTP code
- [ ] Enable 2FA
- [ ] Disable 2FA

**MultiUserService:**
- [ ] Create user
- [ ] Authenticate user
- [ ] Update role
- [ ] Deactivate user
- [ ] Check permissions

### 2. Utilities (Unit Tests)

**Encryption:**
- [ ] Encrypt/decrypt roundtrip
- [ ] Random nonces
- [ ] Invalid key handling
- [ ] Invalid data handling

**Key Generation:**
- [ ] Generate Solana keypair
- [ ] Generate EVM keypair
- [ ] Validate addresses
- [ ] Unique keys

**Retry Logic:**
- [ ] Exponential backoff
- [ ] Max retries
- [ ] Success after retry

**Circuit Breaker:**
- [ ] Open after threshold
- [ ] Half-open transition
- [ ] Close after success

### 3. API Endpoints (Integration Tests)

**Merchant Endpoints:**
- [ ] POST /api/v1/merchant/register
- [ ] PUT /api/v1/merchant/wallets
- [ ] GET /api/v1/merchant/wallets
- [ ] PUT /api/v1/merchant/webhook
- [ ] POST /api/v1/merchant/api-keys/rotate

**Payment Endpoints:**
- [ ] POST /api/v1/payments
- [ ] GET /api/v1/payments/:id
- [ ] GET /api/v1/payments
- [ ] POST /api/v1/payments/:id/verify

**Balance Endpoints:**
- [ ] GET /api/v1/merchant/balance
- [ ] GET /api/v1/merchant/balance/history

**Withdrawal Endpoints:**
- [ ] POST /api/v1/withdrawals
- [ ] GET /api/v1/withdrawals
- [ ] GET /api/v1/withdrawals/:id
- [ ] POST /api/v1/withdrawals/:id/cancel

**Invoice Endpoints:**
- [ ] POST /api/v1/invoices
- [ ] GET /api/v1/invoices
- [ ] GET /api/v1/invoices/:id

**Refund Endpoints:**
- [ ] POST /api/v1/refunds
- [ ] GET /api/v1/refunds/:id

**Analytics Endpoints:**
- [ ] GET /api/v1/analytics
- [ ] GET /api/v1/analytics/export

**Security Endpoints:**
- [ ] PUT /api/v1/merchant/ip-whitelist
- [ ] GET /api/v1/merchant/ip-whitelist
- [ ] GET /api/v1/audit-logs

**2FA Endpoints:**
- [ ] POST /api/v1/merchant/2fa/setup
- [ ] POST /api/v1/merchant/2fa/enable
- [ ] POST /api/v1/merchant/2fa/disable

**Multi-User Endpoints:**
- [ ] POST /api/v1/merchant/users
- [ ] GET /api/v1/merchant/users
- [ ] PUT /api/v1/merchant/users/:id
- [ ] DELETE /api/v1/merchant/users/:id

**Sandbox Endpoints:**
- [ ] POST /api/v1/sandbox/enable
- [ ] POST /api/v1/sandbox/payments/:id/simulate

### 4. Middleware (Integration Tests)

**Authentication:**
- [ ] Valid API key
- [ ] Invalid API key
- [ ] Missing API key
- [ ] Expired API key

**Rate Limiting:**
- [ ] Within limit
- [ ] Exceed limit
- [ ] Reset after window

**IP Whitelist:**
- [ ] Allowed IP
- [ ] Blocked IP
- [ ] Empty whitelist (allow all)
- [ ] CIDR range

**Logging:**
- [ ] Request logging
- [ ] Response logging
- [ ] Error logging

### 5. End-to-End Workflows

**Payment Flow:**
1. [ ] Merchant registers
2. [ ] Merchant sets wallet
3. [ ] Merchant creates payment
4. [ ] Temp address generated
5. [ ] Customer pays
6. [ ] Payment verified
7. [ ] Funds forwarded
8. [ ] Balance credited
9. [ ] Webhook sent
10. [ ] Email sent

**Withdrawal Flow:**
1. [ ] Merchant has balance
2. [ ] Merchant requests withdrawal
3. [ ] Balance reserved
4. [ ] Auto-approve if < $1000
5. [ ] Process withdrawal
6. [ ] Balance debited
7. [ ] Email sent

**Invoice Flow:**
1. [ ] Merchant creates invoice
2. [ ] Customer receives email
3. [ ] Customer pays
4. [ ] Invoice marked paid
5. [ ] Balance credited

**2FA Flow:**
1. [ ] Merchant sets up 2FA
2. [ ] Scans QR code
3. [ ] Verifies code
4. [ ] Enables 2FA
5. [ ] Required for sensitive ops

**Refund Flow:**
1. [ ] Payment confirmed
2. [ ] Merchant creates refund
3. [ ] Balance debited
4. [ ] Refund processed
5. [ ] Webhook sent

### 6. Security Tests

**Encryption:**
- [ ] Cannot decrypt without key
- [ ] Different ciphertext each time
- [ ] Tampered data fails

**API Keys:**
- [ ] Hashed with Argon2
- [ ] Cannot reverse hash
- [ ] Rotation invalidates old key

**2FA:**
- [ ] Invalid code rejected
- [ ] Expired code rejected
- [ ] Time window tolerance

**IP Whitelist:**
- [ ] Bypass attempts blocked
- [ ] CIDR validation

**Webhooks:**
- [ ] Signature verification
- [ ] Replay attack prevention
- [ ] Timestamp validation

### 7. Performance Tests

**Load Testing:**
- [ ] 100 concurrent requests
- [ ] 1000 payments/minute
- [ ] Database connection pooling
- [ ] Redis caching

**Stress Testing:**
- [ ] Rate limit enforcement
- [ ] Circuit breaker activation
- [ ] Graceful degradation

### 8. Error Handling

**Database Errors:**
- [ ] Connection failure
- [ ] Constraint violation
- [ ] Transaction rollback

**External API Errors:**
- [ ] RPC timeout
- [ ] Price API failure
- [ ] SMTP failure

**Validation Errors:**
- [ ] Invalid input
- [ ] Missing required fields
- [ ] Type mismatch

## Test Data

### Test Wallets

**Solana Devnet:**
```
Address: 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

**BSC Testnet:**
```
Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

### Test API Keys

```
Sandbox: test_abc123xyz789
Production: live_placeholderxyz789
```

### Test Merchant

```json
{
  "email": "test@example.com",
  "business_name": "Test Store"
}
```

## Continuous Integration

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
      redis:
        image: redis:7
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all-features
```

## Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

## Test Checklist

Before deployment:
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All E2E tests pass
- [ ] Security tests pass
- [ ] Performance tests pass
- [ ] Coverage > 80%
- [ ] No critical warnings
- [ ] Documentation updated
