# PayFlow - Quick Test Guide

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install sqlx-cli (optional but recommended)
cargo install sqlx-cli --no-default-features --features postgres
```

## Setup Test Database

```bash
# Create test database
createdb payflow_test

# Run migrations
export DATABASE_URL="postgresql://vibes:Soledayo@2001@localhost:5432/payflow_test"
sqlx migrate run

# Or manually:
psql payflow_test < migrations/20240101000001_create_merchant_tables.sql
psql payflow_test < migrations/20240101000002_create_payment_tables.sql
psql payflow_test < migrations/20240101000003_create_webhook_refund_tables.sql
psql payflow_test < migrations/20240101000004_balance_management.sql
psql payflow_test < migrations/20240101000005_withdrawals.sql
psql payflow_test < migrations/20240101000006_roles_invoices_2fa.sql
```

## Run Tests

```bash
# Set environment variables
export DATABASE_URL="postgresql://vibes:Soledayo@2001@localhost:5432/payflow_test"
export ENCRYPTION_KEY="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
export RUST_LOG=error

# Run all tests
cargo test

# Run specific test suite
cargo test --lib                    # Unit tests
cargo test --test services_test     # Service tests
cargo test --test payment_test      # Payment tests
cargo test --test withdrawal_test   # Withdrawal tests
cargo test --test workflows_test    # E2E tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_encryption_roundtrip
```

## Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## Quick Test (No Setup)

```bash
# Just run unit tests (no database needed)
cargo test --lib
```

## Test Results

Expected output:
```
running 15 tests
test test_encryption_roundtrip ... ok
test test_solana_keypair_generation ... ok
test test_evm_keypair_generation ... ok
...

test result: ok. 15 passed; 0 failed; 0 ignored
```

## Troubleshooting

### Database Connection Error
```bash
# Check PostgreSQL is running
pg_isready

# Check database exists
psql -l | grep payflow_test

# Recreate database
dropdb payflow_test
createdb payflow_test
```

### Migration Errors
```bash
# Reset database
dropdb payflow_test && createdb payflow_test

# Run migrations in order
for f in migrations/*.sql; do psql payflow_test < $f; done
```

### Cargo Not Found
```bash
# Add to PATH
source $HOME/.cargo/env
```
