# PayFlow - Comprehensive Testing Guide

**TechyTro Software**

## Test Structure

```
tests/
├── unit/           # Unit tests for individual functions
├── integration/    # Integration tests for services
├── api/           # API endpoint tests
└── e2e/           # End-to-end workflow tests
```

## Running Tests

```bash
# All tests
cargo test

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
- [ ] POST /api/v1/merchants/register
- [ ] PUT /api/v1/merchants/wallets
- [ ] GET /api/v1/merchants/wallets
- [ ] PUT /api/v1/merchants/webhook
- [ ] POST /api/v1/merchants/api-keys/rotate

**Payment Endpoints:**
- [ ] POST /api/v1/payments
- [ ] GET /api/v1/payments/:id
- [ ] GET /api/v1/payments
- [ ] POST /api/v1/payments/:id/verify

**Balance Endpoints:**
- [ ] GET /api/v1/merchants/balance
- [ ] GET /api/v1/merchants/balance/history

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
- [ ] PUT /api/v1/merchants/ip-whitelist
- [ ] GET /api/v1/merchants/ip-whitelist
- [ ] GET /api/v1/audit-logs

**2FA Endpoints:**
- [ ] POST /api/v1/merchants/2fa/setup
- [ ] POST /api/v1/merchants/2fa/enable
- [ ] POST /api/v1/merchants/2fa/disable

**Multi-User Endpoints:**
- [ ] POST /api/v1/merchants/users
- [ ] GET /api/v1/merchants/users
- [ ] PUT /api/v1/merchants/users/:id
- [ ] DELETE /api/v1/merchants/users/:id

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
Production: live_abc123xyz789
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
