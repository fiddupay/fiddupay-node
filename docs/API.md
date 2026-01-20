# Crypto Payment Gateway - API Documentation

## Overview

The Crypto Payment Gateway API enables merchants to accept cryptocurrency payments across multiple blockchains. This RESTful API uses JSON for requests and responses.

**Base URL:** `https://api.cryptogateway.com`

## Documentation Formats

We provide API documentation in multiple formats:

1. **OpenAPI/Swagger** (`openapi.yaml`) - Industry standard, used by Stripe, Twilio, AWS
   - View in Swagger UI: https://editor.swagger.io
   - Import into API clients (Insomnia, Paw, etc.)
   - Generate client SDKs automatically

2. **Postman Collection** (`postman_collection.json`) - Developer favorite
   - Import into Postman
   - Test APIs interactively
   - Share with team members

## Quick Start

### 1. Register Your Account

```bash
curl -X POST https://api.cryptogateway.com/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "merchant@example.com",
    "business_name": "My Store"
  }'
```

Response:
```json
{
  "merchant_id": 1,
  "email": "merchant@example.com",
  "business_name": "My Store",
  "api_key": "live_abc123...",
  "sandbox_api_key": "test_xyz789..."
}
```

**Save your API keys securely!**

### 2. Configure Wallet Addresses

```bash
curl -X PUT https://api.cryptogateway.com/api/v1/merchants/wallets \
  -H "Authorization: Bearer live_abc123..." \
  -H "Content-Type: application/json" \
  -d '{
    "crypto_type": "SOL",
    "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
  }'
```

### 3. Set Webhook URL

```bash
curl -X PUT https://api.cryptogateway.com/api/v1/merchants/webhook \
  -H "Authorization: Bearer live_abc123..." \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-site.com/webhooks/crypto-payments"
  }'
```

### 4. Create Your First Payment

```bash
curl -X POST https://api.cryptogateway.com/api/v1/payments \
  -H "Authorization: Bearer live_abc123..." \
  -H "Content-Type: application/json" \
  -d '{
    "amount_usd": 100.00,
    "crypto_type": "SOL",
    "description": "Order #12345",
    "expiration_minutes": 15
  }'
```

Response:
```json
{
  "payment_id": "pay_abc123",
  "status": "PENDING",
  "amount": "0.45",
  "amount_usd": "100.00",
  "crypto_type": "SOL",
  "deposit_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "payment_link": "https://pay.cryptogateway.com/lnk_xyz789",
  "qr_code_data": "solana:7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU?amount=0.45",
  "fee_amount": "0.0068",
  "fee_amount_usd": "1.50",
  "expires_at": "2026-01-20T10:00:00Z"
}
```

### 5. Redirect Customer

Redirect your customer to the `payment_link` or display the QR code.

### 6. Receive Webhook

When payment is confirmed, you'll receive a webhook:

```json
{
  "event_type": "payment.confirmed",
  "payment_id": "pay_abc123",
  "status": "CONFIRMED",
  "amount": "0.45",
  "crypto_type": "SOL",
  "transaction_hash": "5j7s...xyz",
  "timestamp": 1737364800
}
```

## Authentication

All API requests (except registration) require authentication using an API key in the `Authorization` header:

```
Authorization: Bearer your_api_key_here
```

### API Key Types

- **Live keys** (`live_*`): For production payments
- **Test keys** (`test_*`): For sandbox testing

## Rate Limiting

- **Limit:** 100 requests per minute per API key
- **Response:** 429 Too Many Requests when exceeded
- **Headers:**
  - `X-RateLimit-Limit`: Maximum requests per minute
  - `X-RateLimit-Remaining`: Remaining requests
  - `X-RateLimit-Reset`: Unix timestamp when limit resets

## Supported Cryptocurrencies

| Crypto Type | Network | Fee |
|-------------|---------|-----|
| `SOL` | Solana | 1.5% |
| `USDT_SPL` | Solana (SPL) | 0% |
| `USDT_BEP20` | Binance Smart Chain | 0% |
| `USDT_ARBITRUM` | Arbitrum One | 0% |
| `USDT_POLYGON` | Polygon | 0% |

## Payment Statuses

| Status | Description |
|--------|-------------|
| `PENDING` | Waiting for customer payment |
| `CONFIRMING` | Transaction detected, awaiting confirmations |
| `CONFIRMED` | Payment confirmed on blockchain |
| `FAILED` | Payment failed or invalid |
| `EXPIRED` | Payment expired before completion |

## Webhooks

### Configuration

Set your webhook URL via API:

```bash
PUT /api/v1/merchants/webhook
{
  "url": "https://your-site.com/webhooks/crypto-payments"
}
```

**Requirements:**
- Must use HTTPS
- Must respond with 2xx status code
- Should respond within 5 seconds

### Events

| Event | Description |
|-------|-------------|
| `payment.confirmed` | Payment confirmed on blockchain |
| `payment.expired` | Payment expired |
| `refund.created` | Refund initiated |
| `refund.completed` | Refund completed |

### Signature Verification

All webhooks include HMAC-SHA256 signatures for security:

**Headers:**
- `X-Signature`: HMAC signature
- `X-Timestamp`: Unix timestamp

**Verification (Node.js):**
```javascript
const crypto = require('crypto');

function verifyWebhook(payload, signature, timestamp, secret) {
  const data = timestamp + '.' + JSON.stringify(payload);
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(data)
    .digest('hex');
  
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expectedSignature)
  );
}
```

**Verification (Python):**
```python
import hmac
import hashlib

def verify_webhook(payload, signature, timestamp, secret):
    data = f"{timestamp}.{json.dumps(payload)}"
    expected_signature = hmac.new(
        secret.encode(),
        data.encode(),
        hashlib.sha256
    ).hexdigest()
    
    return hmac.compare_digest(signature, expected_signature)
```

**Verification (PHP):**
```php
function verifyWebhook($payload, $signature, $timestamp, $secret) {
    $data = $timestamp . '.' . json_encode($payload);
    $expectedSignature = hash_hmac('sha256', $data, $secret);
    
    return hash_equals($signature, $expectedSignature);
}
```

### Retry Logic

Failed webhooks are retried automatically:
- **Attempts:** Up to 5 retries
- **Backoff:** Exponential (1s, 2s, 4s, 8s, 16s)
- **Timeout:** 5 seconds per attempt

## Error Handling

### Error Response Format

```json
{
  "error": "Error message here"
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Invalid API key |
| 403 | Forbidden - IP not whitelisted |
| 404 | Not Found - Resource doesn't exist |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |

## Code Examples

### JavaScript/Node.js

```javascript
const axios = require('axios');

const client = axios.create({
  baseURL: 'https://api.cryptogateway.com',
  headers: {
    'Authorization': 'Bearer live_abc123...',
    'Content-Type': 'application/json'
  }
});

// Create payment
async function createPayment() {
  const response = await client.post('/api/v1/payments', {
    amount_usd: 100.00,
    crypto_type: 'SOL',
    description: 'Order #12345'
  });
  
  return response.data;
}

// Get payment
async function getPayment(paymentId) {
  const response = await client.get(`/api/v1/payments/${paymentId}`);
  return response.data;
}
```

### Python

```python
import requests

class CryptoGatewayClient:
    def __init__(self, api_key):
        self.base_url = 'https://api.cryptogateway.com'
        self.headers = {
            'Authorization': f'Bearer {api_key}',
            'Content-Type': 'application/json'
        }
    
    def create_payment(self, amount_usd, crypto_type, description):
        response = requests.post(
            f'{self.base_url}/api/v1/payments',
            headers=self.headers,
            json={
                'amount_usd': amount_usd,
                'crypto_type': crypto_type,
                'description': description
            }
        )
        return response.json()
    
    def get_payment(self, payment_id):
        response = requests.get(
            f'{self.base_url}/api/v1/payments/{payment_id}',
            headers=self.headers
        )
        return response.json()

# Usage
client = CryptoGatewayClient('live_abc123...')
payment = client.create_payment(100.00, 'SOL', 'Order #12345')
```

### PHP

```php
<?php

class CryptoGatewayClient {
    private $apiKey;
    private $baseUrl = 'https://api.cryptogateway.com';
    
    public function __construct($apiKey) {
        $this->apiKey = $apiKey;
    }
    
    public function createPayment($amountUsd, $cryptoType, $description) {
        $ch = curl_init($this->baseUrl . '/api/v1/payments');
        
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_POST => true,
            CURLOPT_HTTPHEADER => [
                'Authorization: Bearer ' . $this->apiKey,
                'Content-Type: application/json'
            ],
            CURLOPT_POSTFIELDS => json_encode([
                'amount_usd' => $amountUsd,
                'crypto_type' => $cryptoType,
                'description' => $description
            ])
        ]);
        
        $response = curl_exec($ch);
        curl_close($ch);
        
        return json_decode($response, true);
    }
    
    public function getPayment($paymentId) {
        $ch = curl_init($this->baseUrl . '/api/v1/payments/' . $paymentId);
        
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_HTTPHEADER => [
                'Authorization: Bearer ' . $this->apiKey
            ]
        ]);
        
        $response = curl_exec($ch);
        curl_close($ch);
        
        return json_decode($response, true);
    }
}

// Usage
$client = new CryptoGatewayClient('live_abc123...');
$payment = $client->createPayment(100.00, 'SOL', 'Order #12345');
```

## Testing with Sandbox

### Enable Sandbox Mode

```bash
curl -X POST https://api.cryptogateway.com/api/v1/sandbox/enable \
  -H "Authorization: Bearer live_abc123..."
```

### Use Test API Key

Replace your live key with the test key for all requests.

### Simulate Payment Confirmation

```bash
curl -X POST https://api.cryptogateway.com/api/v1/sandbox/payments/pay_abc123/simulate \
  -H "Authorization: Bearer test_xyz789..." \
  -H "Content-Type: application/json" \
  -d '{
    "status": "CONFIRMED"
  }'
```

## Best Practices

### Security

1. **Never expose API keys** in client-side code
2. **Use HTTPS** for all webhook endpoints
3. **Verify webhook signatures** before processing
4. **Rotate API keys** regularly
5. **Use IP whitelisting** for production
6. **Enable 2FA** on your account (coming soon)

### Performance

1. **Cache prices** for 30 seconds
2. **Use webhooks** instead of polling
3. **Implement exponential backoff** for retries
4. **Handle rate limits** gracefully

### Reliability

1. **Store payment_id** in your database
2. **Make webhook handlers idempotent**
3. **Log all API responses**
4. **Monitor webhook delivery**
5. **Set up alerts** for failed payments

## Support

- **Documentation:** https://docs.cryptogateway.com
- **Email:** support@cryptogateway.com
- **Status Page:** https://status.cryptogateway.com
- **GitHub:** https://github.com/cryptogateway

## Changelog

### v1.0.0 (2026-01-20)
- Initial release
- Multi-blockchain support (Solana, BSC, Arbitrum, Polygon)
- Payment creation and verification
- Webhook notifications
- Refund system
- Analytics and reporting
- Sandbox mode
- IP whitelisting
- Audit logging
