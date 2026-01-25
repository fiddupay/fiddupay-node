# Database Cleanup Scripts

## Available Scripts

### 1. Cleanup Database (Keep Structure)
```bash
./scripts/cleanup_database.sh
```
- **What it does**: Truncates all tables but keeps structure
- **Use when**: You want to clear data but keep the database schema
- **Safety**: Requires typing 'DELETE' to confirm

### 2. Reset Database (Complete Reset)
```bash
./scripts/reset_database.sh
```
- **What it does**: Drops and recreates database completely
- **Use when**: You want a completely fresh start
- **Safety**: Requires typing 'RESET' to confirm

## Environment Variables

Set these before running scripts:

```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=fiddupay
export DB_USER=postgres
```

## What Gets Cleaned

### Tables Cleared:
- `payment_transactions` - All payment records
- `partial_payments` - Partial payment data
- `webhook_events` - Webhook delivery logs
- `refunds` - Refund records
- `merchants` - Merchant accounts
- `merchant_balances` - Balance records
- `merchant_wallets` - Wallet configurations
- `merchant_withdrawals` - Withdrawal history
- `merchant_invoices` - Invoice data
- `merchant_api_keys` - API key records
- `merchant_users` - Multi-user data
- `analytics_events` - Analytics data
- `audit_logs` - Audit trail
- `security_events` - Security logs
- `user_sessions` - Active sessions
- `rate_limit_entries` - Rate limiting data

### What's Preserved:
- Database schema/structure
- Migrations table
- Indexes and constraints

## Usage Examples

### Clean Development Data
```bash
# Set environment
export DB_NAME=fiddupay_dev

# Clean data
./scripts/cleanup_database.sh
```

### Reset Test Database
```bash
# Set environment  
export DB_NAME=fiddupay_test

# Complete reset
./scripts/reset_database.sh
```

## Safety Features

- ✅ Confirmation prompts
- ✅ Environment variable support
- ✅ Error handling with `set -e`
- ✅ Verification queries
- ✅ Graceful fallbacks

**⚠️ Always backup production data before running cleanup scripts!**
