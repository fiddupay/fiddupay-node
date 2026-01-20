# Testing Guide - Crypto Payment Gateway

## Overview

This guide covers how to test all implemented functionality from Tasks 1-17.

---

## Prerequisites

Before running tests, ensure you have:

1. **Rust installed** (1.75+)
   ```bash
   rustc --version
   ```

2. **PostgreSQL running** (for integration tests)
   ```bash
   psql --version
   sudo systemctl status postgresql
   ```

3. **Test database created**
   ```bash
   createdb crypto_gateway_test
   ```

4. **Environment configured**
   ```bash
   # Create test .env
   cp .env .env.test
   # Update DATABASE_URL to use test database
   sed -i 's/crypto_gateway_staging/crypto_gateway_test/g' .env.test
   ```

---

## Running Tests

### Quick Test (All Tests)

```bash
# Run the test script
./run_tests.sh

# Or manually:
cargo test
```

### Unit Tests Only

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test --lib merchant_service
cargo test --lib payment_service
cargo test --lib webhook_service
```

### Integration Tests

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test payment_listing_tests
cargo test --test analytics_service_tests
```

### With Output

```bash
# Show println! and tracing output
cargo test -- --nocapture

# Show only failed tests
cargo test -- --nocapture --test-threads=1
```

---

## Test Coverage by Task

### ✅ Task 1: Project Setup
**What to test:**
- Project compiles without errors
- All dependencies resolve

**How to test:**
```bash
cargo build
cargo check
```

**Expected:** No compilation errors

---

### ✅ Task 2: Database Schema
**What to test:**
- Migrations run successfully
- All tables created with correct schema

**How to test:**
```bash
# Run migrations
sqlx migrate run

# Check tables exist
psql -d crypto_gateway_test -c "\dt"

# Verify schema
psql -d crypto_gateway_test -c "\d merchants"
psql -d crypto_gateway_test -c "\d payment_transactions"
psql -d crypto_gateway_test -c "\d webhook_deliveries"
```

**Expected:** All tables present with correct columns

---

### ✅ Task 3: Core Data Models
**What to test:**
- Models serialize/deserialize correctly
- FromRow derives work

**How to test:**
```bash
cargo test --lib models
```

**Expected:** All model tests pass

---

### ✅ Task 4: Merchant Service
**What to test:**
- Merchant registration
- API key generation and rotation
- Wallet address management

**How to test:**
```bash
cargo test --lib merchant_service

# Specific tests:
cargo test test_register_merchant
cargo test test_generate_api_key
cargo test test_set_wallet_address
```

**Expected:** All merchant service tests pass

---

### ✅ Task 6: Payment Service
**What to test:**
- Payment creation with fees
- Payment verification
- Payment listing with filters

**How to test:**
```bash
cargo test --lib payment_service
cargo test --test payment_listing_tests
```

**Expected:** Payment creation, verification, and listing work

---

### ✅ Task 7: Webhook Service
**What to test:**
- Webhook configuration
- Webhook delivery
- Signature generation

**How to test:**
```bash
cargo test --lib webhook_service

# Specific tests:
cargo test test_set_webhook_url
cargo test test_generate_signature
```

**Expected:** Webhook service tests pass

---

### ✅ Task 9: Fee Calculation
**What to test:**
- Fee calculation accuracy
- Fee percentage bounds

**How to test:**
```bash
cargo test --lib fee_calculator
```

**Expected:** Fee calculations are correct

---

### ✅ Task 10: Refund Service
**What to test:**
- Refund creation
- Refund completion
- Balance calculation

**How to test:**
```bash
cargo test --lib refund_service
```

**Expected:** Refund operations work correctly

---

### ✅ Task 11: Analytics Service
**What to test:**
- Analytics calculation
- CSV export

**How to test:**
```bash
cargo test --lib analytics_service
cargo test --test analytics_service_tests
```

**Expected:** Analytics and export work

---

### ✅ Task 12: Sandbox Service
**What to test:**
- Sandbox credentials creation
- Payment simulation
- Sandbox isolation

**How to test:**
```bash
cargo test --lib sandbox_service

# Specific tests:
cargo test test_create_sandbox_credentials
cargo test test_is_sandbox_key
cargo test test_simulate_confirmation
```

**Expected:** Sandbox functionality works

---

### ✅ Task 14: Partial Payments
**What to test:**
- Partial payment recording
- Balance tracking
- Auto-completion

**How to test:**
```bash
cargo test --lib verifier

# Specific test:
cargo test test_record_partial_payment
```

**Expected:** Partial payments tracked correctly

---

### ✅ Task 16: API Layer
**What to test:**
- All endpoints respond
- Request/response formats correct

**How to test:**
```bash
# Start the server
cargo run &
SERVER_PID=$!

# Test health endpoint
curl http://localhost:8080/health

# Test merchant registration
curl -X POST http://localhost:8080/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","business_name":"Test"}'

# Save API key from response
API_KEY="<your_api_key>"

# Test payment creation
curl -X POST http://localhost:8080/api/v1/payments \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"amount_usd":100,"crypto_type":"SOL"}'

# Stop server
kill $SERVER_PID
```

**Expected:** All endpoints return valid responses

---

### ✅ Task 17: Authentication & Middleware
**What to test:**
- API key authentication
- Rate limiting
- IP whitelisting

**How to test:**
```bash
# Test without auth (should fail)
curl -X POST http://localhost:8080/api/v1/payments \
  -H "Content-Type: application/json" \
  -d '{"amount_usd":100,"crypto_type":"SOL"}'
# Expected: 401 Unauthorized

# Test with invalid API key (should fail)
curl -X POST http://localhost:8080/api/v1/payments \
  -H "Authorization: Bearer invalid_key" \
  -H "Content-Type: application/json" \
  -d '{"amount_usd":100,"crypto_type":"SOL"}'
# Expected: 401 Unauthorized

# Test rate limiting (make 101 requests quickly)
for i in {1..101}; do
  curl -X GET http://localhost:8080/health &
done
wait
# Expected: Some requests return 429 Too Many Requests
```

**Expected:** Authentication and rate limiting work

---

## Manual Testing Checklist

### End-to-End Payment Flow

1. **Register Merchant**
   ```bash
   curl -X POST http://localhost:8080/api/v1/merchants/register \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","business_name":"Test Co"}'
   ```
   ✅ Returns merchant_id and api_key

2. **Create Payment**
   ```bash
   curl -X POST http://localhost:8080/api/v1/payments \
     -H "Authorization: Bearer $API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"amount_usd":100,"crypto_type":"SOL","description":"Test"}'
   ```
   ✅ Returns payment details with payment_link

3. **Visit Payment Page**
   ```bash
   # Open in browser
   open http://localhost:8080/pay/lnk_...
   ```
   ✅ Shows payment page with QR code and countdown

4. **Check Payment Status**
   ```bash
   curl http://localhost:8080/pay/lnk_.../status
   ```
   ✅ Returns current payment status

5. **List Payments**
   ```bash
   curl http://localhost:8080/api/v1/payments \
     -H "Authorization: Bearer $API_KEY"
   ```
   ✅ Returns paginated payment list

---

## Performance Testing

### Load Test with Apache Bench

```bash
# Install Apache Bench
sudo apt-get install apache2-utils

# Test health endpoint
ab -n 1000 -c 10 http://localhost:8080/health

# Test with authentication
ab -n 100 -c 5 -H "Authorization: Bearer $API_KEY" \
  http://localhost:8080/api/v1/payments
```

**Expected:**
- Health endpoint: >1000 req/sec
- Authenticated endpoints: >100 req/sec

---

## Troubleshooting Tests

### Tests Fail to Connect to Database

**Error:** `Connection refused` or `database does not exist`

**Solution:**
```bash
# Ensure PostgreSQL is running
sudo systemctl start postgresql

# Create test database
createdb crypto_gateway_test

# Update DATABASE_URL in .env
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/crypto_gateway_test"
```

### Tests Timeout

**Error:** `test timed out after 60 seconds`

**Solution:**
```bash
# Increase timeout
cargo test -- --test-threads=1 --nocapture

# Or run specific slow test
cargo test slow_test_name -- --ignored
```

### Compilation Errors

**Error:** `cannot find function/type in this scope`

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

---

## Test Results Summary

After running all tests, you should see:

```
test result: ok. XX passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Expected Test Count:**
- Unit tests: ~30-50 tests
- Integration tests: ~10-20 tests
- **Total: ~40-70 tests**

---

## Continuous Integration

For CI/CD pipelines:

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
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all-features
```

---

## Next Steps

After all tests pass:

1. ✅ Review test coverage
2. ✅ Fix any failing tests
3. ✅ Add more integration tests if needed
4. ✅ Deploy to staging environment
5. ✅ Run smoke tests in staging
6. ✅ Monitor logs for errors
7. ✅ Proceed to production deployment

---

## Support

If tests fail:
1. Check the error message carefully
2. Review the test code to understand what's being tested
3. Check logs: `RUST_LOG=debug cargo test`
4. Verify database and services are running
5. Consult SETUP_INSTRUCTIONS.md
6. Check FINAL_SUMMARY.md for known limitations
