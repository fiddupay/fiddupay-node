# PayFlow API Reference

**Version**: 1.0  
**Base URL**: `https://api.payflow.com` (Production) | `http://localhost:8080` (Development)

## Authentication

All API requests require authentication using Bearer tokens:

```http
Authorization: Bearer <your_api_key>
```

Get your API key by registering a merchant account.

## Core Concepts

### Payment Flow
1. **Create Payment** - Generate payment request with amount and crypto type
2. **Customer Pays** - Customer sends crypto to provided deposit address
3. **Verification** - System monitors blockchain and verifies payment
4. **Notification** - Webhook sent to merchant on payment confirmation
5. **Settlement** - Funds forwarded to merchant wallet (minus fees)

### Supported Cryptocurrencies

| Currency | Network | Contract Address | Confirmations |
|----------|---------|------------------|---------------|
| SOL | Solana | Native | 32 |
| USDT_SOL | Solana | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 32 |
| USDT_ETH | Ethereum | `0xdAC17F958D2ee523a2206206994597C13D831ec7` | 12 |
| USDT_BSC | BSC | `0x55d398326f99059fF775485246999027B3197955` | 15 |
| USDT_POLYGON | Polygon | `0xc2132D05D31c914a87C6611C10748AEb04B58e8F` | 30 |
| USDT_ARBITRUM | Arbitrum | `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9` | 1 |

## Endpoints

### Health Check

#### GET /health
Check service health status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-24T15:30:00Z"
}
```

### Merchant Management

#### POST /api/v1/merchants/register
Register a new merchant account.

**Request:**
```json
{
  "business_name": "My Business",
  "email": "merchant@example.com",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "merchant_id": 123,
  "api_key": "your_api_key_here"
}
```

#### POST /api/v1/merchants/api-keys/rotate
Rotate API key for security.

**Headers:** `Authorization: Bearer <current_api_key>`

**Response:**
```json
{
  "api_key": "new_api_key_here"
}
```

#### PUT /api/v1/merchants/wallets
Configure wallet address for a cryptocurrency.

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "crypto_type": "USDT_ETH",
  "address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87"
}
```

**Response:**
```json
{
  "success": true
}
```

#### PUT /api/v1/merchants/webhook
Configure webhook URL for payment notifications.

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "url": "https://your-site.com/webhook"
}
```

**Response:**
```json
{
  "success": true
}
```

### Payment Management

#### POST /api/v1/payments
Create a new payment request.

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "amount_usd": "100.00",
  "crypto_type": "USDT_ETH",
  "description": "Order #12345",
  "metadata": {
    "order_id": "12345",
    "customer_id": "cust_123"
  }
}
```

**Response:**
```json
{
  "payment_id": "pay_abc123",
  "status": "PENDING",
  "amount": "101.50",
  "amount_usd": "101.50",
  "crypto_type": "USDT_ETH",
  "network": "ETHEREUM",
  "deposit_address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
  "payment_link": "https://pay.example.com/pay_abc123",
  "qr_code_data": "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?value=101.50",
  "fee_amount": "1.50",
  "fee_amount_usd": "1.50",
  "expires_at": "2026-01-24T16:30:00Z",
  "created_at": "2026-01-24T15:30:00Z",
  "confirmed_at": null,
  "transaction_hash": null
}
```

#### GET /api/v1/payments
List payments with optional filtering.

**Headers:** `Authorization: Bearer <api_key>`

**Query Parameters:**
- `status` - Filter by payment status (PENDING, CONFIRMED, FAILED, EXPIRED)
- `crypto_type` - Filter by cryptocurrency
- `from_date` - Start date (ISO 8601)
- `to_date` - End date (ISO 8601)
- `page` - Page number (default: 1)
- `page_size` - Items per page (default: 20, max: 100)

**Response:**
```json
{
  "payments": [
    {
      "payment_id": "pay_abc123",
      "status": "CONFIRMED",
      "amount": "101.50",
      "amount_usd": "101.50",
      "crypto_type": "USDT_ETH",
      "network": "ETHEREUM",
      "deposit_address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
      "payment_link": "https://pay.example.com/pay_abc123",
      "qr_code_data": "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?value=101.50",
      "fee_amount": "1.50",
      "fee_amount_usd": "1.50",
      "expires_at": "2026-01-24T16:30:00Z",
      "created_at": "2026-01-24T15:30:00Z",
      "confirmed_at": "2026-01-24T15:35:00Z",
      "transaction_hash": "0xabc123..."
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

#### GET /api/v1/payments/{payment_id}
Get details of a specific payment.

**Headers:** `Authorization: Bearer <api_key>`

**Response:**
```json
{
  "payment_id": "pay_abc123",
  "status": "CONFIRMED",
  "amount": "101.50",
  "amount_usd": "101.50",
  "crypto_type": "USDT_ETH",
  "network": "ETHEREUM",
  "deposit_address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
  "payment_link": "https://pay.example.com/pay_abc123",
  "qr_code_data": "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?value=101.50",
  "fee_amount": "1.50",
  "fee_amount_usd": "1.50",
  "expires_at": "2026-01-24T16:30:00Z",
  "created_at": "2026-01-24T15:30:00Z",
  "confirmed_at": "2026-01-24T15:35:00Z",
  "transaction_hash": "0xabc123..."
}
```

#### POST /api/v1/payments/{payment_id}/verify
Manually verify a payment (useful for testing).

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "transaction_hash": "0xabc123..."
}
```

**Response:**
```json
{
  "success": true,
  "status": "CONFIRMED"
}
```

### Address-Only Payments

#### POST /api/v1/address-only-payments
Create an address-only payment that sends funds directly to merchant address.

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "requested_amount": "100.00",
  "crypto_type": "ETH",
  "merchant_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "description": "Direct payment to merchant wallet"
}
```

**Response:**
```json
{
  "payment_id": "pay_addr_123",
  "requested_amount": "100.00",
  "customer_amount": "100.75",
  "processing_fee": "0.75",
  "crypto_type": "ETH",
  "gateway_deposit_address": "0x1234567890abcdef",
  "customer_pays_fee": true,
  "customer_instructions": "Send exactly 100.75 ETH to the deposit address. This includes the processing fee.",
  "supported_currencies": ["ETH"],
  "expires_at": "2026-01-24T16:30:00Z",
  "created_at": "2026-01-24T15:30:00Z"
}
```

#### GET /api/v1/address-only-payments/{payment_id}
Get details of a specific address-only payment.

**Headers:** `Authorization: Bearer <api_key>`

**Response:**
```json
{
  "payment_id": "pay_addr_123",
  "requested_amount": "100.00",
  "customer_amount": "100.75",
  "processing_fee": "0.75",
  "crypto_type": "ETH",
  "gateway_deposit_address": "0x1234567890abcdef",
  "customer_pays_fee": true,
  "customer_instructions": "Send exactly 100.75 ETH to the deposit address. This includes the processing fee.",
  "supported_currencies": ["ETH"],
  "status": "CONFIRMED",
  "transaction_hash": "0xdef456...",
  "confirmations": 15,
  "expires_at": "2026-01-24T16:30:00Z",
  "created_at": "2026-01-24T15:30:00Z",
  "confirmed_at": "2026-01-24T15:35:00Z"
}
```

### Fee Setting Management

#### POST /api/v1/fee-setting
Update merchant fee payment preference.

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "customer_pays_fee": true
}
```

**Response:**
```json
{
  "success": true,
  "customer_pays_fee": true,
  "message": "Fee payment setting updated: Customer pays fee"
}
```

#### GET /api/v1/fee-setting
Get current fee payment preference.

**Headers:** `Authorization: Bearer <api_key>`

**Response:**
```json
{
  "customer_pays_fee": true,
  "description": "Customer pays processing fee"
}
```

### Balance Management

#### GET /api/v1/balance
Get current account balances.

**Headers:** `Authorization: Bearer <api_key>`

**Response:**
```json
{
  "balances": [
    {
      "crypto_type": "USDT_ETH",
      "available": "1000.00",
      "reserved": "50.00",
      "total": "1050.00"
    }
  ],
  "total_usd": "1050.00",
  "available_usd": "1000.00",
  "reserved_usd": "50.00"
}
```

#### GET /api/v1/balance/history
Get balance history with optional filtering.

**Headers:** `Authorization: Bearer <api_key>`

**Query Parameters:**
- `crypto_type` - Filter by cryptocurrency
- `from_date` - Start date (ISO 8601)
- `to_date` - End date (ISO 8601)
- `page` - Page number (default: 1)
- `page_size` - Items per page (default: 20, max: 100)

**Response:**
```json
{
  "entries": [
    {
      "id": 123,
      "crypto_type": "USDT_ETH",
      "amount": "100.00",
      "balance_after": "1100.00",
      "transaction_type": "CREDIT",
      "description": "Payment received",
      "created_at": "2026-01-24T15:30:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

### Analytics

#### GET /api/v1/analytics
Get payment analytics and statistics.

**Headers:** `Authorization: Bearer <api_key>`

**Query Parameters:**
- `from_date` - Start date (ISO 8601)
- `to_date` - End date (ISO 8601)
- `blockchain` - Filter by blockchain network

**Response:**
```json
{
  "total_volume_usd": "10000.00",
  "successful_payments": 95,
  "failed_payments": 5,
  "total_fees_paid": "150.00",
  "average_transaction_value": "105.26",
  "by_blockchain": {
    "ETHEREUM": {
      "volume_usd": "5000.00",
      "count": 50,
      "average_value": "100.00"
    },
    "SOLANA": {
      "volume_usd": "3000.00",
      "count": 30,
      "average_value": "100.00"
    }
  }
}
```

### Withdrawals

#### POST /api/v1/withdrawals
Create a withdrawal request.

**Headers:** `Authorization: Bearer <api_key>`

**Request:**
```json
{
  "crypto_type": "USDT_ETH",
  "amount": "100.00",
  "destination_address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87"
}
```

**Response:**
```json
{
  "withdrawal_id": "wd_abc123",
  "status": "PENDING",
  "crypto_type": "USDT_ETH",
  "amount": "100.00",
  "destination_address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
  "created_at": "2026-01-24T15:30:00Z"
}
```

#### GET /api/v1/withdrawals
List withdrawals.

**Headers:** `Authorization: Bearer <api_key>`

**Query Parameters:**
- `status` - Filter by status (PENDING, COMPLETED, FAILED, CANCELLED)
- `crypto_type` - Filter by cryptocurrency
- `page` - Page number (default: 1)
- `page_size` - Items per page (default: 20, max: 100)

**Response:**
```json
{
  "withdrawals": [
    {
      "withdrawal_id": "wd_abc123",
      "status": "COMPLETED",
      "crypto_type": "USDT_ETH",
      "amount": "100.00",
      "destination_address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
      "transaction_hash": "0xdef456...",
      "created_at": "2026-01-24T15:30:00Z",
      "completed_at": "2026-01-24T15:35:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

## Webhooks

PayFlow sends webhook notifications for payment events.

### Webhook Events

- `payment.created` - Payment request created
- `payment.confirmed` - Payment confirmed on blockchain
- `payment.failed` - Payment failed or expired
- `withdrawal.completed` - Withdrawal processed

### Webhook Payload

```json
{
  "event": "payment.confirmed",
  "payment_id": "pay_abc123",
  "merchant_id": 123,
  "status": "CONFIRMED",
  "amount": "101.50",
  "amount_usd": "101.50",
  "crypto_type": "USDT_ETH",
  "transaction_hash": "0xabc123...",
  "confirmed_at": "2026-01-24T15:35:00Z",
  "metadata": {
    "order_id": "12345"
  }
}
```

### Webhook Security

Webhooks are signed using HMAC-SHA256. Verify the signature using the `X-PayFlow-Signature` header:

```python
import hmac
import hashlib

def verify_webhook(payload, signature, secret):
    expected = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)
```

## Error Handling

### HTTP Status Codes

- `200` - Success
- `400` - Bad Request (invalid parameters)
- `401` - Unauthorized (invalid API key)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `429` - Too Many Requests (rate limited)
- `500` - Internal Server Error

### Error Response Format

```json
{
  "error": "Invalid API key",
  "message": "The provided API key is not valid",
  "code": "INVALID_API_KEY"
}
```

## Rate Limits

- **Default**: 100 requests per minute per API key
- **Burst**: Up to 200 requests in a 10-second window
- **Headers**: Rate limit info included in response headers

## SDKs and Libraries

### Official SDKs
- **Node.js**: `npm install @payflow/node-sdk`
- **Python**: `pip install payflow-python`
- **PHP**: `composer require payflow/php-sdk`

### Community SDKs
- **Go**: `go get github.com/payflow/go-sdk`
- **Ruby**: `gem install payflow-ruby`

## Testing

### Sandbox Mode

Enable sandbox mode for testing:

```bash
POST /api/v1/merchants/sandbox/enable
```

In sandbox mode:
- No real blockchain transactions
- Instant payment confirmations
- Test webhook deliveries
- Separate test API keys

### Test Cards and Addresses

Use these test addresses for sandbox testing:

**Ethereum (USDT_ETH):**
- `0x742d35Cc6634C0532925a3b8D4C9db96590c6C87`

**Solana (SOL/USDT_SOL):**
- `9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM`

## Support

- **Documentation**: https://docs.payflow.com
- **Support Email**: support@payflow.com
- **Status Page**: https://status.payflow.com
- **Discord**: https://discord.gg/payflow
