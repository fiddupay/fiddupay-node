# Testing Guide for Real-Time Features

This guide provides instructions for manually verifying the new Real-Time Webhooks and Partial Payment features.

## Prerequisites

- Local backend running (`npm run dev:backend`)
- Local PostgreSQL database
- Postman or curl

## Scenario 1: Partial Payment Simulation

This scenario tests the system's ability to detect an underpayment and trigger the `payment.partially_paid` webhook.

### Steps

1. **Create a Payment**
   Use the `Create Payment` API to generate a new payment for 100 USDT (ERC20).
   ```bash
   curl -X POST http://localhost:8080/v1/payments \
     -H "Authorization: Bearer sk_sandbox_..." \
     -H "Content-Type: application/json" \
     -d '{
       "amount": "100.00",
       "crypto_type": "USDT_ETH",
       "webhook_url": "https://webhook.site/..."
     }'
   ```
   **Save the `payment_id` and `to_address` from the response.**

2. **Simulate Incoming Transaction (Underpayment)**
   Directly insert a mock transaction into the database to skip blockchain waiting times.
   ```sql
   -- Run in your SQL client
   UPDATE address_only_payments 
   SET status = 'partial_payment_received' 
   WHERE payment_id = 'YOUR_PAYMENT_ID';
   ```
   *Note: In production, the system detects this via `PaymentMonitorService` when `balance > 0` and `balance < requested_amount`.*

3. **Verify Webhook**
   Check your webhook URL (e.g., webhook.site). You should receive a payload:
   ```json
   {
     "event_type": "payment.partially_paid",
     "payment_id": "pay_...",
     "status": "partial_payment_received",
     "amount": "50.00",
     ...
   }
   ```

## Scenario 2: Real-Time Detection (0-Conf)

This scenario tests the `payment.detected` event, which should fire immediately when a transaction hits the mempool.

### Steps

1. **Start Backend with WebSocket Config**
   Ensure `SOLANA_RPC_URL` starts with `https://` (the backend converts this to `wss://` automatically).

2. **Trigger Mock Detection**
   Since creating an actual 0-conf transaction on mainnet is costly, you can verify the connection logs:
   ```bash
   # Check logs for
   "Connecting to Solana WebSocket..."
   ```

3. **Verify Event**
   When the system sees a transaction hash for a monitored address, it queues a webhook:
   ```json
   {
     "event_type": "payment.detected",
     "payment_id": "pay_...",
     "status": "pending",
     "transaction_hash": "...",
     ...
   }
   ```

## Troubleshooting

- **No Webhook?** Check `webhook_deliveries` table for formatting errors.
- **Connection Failed?** Verify your RPC provider supports WebSocket connections.
