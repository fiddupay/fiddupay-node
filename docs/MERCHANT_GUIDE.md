# Crypto Payment Gateway - Complete Merchant Flow Documentation

## Table of Contents
1. [Merchant Onboarding](#merchant-onboarding)
2. [Payment Processing](#payment-processing)
3. [Invoice Management](#invoice-management)
4. [Balance & Withdrawals](#balance--withdrawals)
5. [Receiving Payments](#receiving-payments)
6. [Payment Links & QR Codes](#payment-links--qr-codes)
7. [Security](#security)
8. [Currency Support](#currency-support)
9. [Webhooks & Notifications](#webhooks--notifications)
10. [Analytics & Reporting](#analytics--reporting)

---

## 1. MERCHANT ONBOARDING

### 1.1 Registration Flow

```
Step 1: Sign Up
├─> Merchant visits: https://yourdomain.com/signup
├─> Provides:
│   ├─> Email address
│   ├─> Business name
│   ├─> Password
│   └─> Business type (optional)
└─> System generates:
    ├─> Unique merchant_id
    ├─> API key (for production)
    └─> Sandbox API key (for testing)
```

**API Endpoint:**
```bash
POST /api/v1/merchants/register
{
  "email": "merchant@example.com",
  "business_name": "My Store"
}

Response:
{
  "merchant_id": 1,
  "api_key": "live_abc123...",
  "sandbox_api_key": "test_xyz789..."
}
```

### 1.2 Account Setup

```
Step 2: Configure Wallets
├─> Merchant sets receiving addresses for each blockchain:
│   ├─> Solana: 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
│   ├─> BSC: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
│   ├─> Arbitrum: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
│   └─> Polygon: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
└─> System validates addresses
```

**API Endpoint:**
```bash
PUT /api/v1/merchants/wallets
Authorization: Bearer live_abc123...
{
  "crypto_type": "SOL",
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

### 1.3 Webhook Configuration

```
Step 3: Set Webhook URL
├─> Merchant provides webhook endpoint
├─> System validates HTTPS
└─> Merchant receives test webhook
```

**API Endpoint:**
```bash
PUT /api/v1/merchants/webhook
Authorization: Bearer live_abc123...
{
  "url": "https://merchant-site.com/webhooks/crypto-payments"
}
```

### 1.4 Currency Selection

```
Step 4: Choose Accepted Currencies
├─> Merchant selects which currencies to accept:
│   ├─> [x] SOL
│   ├─> [x] USDT (Solana)
│   ├─> [x] USDT (BSC)
│   ├─> [ ] USDT (Arbitrum)
│   └─> [x] USDT (Polygon)
└─> System updates merchant preferences
```

**Implementation:** Add `accepted_currencies` JSON field to merchants table

---

## 2. PAYMENT PROCESSING

### 2.1 Creating a Payment

```
Payment Creation Flow:
├─> Merchant creates payment request
├─> System:
│   ├─> Generates unique payment_id (pay_abc123)
│   ├─> Fetches current crypto price
│   ├─> Calculates crypto amount (USD / price)
│   ├─> Adds platform fee (1.5% default)
│   ├─> Generates payment link (https://pay.yourdomain.com/lnk_xyz)
│   └─> Creates QR code
└─> Returns payment details to merchant
```

**API Endpoint:**
```bash
POST /api/v1/payments
Authorization: Bearer live_abc123...
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
├─> Customer sends crypto to deposit_address
├─> Background monitor detects transaction
├─> System verifies:
│   ├─> Amount matches (within 0.1% tolerance)
│   ├─> Address matches merchant wallet
│   ├─> Transaction has required confirmations
│   └─> Payment hasn't expired
├─> Updates payment status: PENDING → CONFIRMING → CONFIRMED
└─> Triggers webhook notification
```

**Automatic:** Background task runs every 30 seconds

**Manual Verification:**
```bash
POST /api/v1/payments/pay_abc123/verify
Authorization: Bearer live_abc123...
{
  "transaction_hash": "5j7s...xyz"
}
```

### 2.3 Payment States

```
Payment Lifecycle:
PENDING ──────> CONFIRMING ──────> CONFIRMED
   │                                    │
   │                                    └──> REFUNDED
   │
   └──> FAILED (expired or invalid)
```

---

## 3. INVOICE MANAGEMENT

### 3.1 Invoice Creation (Future Feature)

```
Invoice Flow:
├─> Merchant creates invoice with line items
├─> System generates invoice_id
├─> Customer receives invoice link
├─> Customer pays invoice
└─> Invoice marked as paid
```

**Proposed API:**
```bash
POST /api/v1/invoices
Authorization: Bearer live_abc123...
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
├─> Total Balance
│   ├─> Available Balance (can withdraw)
│   └─> Reserved Balance (pending payments)
├─> Per Currency:
│   ├─> SOL: 10.5 ($2,310.00)
│   ├─> USDT (Solana): 1,500.00
│   ├─> USDT (BSC): 800.00
│   └─> USDT (Polygon): 300.00
└─> Historical Balance (chart)
```

**Proposed API:**
```bash
GET /api/v1/merchants/balance
Authorization: Bearer live_abc123...

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

### 4.2 Withdrawal Flow (Future Feature)

```
Withdrawal Process:
├─> Merchant requests withdrawal
├─> System checks:
│   ├─> Available balance sufficient?
│   ├─> Minimum withdrawal met? ($10)
│   └─> 2FA verified?
├─> Creates withdrawal request
├─> Admin/System approves (or auto-approve if < $1000)
├─> Processes blockchain transaction
└─> Updates balance
```

**Proposed API:**
```bash
POST /api/v1/withdrawals
Authorization: Bearer live_abc123...
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
├─> Customer clicks "Pay with Crypto"
├─> Merchant backend calls API to create payment
├─> Merchant displays payment details or redirects to payment link
├─> Customer completes payment
├─> Merchant receives webhook notification
└─> Merchant fulfills order
```

**Example (JavaScript):**
```javascript
// Merchant backend
const response = await fetch('https://api.yourdomain.com/api/v1/payments', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer live_abc123...',
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
├─> Customer visits payment link
├─> Sees payment page with:
│   ├─> Amount in crypto and USD
│   ├─> QR code
│   ├─> Wallet address (copy button)
│   ├─> Network information
│   ├─> Countdown timer
│   └─> Status (pending/confirmed/expired)
├─> Customer scans QR or copies address
├─> Sends payment from wallet
└─> Page auto-updates when confirmed
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
├─> Domain: pay.yourdomain.com
├─> Path: /lnk_
└─> ID: abc123xyz (12 characters, unique)
```

**Current Implementation:** ✅ Done

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

**Current Implementation:** ✅ Text-based QR (upgrade to image recommended)

### 6.3 Dynamic QR Codes (Future)

```
Features:
├─> Update amount in real-time
├─> Show payment status in QR
├─> Multi-currency QR (customer chooses)
└─> Branded QR codes
```

---

## 7. SECURITY

### 7.1 API Key Security

```
Current Implementation:
├─> API keys hashed with Argon2 ✅
├─> Keys prefixed: live_ or test_
├─> Key rotation supported ✅
└─> Keys never logged in plaintext ✅
```

**Best Practices:**
```bash
# Store API keys in environment variables
export CRYPTO_GATEWAY_API_KEY="live_abc123..."

# Never commit to git
echo "*.env" >> .gitignore

# Rotate keys regularly
POST /api/v1/merchants/api-keys/rotate
```

### 7.2 Two-Factor Authentication (Future)

```
2FA Flow:
├─> Merchant enables 2FA
├─> Scans QR code with authenticator app
├─> Enters 6-digit code to verify
└─> Required for:
    ├─> Withdrawals
    ├─> API key rotation
    ├─> Wallet address changes
    └─> Sensitive settings
```

### 7.3 IP Whitelisting

```
Current Implementation: ✅
├─> Merchant adds allowed IPs
├─> Supports CIDR ranges
├─> Max 10 entries per merchant
└─> Empty whitelist = allow all
```

**API:**
```bash
PUT /api/v1/merchants/ip-whitelist
Authorization: Bearer live_abc123...
{
  "ip_addresses": [
    "203.0.113.0/24",
    "198.51.100.42"
  ]
}
```

### 7.4 Webhook Signatures

```
Current Implementation: ✅
├─> HMAC-SHA256 signature
├─> Included in X-Signature header
├─> Timestamp in X-Timestamp header
└─> Prevents replay attacks
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
Current Implementation: ✅
├─> 100 requests per minute per API key
├─> Returns 429 when exceeded
└─> Resets every minute
```

### 7.6 Fraud Prevention (Future)

```
Proposed Features:
├─> Velocity checks (max payments per hour)
├─> Blacklist addresses
├─> Risk scoring
├─> Suspicious activity alerts
└─> Manual review queue
```

---

## 8. CURRENCY SUPPORT

### 8.1 Currently Supported

```
✅ Solana (SOL)
✅ USDT on Solana (SPL)
✅ USDT on BSC (BEP20)
✅ USDT on Arbitrum
✅ USDT on Polygon
```

### 8.2 Merchant Currency Toggle (Future Feature)

```
Proposed UI:
┌─────────────────────────────────────┐
│ Accepted Currencies                 │
├─────────────────────────────────────┤
│ [x] SOL (Solana)           1.5% fee │
│ [x] USDT (Solana)          0% fee   │
│ [x] USDT (BSC)             0% fee   │
│ [ ] USDT (Arbitrum)        0% fee   │
│ [x] USDT (Polygon)         0% fee   │
│ [ ] ETH (Ethereum)         1.5% fee │
│ [ ] USDC (Polygon)         0% fee   │
└─────────────────────────────────────┘
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
PUT /api/v1/merchants/currencies
Authorization: Bearer live_abc123...
{
  "accepted_currencies": ["SOL", "USDT_SPL", "USDT_POLYGON"]
}
```

### 8.3 Adding New Currencies

```
Process to Add New Currency:
├─> 1. Add to CryptoType enum
├─> 2. Implement blockchain monitor
├─> 3. Add price fetching
├─> 4. Add address validation
├─> 5. Update documentation
└─> 6. Test on testnet
```

---

## 9. WEBHOOKS & NOTIFICATIONS

### 9.1 Webhook Events

```
Current Events:
├─> payment.confirmed
├─> payment.expired
├─> refund.created
└─> refund.completed
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
Current Implementation: ✅
├─> Retry up to 5 times
├─> Exponential backoff: 1s, 2s, 4s, 8s, 16s
├─> Logs all attempts
└─> Marks as failed after 5 attempts
```

### 9.3 Email Notifications (Future)

```
Proposed Notifications:
├─> To Merchant:
│   ├─> Payment received
│   ├─> Withdrawal completed
│   ├─> API key rotated
│   └─> Security alerts
└─> To Customer:
    ├─> Payment confirmation
    ├─> Receipt
    └─> Refund processed
```

---

## 10. ANALYTICS & REPORTING

### 10.1 Available Analytics

```
Current Implementation: ✅
├─> Total volume (USD)
├─> Payment counts (successful/failed)
├─> Total fees paid
├─> Average transaction value
├─> Breakdown by blockchain
└─> Date range filtering
```

**API:**
```bash
GET /api/v1/analytics?from=2026-01-01&to=2026-01-31
Authorization: Bearer live_abc123...

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
Current Implementation: ✅
GET /api/v1/analytics/export?from=2026-01-01&to=2026-01-31

Returns CSV with:
payment_id, date, amount, crypto_type, status, fee, transaction_hash
```

### 10.3 Advanced Analytics (Future)

```
Proposed Metrics:
├─> Revenue forecasting
├─> Customer lifetime value
├─> Conversion rates
├─> Geographic distribution
├─> Peak transaction times
├─> Currency preferences
└─> Refund rates
```

---

## QUICK REFERENCE

### Essential API Endpoints

```bash
# Authentication
All requests: Authorization: Bearer <api_key>

# Merchant Setup
POST   /api/v1/merchants/register
PUT    /api/v1/merchants/wallets
PUT    /api/v1/merchants/webhook

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

1. ✅ Register account
2. ✅ Configure wallet addresses
3. ✅ Set webhook URL
4. ✅ Test in sandbox mode
5. ✅ Integrate API
6. ✅ Go live!

**Support:** support@yourdomain.com
**Documentation:** https://docs.yourdomain.com
**Status Page:** https://status.yourdomain.com
