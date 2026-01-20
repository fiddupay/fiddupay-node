# Background Tasks

This document describes the background tasks implemented in the Crypto Payment Gateway service.

## Overview

The gateway runs several background tasks to handle asynchronous operations:

1. **Payment Expiration Checker** - Monitors and expires payments that have passed their expiration time
2. **Webhook Retry Processor** (future) - Retries failed webhook deliveries with exponential backoff

## Payment Expiration Checker

### Purpose

Automatically marks payments as expired when their expiration time has elapsed, ensuring that:
- Merchants don't receive payments for expired payment requests
- Payment status accurately reflects the current state
- Webhook notifications are sent when payments expire

### How It Works

1. **Runs every 30 seconds** - The task wakes up at regular intervals to check for expired payments
2. **Queries expired payments** - Finds all payments where:
   - `expires_at < current_time`
   - `status IN ('PENDING', 'CONFIRMING')`
3. **Updates status** - Changes payment status from `PENDING`/`CONFIRMING` to `FAILED`
4. **Sends webhooks** - Queues webhook notifications with event type `payment.expired`

### Database Query

```sql
SELECT id, merchant_id, payment_id, amount, crypto_type
FROM payment_transactions
WHERE expires_at < NOW()
  AND status IN ('PENDING', 'CONFIRMING')
```

### Webhook Notification

When a payment expires, a webhook is queued with the following payload:

```json
{
  "event_type": "payment.expired",
  "payment_id": "pay_abc123",
  "merchant_id": 42,
  "status": "FAILED",
  "amount": "100.00",
  "crypto_type": "USDT_BEP20",
  "transaction_hash": null,
  "timestamp": 1234567890
}
```

### Requirements Satisfied

- **Requirement 2.4**: Mark payments as expired when expiration time elapses
- **Requirement 2.7**: Update status to expired when time elapses
- **Requirement 4.3**: Trigger webhook notifications for expired payments

## Starting Background Tasks

### In Application Code

```rust
use crypto_payment_gateway::background_tasks::BackgroundTasks;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database pool
    let db_pool = PgPool::connect(&database_url).await?;
    
    // Create and start background tasks
    let background_tasks = Arc::new(BackgroundTasks::new(db_pool.clone()));
    background_tasks.start();
    
    // Continue with server startup...
    Ok(())
}
```

### Configuration

The expiration checker interval is currently hardcoded to 30 seconds. This can be made configurable via environment variables in the future:

```bash
# Future configuration option
EXPIRATION_CHECK_INTERVAL_SECONDS=30
```

## Error Handling

### Graceful Degradation

The background tasks are designed to handle errors gracefully:

1. **Database errors** - Logged and retried on next interval
2. **Webhook queueing errors** - Logged but don't prevent payment status update
3. **Race conditions** - Handled by checking `rows_affected()` after update

### Logging

All operations are logged with appropriate levels:

- **INFO**: Normal operations (payments expired, webhooks queued)
- **WARN**: Race conditions (payment already updated)
- **ERROR**: Failures (database errors, webhook queueing failures)

Example logs:

```
INFO Found 3 expired payments to process
INFO Marked payment pay_abc123 (id: 42) as expired for merchant 10
INFO Queued webhook for merchant 10 - event: payment.expired
ERROR Failed to queue webhook for expired payment pay_xyz789: Database error
```

## Testing

### Unit Tests

The module includes unit tests for:
- Background task creation
- Webhook payload structure
- Expiration time logic
- Payment status transitions
- Event type validation

Run tests with:

```bash
cargo test background_tasks
```

### Integration Testing

To test the expiration checker in a development environment:

1. Create a payment with a short expiration time (e.g., 1 minute)
2. Wait for the expiration time to elapse
3. Observe logs for expiration processing
4. Verify payment status is updated to `FAILED`
5. Check webhook_deliveries table for queued webhook

## Performance Considerations

### Scalability

- **Query efficiency**: Uses indexed columns (`expires_at`, `status`)
- **Batch processing**: Processes all expired payments in a single query
- **Non-blocking**: Runs in separate tokio task, doesn't block main server
- **Interval-based**: 30-second interval prevents excessive database load

### Database Load

With 30-second intervals:
- **Queries per hour**: 120
- **Queries per day**: 2,880

This is minimal load even for large-scale deployments.

### Horizontal Scaling

For multi-instance deployments:
- **Race condition handling**: Uses `WHERE status IN ('PENDING', 'CONFIRMING')` in UPDATE to prevent double-processing
- **Idempotent**: Safe to run on multiple instances simultaneously
- **No coordination needed**: Each instance independently processes expired payments

## Future Enhancements

1. **Configurable interval** - Make check interval configurable via environment variable
2. **Metrics** - Add Prometheus metrics for expired payment counts
3. **Webhook retry task** - Implement background task for webhook delivery retries
4. **Payment monitoring** - Add background task for monitoring pending payments on blockchain
5. **Graceful shutdown** - Handle shutdown signals to complete in-flight operations

## Related Documentation

- [Webhook Service](./webhooks.md) - Webhook delivery and retry logic
- [Payment Flow](./payment_flow.md) - Complete payment lifecycle
- [Database Schema](./schema.md) - Database tables and indexes
