# FidduPay API Reference v2.4.0

## Base URL
- **Sandbox**: `http://localhost:8080`
- **Production**: `https://api.fiddupay.com`

## Authentication
All API requests require authentication using Bearer tokens:
```
Authorization: Bearer sk_your_api_key_here
```

### API Key Formats
- **Sandbox**: `sk_` prefix (e.g., `sk_1234567890abcdef...`)
- **Production**: `live_` prefix (e.g., `live_1234567890abcdef...`)

## Daily Volume Limits
- **Non-KYC Merchants**: $1,000 USD daily volume limit (combined deposits + withdrawals)
- **KYC Verified Merchants**: No daily volume limits
- **Reset**: Daily limits reset at midnight UTC
- **Tracking**: Real-time volume tracking across all transaction types
- **Error**: `DAILY_VOLUME_EXCEEDED` when limit is reached

### Check Remaining Volume
```http
GET /api/v1/merchants/profile
Authorization: Bearer {api_key}
```

Response includes `daily_volume_remaining` for non-KYC merchants:
```json
{
  "id": 123,
  "business_name": "My Business",
  "email": "merchant@example.com",
  "kyc_verified": false,
  "daily_volume_remaining": "750.00"
}
```

## Public Endpoints (No Auth Required)

### Health Check
```http
GET /health
```
Returns system health status.

### System Status
```http
GET /api/v1/status
```
Returns detailed system status including service health and performance metrics.

### Payment Page
```http
GET /pay/{link_id}
```
Displays payment page for a specific payment link.

### Payment Status
```http
GET /pay/{link_id}/status
```
Returns payment status for a specific payment link.

### Register Merchant
```http
POST /api/v1/merchants/register
Content-Type: application/json

{
  "email": "merchant@example.com",
  "business_name": "My Business",
  "password": "secure_password"
}
```

### Login Merchant
```http
POST /api/v1/merchants/login
Content-Type: application/json

{
  "email": "merchant@example.com",
  "password": "secure_password"
}
```

### Get Supported Currencies
```http
GET /api/v1/currencies/supported
```

## Merchant Endpoints (Auth Required)

### Get Merchant Profile
```http
GET /api/v1/merchants/profile
Authorization: Bearer {api_key}
```

### Switch Environment
```http
POST /api/v1/merchants/environment/switch
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "environment": "sandbox" // or "production"
}
```

### Generate API Key
```http
POST /api/v1/merchants/api-keys/generate
Authorization: Bearer {api_key}
```

### Rotate API Key
```http
POST /api/v1/merchants/api-keys/rotate
Authorization: Bearer {api_key}
```

### Set Wallet
```http
PUT /api/v1/merchants/wallets
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "wallet_address": "your_wallet_address"
}
```

### Set Webhook
```http
PUT /api/v1/merchants/webhook
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "webhook_url": "https://your-site.com/webhook"
}
```

## Payment Endpoints

### Create Payment
```http
POST /api/v1/merchants/payments
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "amount_usd": "100.00",
  "crypto_type": "SOL",
  "description": "Payment for order #123",
  "webhook_url": "https://your-site.com/webhook",
  "metadata": { "order_id": 123 },
  "expires_in": 3600,
  "partial_payments_enabled": false
}
```

**Note**: You must provide either `amount` (in crypto) or `amount_usd` (in USD), but not both.

### List Payments
```http
GET /api/v1/merchants/payments
Authorization: Bearer {api_key}
```

### Get Payment
```http
GET /api/v1/merchants/payments/{payment_id}
Authorization: Bearer {api_key}
```

### Verify Payment
```http
POST /api/v1/merchants/payments/{payment_id}/verify
Authorization: Bearer {api_key}
```

## Refund Endpoints

### Create Refund
```http
POST /api/v1/merchants/refunds
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "payment_id": "payment_123",
  "amount": "50.00",
  "reason": "Customer request"
}
```

### Get Refund
```http
GET /api/v1/merchants/refunds/{refund_id}
Authorization: Bearer {api_key}
```

### Complete Refund
```http
POST /api/v1/merchants/refunds/{refund_id}/complete
Authorization: Bearer {api_key}
```

## Analytics Endpoints

### Get Analytics
```http
GET /api/v1/merchants/analytics
Authorization: Bearer {api_key}
```

### Export Analytics
```http
GET /api/v1/merchants/analytics/export
Authorization: Bearer {api_key}
```

## Invoice Management

### Create Invoice
```http
POST /api/v1/merchants/invoices
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "amount_usd": "150.00",
  "customer_email": "customer@example.com",
  "description": "Consulting services",
  "expires_in": 86400
}
```

### List Invoices
```http
GET /api/v1/merchants/invoices
Authorization: Bearer {api_key}
```

### Get Invoice
```http
GET /api/v1/merchants/invoices/{invoice_id}
Authorization: Bearer {api_key}
```

## Sandbox Endpoints

### Enable Sandbox
```http
POST /api/v1/merchants/sandbox/enable
Authorization: Bearer {api_key}
```

### Simulate Payment
```http
POST /api/v1/merchants/sandbox/payments/{payment_id}/simulate
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "status": "completed", // or "failed"
  "transaction_hash": "0xabc...",
  "from_address": "0xsender..."
}
```

## Security Endpoints

### Set IP Whitelist
```http
PUT /api/v1/merchants/ip-whitelist
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "ip_addresses": ["192.168.1.1", "10.0.0.1"]
}
```

### Get IP Whitelist
```http
GET /api/v1/merchants/ip-whitelist
Authorization: Bearer {api_key}
```

### Get Audit Logs
```http
GET /api/v1/merchants/audit-logs
Authorization: Bearer {api_key}
```

## Balance Endpoints

### Get Balance
```http
GET /api/v1/merchants/balance
Authorization: Bearer {api_key}
```

Response:
```json
{
  "total_usd": "1250.50",
  "available_usd": "1100.00",
  "reserved_usd": "150.50",
  "balances": [
    {
      "crypto_type": "SOL",
      "total_balance": "10.0234",
      "available_balance": "9.5234",
      "reserved_balance": "0.5",
      "balance_usd": "1250.50",
      "available_usd": "1100.00",
      "reserved_usd": "150.50",
      "last_updated": "2024-01-01T12:00:00Z"
    }
  ]
}
```

### Get Balance History
```http
GET /api/v1/merchants/balance/history
Authorization: Bearer {api_key}
```

## Withdrawal Endpoints

### Create Withdrawal
```http
POST /api/v1/merchants/withdrawals
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "amount": "100.00",
  "crypto_type": "SOL",
  "destination_address": "recipient_wallet_address"
}
```

### List Withdrawals
```http
GET /api/v1/merchants/withdrawals
Authorization: Bearer {api_key}
```

### Get Withdrawal
```http
GET /api/v1/merchants/withdrawals/{withdrawal_id}
Authorization: Bearer {api_key}
```

### Cancel Withdrawal
```http
POST /api/v1/merchants/withdrawals/{withdrawal_id}/cancel
Authorization: Bearer {api_key}
```

## Wallet Management Endpoints

### Get Wallet Configs
```http
GET /api/v1/merchants/wallets
Authorization: Bearer {api_key}
```

### Configure Address-Only Wallet
```http
POST /api/v1/merchants/wallets/configure-address
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "wallet_address": "your_wallet_address"
}
```

#### Node SDK Example
```javascript
// Configure wallet addresses
await fiddupay.merchants.setWallet({
  crypto_type: 'SOL',
  address: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM'
});
```

### Generate Wallet
```http
POST /api/v1/merchants/wallets/generate
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL"
}
```

### Import Wallet
```http
POST /api/v1/merchants/wallets/import
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "private_key": "your_private_key"
}
```

### Export Private Key
```http
POST /api/v1/merchants/wallets/export-key
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL"
}
```

### Check Gas Requirements
```http
GET /api/v1/merchants/wallets/gas-check
Authorization: Bearer {api_key}
```

### Get Gas Estimates
```http
GET /api/v1/merchants/wallets/gas-estimates
Authorization: Bearer {api_key}
```

### Check Withdrawal Capability
```http
GET /api/v1/merchants/wallets/withdrawal-capability/{crypto_type}
Authorization: Bearer {api_key}
```

## Security Monitoring Endpoints

### Get Security Events
```http
GET /api/v1/merchants/security/events
Authorization: Bearer {api_key}
```

### Get Security Alerts
```http
GET /api/v1/merchants/security/alerts
Authorization: Bearer {api_key}
```

### Acknowledge Security Alert
```http
POST /api/v1/merchants/security/alerts/{alert_id}/acknowledge
Authorization: Bearer {api_key}
```

### Get Balance Alerts
```http
GET /api/v1/merchants/security/balance-alerts
Authorization: Bearer {api_key}
```

### Resolve Balance Alert
```http
POST /api/v1/merchants/security/balance-alerts/{alert_id}/resolve
Authorization: Bearer {api_key}
```

### Check Gas Balances
```http
GET /api/v1/merchants/security/gas-check
Authorization: Bearer {api_key}
```

### Get Security Settings
```http
GET /api/v1/merchants/security/settings
Authorization: Bearer {api_key}
```

### Update Security Settings
```http
PUT /api/v1/merchants/security/settings
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "two_factor_enabled": true,
  "login_notifications": true
}
```

## Merchant Fee Settings

### Get Fee Settings
```http
GET /api/v1/merchants/fee-setting
Authorization: Bearer {api_key}
```

## Advanced Wallet & Withdrawal Endpoints

### Check Withdrawal Capability
```http
GET /api/v1/merchants/wallets/withdrawal-capability/{crypto_type}
Authorization: Bearer {api_key}
```

### Check Gas Requirements
```http
GET /api/v1/merchants/wallets/gas-check
Authorization: Bearer {api_key}
```

### Get Gas Estimates
```http
GET /api/v1/merchants/wallets/gas-estimates
Authorization: Bearer {api_key}
```

## Error Codes

| Code | Description |
|------|-------------|
| `INVALID_API_KEY` | API key is invalid or expired |
| `INSUFFICIENT_BALANCE` | Insufficient balance for transaction |
| `DAILY_VOLUME_EXCEEDED` | Daily volume limit exceeded for non-KYC merchant |
| `INVALID_CRYPTO_TYPE` | Unsupported cryptocurrency |
| `PAYMENT_NOT_FOUND` | Payment ID not found |
| `WITHDRAWAL_FAILED` | Withdrawal processing failed |
| `RATE_LIMIT_EXCEEDED` | Too many requests |

## Supported Cryptocurrencies

- **Solana**: SOL, USDT (SPL)
- **Ethereum**: ETH, USDT (ERC-20)
- **Binance Smart Chain**: BNB, USDT (BEP-20)
- **Polygon**: MATIC, USDT
- **Arbitrum**: ARB, USDT

## Rate Limits

- **Default**: 60 requests per minute per API key
- **Burst**: Up to 100 requests in 10 seconds
- **Headers**: `X-RateLimit-Remaining`, `X-RateLimit-Reset`

## Webhooks

FidduPay sends webhook notifications for payment events:

### Webhook Payload
```json
{
  "event": "payment.confirmed",
  "payment_id": "payment_123",
  "amount": "100.00",
  "currency": "USD",
  "crypto_type": "SOL",
  "status": "confirmed",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Webhook Events
- `payment.created`
- `payment.confirmed`
- `payment.failed`
- `refund.created`
- `refund.completed`
- `withdrawal.created`
- `withdrawal.completed`
- `withdrawal.failed`
