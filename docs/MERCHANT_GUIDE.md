# FidduPay Merchant Integration Guide

Complete guide for merchants to integrate FidduPay cryptocurrency payment gateway.

## Daily Volume Limits

**Important**: All merchants have daily volume limits based on KYC status:

- **Non-KYC Merchants**: $1,000 USD daily volume limit (combined deposits + withdrawals)
- **KYC Verified Merchants**: No daily volume limits
- **Reset**: Daily limits reset at midnight UTC
- **Tracking**: Real-time volume tracking across all transaction types

### Check Your Daily Volume Status
```bash
curl -X GET https://api.fiddupay.com/api/v1/merchant/profile \
  -H "Authorization: Bearer your_api_key"
```

Response includes your remaining daily volume:
```json
{
  "id": 123,
  "business_name": "My Store",
  "email": "merchant@example.com",
  "kyc_verified": false,
  "daily_volume_remaining": "750.00"
}
```

## Quick Start

### 1. Register Account
```bash
curl -X POST https://api.fiddupay.com/api/v1/merchant/register \
  -H "Content-Type: application/json" \
  -d '{
    "business_name": "My Store",
    "email": "merchant@example.com",
    "password": "secure_password"
  }'
```

Response:
```json
{
  "merchant_id": 123,
  "api_key": "your_api_key_here"
}
```

### 2. Configure Wallets
```bash
curl -X PUT https://api.fiddupay.com/api/v1/merchant/wallets \
  -H "Authorization: Bearer your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "crypto_type": "USDT_ETH",
    "address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87"
  }'
```

### 3. Set Webhook
```bash
curl -X PUT https://api.fiddupay.com/api/v1/merchant/webhook \
  -H "Authorization: Bearer your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-site.com/webhook"
  }'
```

### 4. Create Payment
```bash
# USD-based payment
curl -X POST https://api.fiddupay.com/api/v1/payments \
  -H "Authorization: Bearer your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "amount_usd": "100.00",
    "crypto_type": "USDT_ETH",
    "description": "Order #12345"
  }'

# Crypto-based payment
curl -X POST https://api.fiddupay.com/api/v1/payments \
  -H "Authorization: Bearer your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": "2.5",
    "crypto_type": "SOL",
    "description": "Order #12345"
  }'
```

## Integration Patterns

### E-commerce Integration

#### WooCommerce Plugin
```php
<?php
class fiddupay_Gateway extends WC_Payment_Gateway {
    public function __construct() {
        $this->id = 'fiddupay';
        $this->title = 'Cryptocurrency';
        $this->description = 'Pay with cryptocurrency';
        $this->api_key = $this->get_option('api_key');
    }
    
    public function process_payment($order_id) {
        $order = wc_get_order($order_id);
        
        $payment = $this->create_payment([
            'amount_usd' => $order->get_total(),
            'crypto_type' => 'USDT_ETH',
            'description' => "Order #{$order_id}",
            'metadata' => ['order_id' => $order_id]
        ]);
        
        return [
            'result' => 'success',
            'redirect' => $payment['payment_link']
        ];
    }
}
```

#### Shopify Integration
```javascript
// Shopify checkout extension
const fiddupayCheckout = {
    async createPayment(orderData) {
        const response = await fetch('/api/fiddupay/create-payment', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${API_KEY}`
            },
            body: JSON.stringify({
                amount_usd: orderData.total,
                crypto_type: 'USDT_ETH',
                description: `Order #${orderData.order_number}`,
                metadata: { order_id: orderData.id }
            })
        });
        
        const payment = await response.json();
        window.location.href = payment.payment_link;
    },

    async createCryptoPayment(orderData, cryptoAmount) {
        const response = await fetch('/api/fiddupay/create-payment', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${API_KEY}`
            },
            body: JSON.stringify({
                amount: cryptoAmount,
                crypto_type: 'SOL',
                description: `Order #${orderData.order_number}`,
                metadata: { order_id: orderData.id }
            })
        });
        
        const payment = await response.json();
        window.location.href = payment.payment_link;
    }
};
```
                'Authorization': `Bearer ${API_KEY}`
            },
            body: JSON.stringify({
                amount_usd: orderData.total,
                crypto_type: 'USDT_ETH',
                description: `Order #${orderData.order_number}`,
                metadata: { order_id: orderData.id }
            })
        });
        
        const payment = await response.json();
        window.location.href = payment.payment_link;
    }
};
```

### Custom Integration

#### Node.js SDK
```javascript
const fiddupay = require('@fiddupay/node-sdk');

const fiddupay = new fiddupay({
    apiKey: 'your_api_key',
    environment: 'production' // or 'sandbox'
});

// Create USD-based payment
const payment = await fiddupay.payments.create({
    amount_usd: '100.00',
    crypto_type: 'USDT_ETH',
    description: 'Order #12345',
    metadata: {
        order_id: '12345',
        customer_id: 'cust_123'
    }
});

// Create crypto-based payment
const cryptoPayment = await fiddupay.payments.create({
    amount: '2.5',
    crypto_type: 'SOL',
    description: 'Order #12345',
    metadata: {
        order_id: '12345',
        customer_id: 'cust_123'
    }
});

console.log('Payment URL:', payment.payment_link);
console.log('QR Code:', payment.qr_code_data);
```

#### Python SDK
```python
import fiddupay

client = fiddupay.Client(
    api_key='YOUR_API_KEY',
    environment='production'
)

# Create USD-based payment
payment = client.payments.create(
    amount_usd='100.00',
    crypto_type='USDT_ETH',
    description='Order #12345',
    metadata={
        'order_id': '12345',
        'customer_id': 'cust_123'
    }
)

# Create crypto-based payment
crypto_payment = client.payments.create(
    amount='2.5',
    crypto_type='SOL',
    description='Order #12345',
    metadata={
        'order_id': '12345',
        'customer_id': 'cust_123'
    }
)

print(f"Payment URL: {payment.payment_link}")
print(f"QR Code: {payment.qr_code_data}")
```

## Webhook Handling

### Webhook Security
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

# Flask example
@app.route('/webhook', methods=['POST'])
def handle_webhook():
    payload = request.get_data(as_text=True)
    signature = request.headers.get('X-fiddupay-Signature')
    
    if not verify_webhook(payload, signature, WEBHOOK_SECRET):
        return 'Invalid signature', 401
    
    event = request.get_json()
    
    if event['event'] == 'payment.confirmed':
        # Update order status
        update_order_status(event['metadata']['order_id'], 'paid')
    
    return 'OK'
```

### Event Types
- `payment.created` - Payment request created
- `payment.confirmed` - Payment confirmed on blockchain
- `payment.failed` - Payment failed or expired
- `withdrawal.completed` - Withdrawal processed

## Payment Flow

### 1. Customer Checkout
```mermaid
sequenceDiagram
    Customer->>Merchant: Checkout
    Merchant->>fiddupay: Create Payment
    fiddupay->>Merchant: Payment Details
    Merchant->>Customer: Payment Page
    Customer->>Blockchain: Send Crypto
    fiddupay->>Merchant: Webhook Notification
    Merchant->>Customer: Order Confirmation
```

### 2. Payment States
- **PENDING** - Waiting for payment
- **CONFIRMED** - Payment received and confirmed
- **FAILED** - Payment failed or insufficient
- **EXPIRED** - Payment expired (default: 15 minutes)

## Supported Cryptocurrencies

| Currency | Network | Contract Address | Confirmations |
|----------|---------|------------------|---------------|
| SOL | Solana | Native | 32 |
| USDT_SOL | Solana | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 32 |
| USDT_ETH | Ethereum | `0xdAC17F958D2ee523a2206206994597C13D831ec7` | 12 |
| USDT_BSC | BSC | `0x55d398326f99059fF775485246999027B3197955` | 15 |
| USDT_POLYGON | Polygon | `0xc2132D05D31c914a87C6611C10748AEb04B58e8F` | 30 |
| USDT_ARBITRUM | Arbitrum | `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9` | 1 |

## Testing

### Sandbox Mode
```bash
# Enable sandbox
curl -X POST https://api.fiddupay.com/api/v1/merchant/sandbox/enable \
  -H "Authorization: Bearer your_api_key"

# Simulate payment
curl -X POST https://api.fiddupay.com/api/v1/sandbox/simulate-payment \
  -H "Authorization: Bearer your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "payment_id": "pay_abc123",
    "action": "confirm"
  }'
```

### Test Addresses
Use these addresses for sandbox testing:
- **Ethereum**: `0x742d35Cc6634C0532925a3b8D4C9db96590c6C87`
- **Solana**: `9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM`

## Error Handling

### Common Errors
```json
{
  "error": "Invalid API key",
  "message": "The provided API key is not valid",
  "code": "INVALID_API_KEY"
}
```

### Error Codes
- `INVALID_API_KEY` - API key is invalid or expired
- `WALLET_NOT_FOUND` - Wallet not configured for crypto type
- `INSUFFICIENT_BALANCE` - Insufficient balance for withdrawal
- `PAYMENT_NOT_FOUND` - Payment ID not found
- `PAYMENT_EXPIRED` - Payment has expired

## Best Practices

### Security
- Store API keys securely (environment variables)
- Verify webhook signatures
- Use HTTPS for all communications
- Implement proper error handling
- Log all transactions

### Performance
- Cache payment status locally
- Use webhooks instead of polling
- Implement retry logic for API calls
- Monitor API rate limits
- Use connection pooling

### User Experience
- Show clear payment instructions
- Display QR codes for mobile payments
- Provide payment status updates
- Handle payment timeouts gracefully
- Offer multiple payment options

## Support

### Resources
- **API Documentation**: https://docs.fiddupay.com
- **SDKs**: https://github.com/fiddupay
- **Status Page**: https://status.fiddupay.com
- **Support**: support@fiddupay.com

### Community
- **Discord**: https://discord.gg/fiddupay
- **GitHub**: https://github.com/fiddupay
- **Stack Overflow**: Tag `fiddupay`

```
Step 1: Sign Up
> Merchant visits: https://yourdomain.com/signup
> Provides:
   > Email address
   > Business name
   > Password
   > Business type (optional)
> System generates:
    > Unique merchant_id
    > API key (for production)
    > Sandbox API key (for testing)
```

**API Endpoint:**
```bash
POST /api/v1/merchant/register
{
  "email": "merchant@example.com",
  "business_name": "My Store"
}

Response:
{
  "merchant_id": 1,
  "api_key": "live_your_api_key_here",
  "sandbox_api_key": "test_xyz789..."
}
```

### 1.2 Account Setup

```
Step 2: Configure Wallets
> Merchant sets receiving addresses for each blockchain:
   > Solana: 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
   > BSC: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
   > Arbitrum: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
   > Polygon: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
> System validates addresses
```

**API Endpoint:**
```bash
PUT /api/v1/merchant/wallets
Authorization: Bearer live_your_api_key_here
{
  "crypto_type": "SOL",
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

### 1.3 Webhook Configuration

```
Step 3: Set Webhook URL
> Merchant provides webhook endpoint
> System validates HTTPS
> Merchant receives test webhook
```

**API Endpoint:**
```bash
PUT /api/v1/merchant/webhook
Authorization: Bearer live_your_api_key_here
{
  "url": "https://merchant-site.com/webhooks/crypto-payments"
}
```

### 1.4 Currency Selection

```
Step 4: Choose Accepted Currencies
> Merchant selects which currencies to accept:
   > [x] SOL
   > [x] USDT (Solana)
   > [x] USDT (BSC)
   > [ ] USDT (Arbitrum)
   > [x] USDT (Polygon)
> System updates merchant preferences
```

**Implementation:** Add `accepted_currencies` JSON field to merchants table

---

## 2. PAYMENT PROCESSING

### 2.1 Creating a Payment

```
Payment Creation Flow:
> Merchant creates payment request
> System:
   > Generates unique payment_id (pay_abc123)
   > Fetches current crypto price
   > Calculates crypto amount (USD / price)
   > Adds platform fee (1.5% default)
   > Generates payment link (https://pay.yourdomain.com/lnk_xyz)
   > Creates QR code
> Returns payment details to merchant
```

**API Endpoint:**
```bash
POST /api/v1/payments
Authorization: Bearer live_your_api_key_here
{
  "amount_usd": 100.00,
  "crypto_type": "SOL",
  "description": "Order #12345",
  "metadata": {
    "order_id": "12345",
    "customer_email": "customer@example.com"
  },
  "expiration_minutes": 15
}

Response:
{
  "payment_id": "pay_abc123",
  "status": "PENDING",
  "amount": 0.45,
  "amount_usd": 100.00,
  "crypto_type": "SOL",
  "network": "SOLANA",
  "deposit_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "payment_link": "https://pay.yourdomain.com/lnk_xyz789",
  "qr_code_data": "solana:7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU?amount=0.45",
  "fee_amount": 0.0068,
  "fee_amount_usd": 1.50,
  "expires_at": "2026-01-20T10:00:00Z",
  "created_at": "2026-01-20T09:45:00Z"
}
```

### 2.2 Payment Verification

```
Verification Flow:
> Customer sends crypto to deposit_address
> Background monitor detects transaction
> System verifies:
   > Amount matches (within 0.1% tolerance)
   > Address matches merchant wallet
   > Transaction has required confirmations
   > Payment hasn't expired
> Updates payment status: PENDING → CONFIRMING → CONFIRMED
> Triggers webhook notification
```

**Automatic:** Background task runs every 30 seconds

**Manual Verification:**
```bash
POST /api/v1/payments/pay_abc123/verify
Authorization: Bearer live_your_api_key_here
{
  "transaction_hash": "5j7s...xyz"
}
```

### 2.3 Payment States

```
Payment Lifecycle:
PENDING > CONFIRMING > CONFIRMED
                                       
                                       > REFUNDED
   
   > FAILED (expired or invalid)
```

---

## 3. INVOICE MANAGEMENT

### 3.1 Invoice Creation (Future Feature)

```
Invoice Flow:
> Merchant creates invoice with line items
> System generates invoice_id
> Customer receives invoice link
> Customer pays invoice
> Invoice marked as paid
```

**Proposed API:**
```bash
POST /api/v1/invoices
Authorization: Bearer live_your_api_key_here
{
  "customer_email": "customer@example.com",
  "due_date": "2026-02-01",
  "items": [
    {
      "description": "Web Development",
      "quantity": 10,
      "unit_price": 50.00
    },
    {
      "description": "Hosting (1 year)",
      "quantity": 1,
      "unit_price": 120.00
    }
  ],
  "notes": "Payment due within 30 days"
}

Response:
{
  "invoice_id": "inv_abc123",
  "total_amount": 620.00,
  "status": "UNPAID",
  "invoice_url": "https://pay.yourdomain.com/invoice/inv_abc123",
  "pdf_url": "https://pay.yourdomain.com/invoice/inv_abc123/pdf"
}
```

---

## 4. BALANCE & WITHDRAWALS

### 4.1 Balance Tracking (Future Feature)

```
Balance Structure:
> Total Balance
   > Available Balance (can withdraw)
   > Reserved Balance (pending payments)
> Per Currency:
   > SOL: 10.5 ($2,310.00)
   > USDT (Solana): 1,500.00
   > USDT (BSC): 800.00
   > USDT (Polygon): 300.00
> Historical Balance (chart)
```

**Proposed API:**
```bash
GET /api/v1/merchant/balance
Authorization: Bearer live_your_api_key_here

Response:
{
  "total_usd": 4,910.00,
  "available_usd": 4,500.00,
  "reserved_usd": 410.00,
  "balances": [
    {
      "crypto_type": "SOL",
      "amount": 10.5,
      "amount_usd": 2310.00,
      "available": 10.0,
      "reserved": 0.5
    },
    {
      "crypto_type": "USDT_SPL",
      "amount": 1500.00,
      "amount_usd": 1500.00,
      "available": 1500.00,
      "reserved": 0
    }
  ]
}
```

### 4.2 Daily Volume Limits

**Non-KYC Merchants:**
- **Daily Limit**: $1,000 USD total volume
- **Combined Tracking**: All deposits + withdrawals count toward daily limit
- **Reset**: Daily at midnight UTC
- **No Per-Transaction Limits**: Individual transactions can be any amount up to remaining daily volume

**KYC Verified Merchants:**
- **No Limits**: Unlimited daily volume
- **Full Access**: No restrictions on transaction amounts or frequency

```
Daily Volume Check:
> Check merchant KYC status
> If KYC verified: Allow transaction
> If non-KYC:
   > Calculate today's volume (deposits + withdrawals)
   > Check if transaction + current volume ≤ $1,000
   > Allow or reject based on limit
```

**Check Remaining Volume:**
```bash
GET /api/v1/merchant/profile
Authorization: Bearer sk_abc123...

Response:
{
  "kyc_verified": false,
  "daily_volume_remaining": "750.00"
}
```

### 4.3 Withdrawal Flow (Future Feature)

```
Withdrawal Process:
> Merchant requests withdrawal
> System checks:
   > Available balance sufficient?
   > Daily volume limit (for non-KYC)?
   > 2FA verified?
> Creates withdrawal request
> Admin/System approves (or auto-approve if < $1000)
> Processes blockchain transaction
> Updates balance
```

**Proposed API:**
```bash
POST /api/v1/withdrawals
Authorization: Bearer live_your_api_key_here
{
  "crypto_type": "USDT_SPL",
  "amount": 500.00,
  "destination_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "two_factor_code": "123456"
}

Response:
{
  "withdrawal_id": "wd_abc123",
  "status": "PENDING",
  "amount": 500.00,
  "fee": 1.00,
  "net_amount": 499.00,
  "estimated_completion": "2026-01-20T10:30:00Z"
}
```

---

## 5. RECEIVING PAYMENTS

### 5.1 Direct Integration

```
Merchant Website Integration:
> Customer clicks "Pay with Crypto"
> Merchant backend calls API to create payment
> Merchant displays payment details or redirects to payment link
> Customer completes payment
> Merchant receives webhook notification
> Merchant fulfills order
```

**Example (JavaScript):**
```javascript
// Merchant backend
const response = await fetch('https://api.yourdomain.com/api/v1/payments', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer live_your_api_key_here',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    amount_usd: 100.00,
    crypto_type: 'SOL',
    description: 'Order #12345'
  })
});

const payment = await response.json();

// Redirect customer to payment page
window.location.href = payment.payment_link;
```

### 5.2 Hosted Payment Page

```
Customer Experience:
> Customer visits payment link
> Sees payment page with:
   > Amount in crypto and USD
   > QR code
   > Wallet address (copy button)
   > Network information
   > Countdown timer
   > Status (pending/confirmed/expired)
> Customer scans QR or copies address
> Sends payment from wallet
> Page auto-updates when confirmed
```

**Features:**
- Mobile-responsive
- Real-time status updates (polling every 5s)
- Multiple language support (future)
- Custom branding (future)

---

## 6. PAYMENT LINKS & QR CODES

### 6.1 Payment Link Generation

```
Link Structure:
https://pay.yourdomain.com/lnk_abc123xyz

Components:
> Domain: pay.yourdomain.com
> Path: /lnk_
> ID: abc123xyz (12 characters, unique)
```

**Current Implementation:**  Done

### 6.2 QR Code Format

```
QR Code Data Formats:

Solana:
solana:7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU?amount=0.45

EVM (BSC/Arbitrum/Polygon):
ethereum:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb?value=100000000000000000

Bitcoin (Future):
bitcoin:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa?amount=0.001
```

**Current Implementation:**  Text-based QR (upgrade to image recommended)

### 6.3 Dynamic QR Codes (Future)

```
Features:
> Update amount in real-time
> Show payment status in QR
> Multi-currency QR (customer chooses)
> Branded QR codes
```

---

## 7. SECURITY

### 7.1 API Key Security

```
Current Implementation:
> API keys hashed with Argon2 
> Keys prefixed: live_ or test_
> Key rotation supported 
> Keys never logged in plaintext 
```

**Best Practices:**
```bash
# Store API keys in environment variables

# Never commit to git
echo "*.env" >> .gitignore

# Rotate keys regularly
POST /api/v1/merchant/api-keys/rotate
```

### 7.2 Two-Factor Authentication (Future)

```
2FA Flow:
> Merchant enables 2FA
> Scans QR code with authenticator app
> Enters 6-digit code to verify
> Required for:
    > Withdrawals
    > API key rotation
    > Wallet address changes
    > Sensitive settings
```

### 7.3 IP Whitelisting

```
Current Implementation: 
> Merchant adds allowed IPs
> Supports CIDR ranges
> Max 10 entries per merchant
> Empty whitelist = allow all
```

**API:**
```bash
PUT /api/v1/merchant/ip-whitelist
Authorization: Bearer live_your_api_key_here
{
  "ip_addresses": [
    "203.0.113.0/24",
    "198.51.100.42"
  ]
}
```

### 7.4 Webhook Signatures

```
Current Implementation: 
> HMAC-SHA256 signature
> Included in X-Signature header
> Timestamp in X-Timestamp header
> Prevents replay attacks
```

**Verification (Merchant Side):**
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

### 7.5 Rate Limiting

```
Current Implementation: 
> 100 requests per minute per API key
> Returns 429 when exceeded
> Resets every minute
```

### 7.6 Fraud Prevention (Future)

```
Proposed Features:
> Velocity checks (max payments per hour)
> Blacklist addresses
> Risk scoring
> Suspicious activity alerts
> Manual review queue
```

---

## 8. CURRENCY SUPPORT

### 8.1 Currently Supported

```
 Solana (SOL)
 USDT on Solana (SPL)
 USDT on BSC (BEP20)
 USDT on Arbitrum
 USDT on Polygon
```

### 8.2 Merchant Currency Toggle (Future Feature)

```
Proposed UI:

 Accepted Currencies                 

 [x] SOL (Solana)           1.5% fee 
 [x] USDT (Solana)          0% fee   
 [x] USDT (BSC)             0% fee   
 [ ] USDT (Arbitrum)        0% fee   
 [x] USDT (Polygon)         0% fee   
 [ ] ETH (Ethereum)         1.5% fee 
 [ ] USDC (Polygon)         0% fee   

```

**Implementation:**
```sql
-- Add to merchants table
ALTER TABLE merchants ADD COLUMN accepted_currencies JSONB DEFAULT '["SOL", "USDT_SPL", "USDT_BEP20", "USDT_ARBITRUM", "USDT_POLYGON"]';

-- Query
SELECT accepted_currencies FROM merchants WHERE id = 1;
```

**API:**
```bash
PUT /api/v1/merchant/currencies
Authorization: Bearer live_your_api_key_here
{
  "accepted_currencies": ["SOL", "USDT_SPL", "USDT_POLYGON"]
}
```

### 8.3 Adding New Currencies

```
Process to Add New Currency:
> 1. Add to CryptoType enum
> 2. Implement blockchain monitor
> 3. Add price fetching
> 4. Add address validation
> 5. Update documentation
> 6. Test on testnet
```

---

## 9. WEBHOOKS & NOTIFICATIONS

### 9.1 Webhook Events

```
Current Events:
> payment.confirmed
> payment.expired
> refund.created
> refund.completed
```

**Webhook Payload:**
```json
{
  "event_type": "payment.confirmed",
  "payment_id": "pay_abc123",
  "merchant_id": 1,
  "status": "CONFIRMED",
  "amount": 0.45,
  "crypto_type": "SOL",
  "transaction_hash": "5j7s...xyz",
  "timestamp": 1737364800
}
```

### 9.2 Webhook Retry Logic

```
Current Implementation: 
> Retry up to 5 times
> Exponential backoff: 1s, 2s, 4s, 8s, 16s
> Logs all attempts
> Marks as failed after 5 attempts
```

### 9.3 Email Notifications (Future)

```
Proposed Notifications:
> To Merchant:
   > Payment received
   > Withdrawal completed
   > API key rotated
   > Security alerts
> To Customer:
    > Payment confirmation
    > Receipt
    > Refund processed
```

---

## 10. ANALYTICS & REPORTING

### 10.1 Available Analytics

```
Current Implementation: 
> Total volume (USD)
> Payment counts (successful/failed)
> Total fees paid
> Average transaction value
> Breakdown by blockchain
> Date range filtering
```

**API:**
```bash
GET /api/v1/analytics?from=2026-01-01&to=2026-01-31
Authorization: Bearer live_your_api_key_here

Response:
{
  "total_volume_usd": 15420.50,
  "successful_payments": 142,
  "failed_payments": 8,
  "total_fees_paid": 231.31,
  "average_transaction_value": 108.60,
  "by_blockchain": {
    "SOLANA": {
      "volume_usd": 8500.00,
      "count": 85
    },
    "BEP20": {
      "volume_usd": 6920.50,
      "count": 57
    }
  }
}
```

### 10.2 CSV Export

```
Current Implementation: 
GET /api/v1/analytics/export?from=2026-01-01&to=2026-01-31

Returns CSV with:
payment_id, date, amount, crypto_type, status, fee, transaction_hash
```

### 10.3 Advanced Analytics (Future)

```
Proposed Metrics:
> Revenue forecasting
> Customer lifetime value
> Conversion rates
> Geographic distribution
> Peak transaction times
> Currency preferences
> Refund rates
```

---

## QUICK REFERENCE

### Essential API Endpoints

```bash
# Authentication
All requests: Authorization: Bearer <api_key>

# Merchant Setup
POST   /api/v1/merchant/register
PUT    /api/v1/merchant/wallets
PUT    /api/v1/merchant/webhook

# Payments
POST   /api/v1/payments
GET    /api/v1/payments/:payment_id
GET    /api/v1/payments
POST   /api/v1/payments/:payment_id/verify

# Refunds
POST   /api/v1/refunds
GET    /api/v1/refunds/:refund_id

# Analytics
GET    /api/v1/analytics
GET    /api/v1/analytics/export

# Sandbox
POST   /api/v1/sandbox/enable
POST   /api/v1/sandbox/payments/:payment_id/simulate
```

### Status Codes

```
200 OK - Success
201 Created - Resource created
400 Bad Request - Invalid input
401 Unauthorized - Invalid API key
403 Forbidden - IP not whitelisted
404 Not Found - Resource not found
429 Too Many Requests - Rate limit exceeded
500 Internal Server Error - Server error
```

---

## NEXT STEPS FOR MERCHANTS

1.  Register account
2.  Configure wallet addresses
3.  Set webhook URL
4.  Test in sandbox mode
5.  Integrate API
6.  Go live!

**Support:** support@yourdomain.com
**Documentation:** https://docs.yourdomain.com
**Status Page:** https://status.yourdomain.com
