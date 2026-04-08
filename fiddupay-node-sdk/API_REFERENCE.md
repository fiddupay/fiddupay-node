# FidduPay API Reference v2.6.0

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

Response includes `settlement_mode`, `sandbox_mode`, and `daily_volume_remaining` for non-KYC merchants:
```json
{
  "id": 123,
  "business_name": "My Business",
  "email": "merchant@example.com",
  "kyc_verified": false,
  "sandbox_mode": true,
  "settlement_mode": "managed",
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
  "password": "secure_password",
  "two_factor_code": "123456",
  "remember_me": true
}
```
Returns `{ user: MerchantProfile, dashboard_token: string }`.

### Get Supported Currencies
```http
GET /api/v1/currencies/supported
GET /api/v1/currencies/supported?merchant_id=123
```
Returns `{ currency_groups: {...}, description: string }`.

### Get Pricing
```http
GET /api/v1/pricing
```
Returns transaction fees, supported networks, features, and volume limits.

### Cancel Payment (Public)
```http
POST /pay/{payment_id}/cancel
```
Public endpoint to cancel a payment in `PENDING` or `SELECTION_REQUIRED` status.

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

### Update Merchant Settings (Unified)
```http
PATCH /api/v1/merchants/settings
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "webhook_url": "https://your-site.com/webhook",
  "redirect_url": "https://your-site.com/success",
  "webhook_format": "json", // or "discord", "slack"
  "settlement_mode": "forwarding",
  "customer_pays_fee": true,
  "fee_percentage": 1.5,
  "ip_whitelist": ["1.2.3.4"],
  "sandbox_mode": false,
  "rotate_webhook_secret": false
}
```
**Recommended**: Use this single endpoint to update all merchant-level configurations atomically.

### Get Merchant Readiness Status
```http
GET /api/v1/merchants/status
Authorization: Bearer {api_key}
```
Returns a comprehensive health check of the merchant's integration, including wallet coverage and security alerts.

#### Node SDK Example
```javascript
const readiness = await fiddupay.merchants.getReadiness();
if (readiness.is_ready) {
  console.log('Merchant is ready to accept payments');
} else {
  console.log('Missing steps:', readiness.missing_steps);
}
```

### Set Wallet (Legacy)
> [!WARNING]
> Deprecated in favor of `POST /api/v1/merchants/wallets` (Unified Setup).

```http
PUT /api/v1/merchants/wallets
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "wallet_address": "your_wallet_address"
}
```

### Update Settlement Mode (Legacy)
> [!WARNING]
> Deprecated in favor of `PATCH /api/v1/merchants/settings`.

```http
PUT /api/v1/merchants/settlement-mode
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "mode": "forwarding" // or "managed"
}
```

### Set Webhook (Legacy)
> [!WARNING]
> Deprecated in favor of `PATCH /api/v1/merchants/settings`.

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
  "amount": "2.5",
  "crypto_type": "SOL",
  "description": "Payment for order #123"
}
```

**Field Enforcement Policy (v2.4.5+)**:
- **Stablecoins (USDT)**: You MUST use `amount_usd`. The system treats it as 1:1 and skips price fetching.
- **Native/Volatile (SOL, ETH, etc.)**: You MUST use `amount` (quantity). The system fetches real-time prices to calculate the USD value.
- **Multi-Currency (Selection Required)**: Leave both `amount` and `crypto_type` out, and provide `amount_usd`. The customer will select their preferred currency on the payment page.

**Settlement Mode Enforcement**:
- If `settlement_mode` is `forwarding`, this endpoint will return `403 Forbidden`. You MUST use Address-Only endpoints instead.

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
### Cancel Payment
```http
POST /api/v1/merchants/payments/{payment_id}/cancel
Authorization: Bearer {api_key}
```

### Finalize Currency Selection
```http
POST /api/v1/merchants/payments/{payment_id}/select
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL"
}
```
Used when `amount_usd` was provided without `crypto_type` during creation.

### Unified Transaction Feed
```http
GET /api/v1/merchants/transactions
Authorization: Bearer {api_key}
```
Returns a chronological feed combining payments, refunds, and withdrawals.

## Address-Only Endpoints
> [!IMPORTANT]
> This feature is currently in active development. Endpoints are experimental and for testing purposes only. Forwarding mode is not yet fully production-ready.

### Create Address-Only Payment
```http
POST /api/v1/merchants/address-only/create
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "merchant_address": "your_external_wallet_address",
  "requested_amount": "1.5"
}
```

### Get Address-Only Status
```http
GET /api/v1/merchants/address-only/status?payment_id={payment_id}
Authorization: Bearer {api_key}
```

### List Supported Native Currencies
```http
GET /api/v1/merchants/address-only/currencies
Authorization: Bearer {api_key}
```

### Get Address-Only Mode Stats
```http
GET /api/v1/merchants/address-only/stats
Authorization: Bearer {api_key}
```

### Get Address-Only Health
```http
GET /api/v1/merchants/address-only/health
Authorization: Bearer {api_key}
```

### Get Address-Only Fee Setting
```http
GET /api/v1/merchants/address-only/fee-setting
Authorization: Bearer {api_key}
```

### Update Address-Only Fee Setting
```http
PUT /api/v1/merchants/address-only/fee-setting
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "customer_pays_fee": true
}
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

### List Refunds
```http
GET /api/v1/merchants/refunds
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
GET /api/v1/merchants/analytics/export?from_date=2024-01-01&to_date=2024-01-31&format=csv
Authorization: Bearer {api_key}
```

## Invoice Management

### Create Invoice
```http
POST /api/v1/merchants/invoices
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "customer_email": "customer@example.com",
  "customer_name": "John Doe",
  "items": [
    {
      "description": "Consulting services",
      "quantity": 1,
      "unit_price": "150.00",
      "amount": "150.00"
    }
  ],
  "tax": "0.00",
  "notes": "Payment due within 30 days"
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

### Simulate Payment
```http
POST /api/v1/merchants/sandbox/payments/{payment_id}/simulate
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "success": true,
  "transaction_hash": "0xabc...",
  "from_address": "0xsender..."
}
```

#### Node SDK Example
```javascript
await fiddupay.payments.simulate('payment_123', {
  success: true,
  transaction_hash: '0xabc...',
  from_address: '0xsender...'
});
```

## Security Endpoints

### Set IP Whitelist (Legacy)
> [!WARNING]
> Deprecated in favor of `PATCH /api/v1/merchants/settings`.

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
GET /api/v1/merchants/audit-logs?limit=50&offset=0&action=payment.created
Authorization: Bearer {api_key}
```

## Customer Management (Sub-Accounts)

### Register Customer
```http
POST /api/v1/merchants/customers
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "external_id": "user_12345",
  "email": "customer@example.com",
  "metadata": { "loyalty_tier": "gold" }
}
```

### List Customers
```http
GET /api/v1/merchants/customers?limit=50&offset=0
Authorization: Bearer {api_key}
```

### Get Customer Summary
```http
GET /api/v1/merchants/customers/summary
Authorization: Bearer {api_key}
```
Returns aggregate statistics and total USD balance for all platform customers.

#### Response
```json
{
  "total_customers": 150,
  "active_customers": 142,
  "flagged_customers": 3,
  "recent_customers": 12,
  "total_balance_usd": 12500.50
}
```

#### Node SDK Example
```javascript
const summary = await fiddupay.customers.getSummary();
console.log('Total Platform Deposits:', summary.total_balance_usd);
```

### Update Customer Status
```http
PATCH /api/v1/merchants/customers/{external_id}/status
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "status": "active" // active, suspended, inactive
}
```

### Update Customer Permissions
```http
PATCH /api/v1/merchants/customers/{external_id}/permissions
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "can_withdraw": true,
  "withdrawal_limit": "500.00"
}
```

### Provision Customer Wallets
```http
POST /api/v1/merchants/customers/{external_id}/wallets
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "networks": ["evm", "solana"]
}
```

### Get Customer Balances
```http
GET /api/v1/merchants/customers/{external_id}/balances
Authorization: Bearer {api_key}
```

### Get Customer Deposit Address
```http
GET /api/v1/merchants/customers/{external_id}/deposit-address/{crypto_type}
Authorization: Bearer {api_key}
```

### Get Customer Transactions
```http
GET /api/v1/merchants/customers/{external_id}/transactions
Authorization: Bearer {api_key}
```

### Pay Merchant From Customer Wallet
```http
POST /api/v1/merchants/customers/{external_id}/pay-merchant
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "amount": "1.0",
  "reference_id": "order_123"
}
```

### Withdraw from Customer Wallet
```http
POST /api/v1/merchants/customers/{external_id}/withdraw
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "amount": "1.0",
  "destination_address": "external_wallet_address"
}
```

### Sweep Customer Wallet to Merchant
```http
POST /api/v1/merchants/customers/{external_id}/sweep
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "amount": "1.0" // Optional: Omit to sweep full balance
}
```

### Deactivate Customer
```http
POST /api/v1/merchants/customers/{external_id}/deactivate
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

### Process Withdrawal
```http
POST /api/v1/merchants/withdrawals/{withdrawal_id}/process
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "encryption_password": "your_secure_password"
}
```

## Wallet Management Endpoints

### Unified Wallet Setup
```http
POST /api/v1/merchants/wallets
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "crypto_type": "SOL",
  "mode": "generate", // or "address"
  "address": "external_address_if_mode_is_address",
  "is_active": true
}
```
**Recommended**: Use this single endpoint for all wallet onboarding methods.

### Get Wallet Configs
```http
GET /api/v1/merchants/wallets
Authorization: Bearer {api_key}
```

### Get Wallet Balances
```http
GET /api/v1/merchants/wallets/balances
Authorization: Bearer {api_key}
```
Returns actual on-chain balances and volume statistics for all configured merchant wallets.

### Revoke/Remove Wallet Configuration
```http
DELETE /api/v1/merchants/wallets/{crypto_type}
Authorization: Bearer {api_key}
```
Removes the specified wallet from the merchant profile.

### Configure Address-Only Wallet (Legacy)
> [!WARNING]
> Deprecated in favor of `POST /api/v1/merchants/wallets` with `mode: "address"`.

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
// Unified wallet setup
await fiddupay.wallets.setup({
  crypto_type: 'SOL',
  mode: 'generate', 
  is_active: true
});
```

### Generate Wallet (Legacy)
> [!WARNING]
> Deprecated in favor of `POST /api/v1/merchants/wallets` with `mode: "generate"`.

```http
POST /api/v1/merchants/wallets/generate
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
| `SETTLEMENT_MODE_MISMATCH` | Action forbidden in current settlement mode |

## Supported Cryptocurrencies

- **Bitcoin**: BTC (SegWit/Bech32)
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
  "event_type": "payment.confirmed",
  "payment_id": "pay_abc123",
  "merchant_id": 123,
  "status": "CONFIRMED",
  "amount": "2.5",
  "crypto_type": "SOL",
  "transaction_hash": "5xK9...",
  "timestamp": 1710729600
}
```

### Webhook Events
- `payment.confirmed` — Payment has been confirmed on-chain
- `payment.expired` — Payment expired without confirmation
- `refund.completed` — Refund has been processed and sent

### Supported Webhook Formats
- `standard` — JSON payload with HMAC-SHA256 signature
- `discord` — Discord embed format
- `slack` — Slack message format

### Webhook Signature Verification
Webhooks include a `signature` header in the format: `t={timestamp},v1={hmac_signature}`

Verify by computing `HMAC-SHA256(signing_secret, "{timestamp}.{payload_json}")`.
