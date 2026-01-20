# Design Document: Crypto Payment Gateway Service

## Overview

The Crypto Payment Gateway is a standalone Rust-based service that enables merchants to accept cryptocurrency payments across multiple blockchains. The service transforms the existing trading bot payment functionality into a multi-tenant SaaS platform with merchant management, API authentication, webhook notifications, and fee collection.

### Key Features

- Multi-blockchain support (Solana, BSC, Arbitrum, Polygon)
- RESTful API with authentication and rate limiting
- Real-time webhook notifications with retry logic
- Hosted payment pages with QR codes
- Sandbox testing environment
- Fee-based revenue model
- Comprehensive analytics and reporting

### Technology Stack

- **Language**: Rust (leveraging existing payment codebase)
- **Web Framework**: Axum (async, performant, type-safe)
- **Database**: PostgreSQL with sqlx for migrations
- **Blockchain Integration**: Existing monitors (blockchain_monitor.rs, sol_monitor.rs)
- **Price Data**: Bybit API (existing price_fetcher.rs)
- **Deployment**: Docker containers with docker-compose


## Architecture

### System Architecture

The service follows a layered architecture pattern:

```
┌─────────────────────────────────────────────────────────────┐
│                     External Clients                         │
│  (Merchant APIs, Customer Payment Pages, Webhooks)          │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                   API Gateway Layer                          │
│  - Authentication Middleware (API Key validation)           │
│  - Rate Limiting (100 req/min per merchant)                 │
│  - IP Whitelisting                                           │
│  - Request Logging                                           │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                  Application Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Merchant   │  │   Payment    │  │   Webhook    │      │
│  │   Service    │  │   Service    │  │   Service    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Analytics   │  │    Refund    │  │   Sandbox    │      │
│  │   Service    │  │   Service    │  │   Service    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              Blockchain Integration Layer                    │
│  - Payment Processor (existing processor.rs)                │
│  - Payment Verifier (existing verifier.rs)                  │
│  - Blockchain Monitors (existing monitors)                  │
│  - Price Fetcher (existing price_fetcher.rs)                │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                  Data Layer                                  │
│  - PostgreSQL Database                                       │
│  - Redis Cache (price data, rate limiting)                  │
└──────────────────────────────────────────────────────────────┘
```

### Service Separation Strategy

The gateway will be a **separate repository** from the trading bot with:

1. **Shared Code via Git Submodule or Workspace**:
   - Copy `payment/` module into new repository
   - Maintain as independent codebase
   - Extract reusable blockchain monitoring into shared library (future optimization)

2. **Independent Deployment**:
   - Separate Docker containers
   - Independent scaling
   - Isolated database (separate PostgreSQL instance)

3. **API-First Design**:
   - All functionality exposed via REST API
   - No direct database access from external services
   - Webhook-based event notifications


## Components and Interfaces

### 1. Merchant Service

Manages merchant accounts, API keys, and wallet configurations.

```rust
pub struct MerchantService {
    db_pool: PgPool,
    redis: RedisClient,
}

impl MerchantService {
    /// Register a new merchant
    pub async fn register_merchant(
        &self,
        email: String,
        business_name: String,
    ) -> Result<MerchantRegistrationResponse, ServiceError>;

    /// Generate API key for merchant
    pub async fn generate_api_key(
        &self,
        merchant_id: i64,
    ) -> Result<String, ServiceError>;

    /// Rotate API key (invalidate old, generate new)
    pub async fn rotate_api_key(
        &self,
        merchant_id: i64,
        old_api_key: String,
    ) -> Result<String, ServiceError>;

    /// Validate API key and return merchant
    pub async fn authenticate(
        &self,
        api_key: &str,
    ) -> Result<Merchant, ServiceError>;

    /// Add or update wallet address for blockchain
    pub async fn set_wallet_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        address: String,
    ) -> Result<(), ServiceError>;

    /// Get merchant wallet for crypto type
    pub async fn get_wallet_address(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
    ) -> Result<String, ServiceError>;
}
```

### 2. Payment Service

Core payment creation, verification, and status management.

```rust
pub struct PaymentService {
    db_pool: PgPool,
    processor: PaymentProcessor,  // Reuse existing
    verifier: PaymentVerifier,    // Reuse existing
    price_fetcher: PriceFetcher,  // Reuse existing
    webhook_service: Arc<WebhookService>,
}

impl PaymentService {
    /// Create a new payment request
    pub async fn create_payment(
        &self,
        merchant_id: i64,
        request: CreatePaymentRequest,
    ) -> Result<PaymentResponse, ServiceError>;

    /// Verify payment with transaction hash
    pub async fn verify_payment(
        &self,
        payment_id: String,
        transaction_hash: String,
    ) -> Result<PaymentStatus, ServiceError>;

    /// Get payment status
    pub async fn get_payment(
        &self,
        payment_id: String,
    ) -> Result<PaymentResponse, ServiceError>;

    /// List payments for merchant
    pub async fn list_payments(
        &self,
        merchant_id: i64,
        filters: PaymentFilters,
    ) -> Result<Vec<PaymentResponse>, ServiceError>;

    /// Mark payment as expired (background task)
    async fn expire_payment(&self, payment_id: String) -> Result<(), ServiceError>;
}
```

### 3. Webhook Service

Manages webhook delivery with retry logic and signature verification.

```rust
pub struct WebhookService {
    db_pool: PgPool,
    http_client: Client,
    signing_key: String,
}

impl WebhookService {
    /// Send webhook notification
    pub async fn send_webhook(
        &self,
        merchant_id: i64,
        event: WebhookEvent,
    ) -> Result<(), ServiceError>;

    /// Retry failed webhooks (background task)
    pub async fn retry_failed_webhooks(&self) -> Result<(), ServiceError>;

    /// Generate webhook signature
    fn generate_signature(
        &self,
        payload: &str,
        timestamp: i64,
    ) -> String;

    /// Configure webhook URL for merchant
    pub async fn set_webhook_url(
        &self,
        merchant_id: i64,
        url: String,
    ) -> Result<(), ServiceError>;
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub event_type: String,  // "payment.confirmed", "payment.expired"
    pub payment_id: String,
    pub merchant_id: i64,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub crypto_type: String,
    pub transaction_hash: Option<String>,
    pub timestamp: i64,
}
```

### 4. Analytics Service

Provides transaction analytics and reporting.

```rust
pub struct AnalyticsService {
    db_pool: PgPool,
}

impl AnalyticsService {
    /// Get merchant analytics for date range
    pub async fn get_analytics(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<AnalyticsReport, ServiceError>;

    /// Export analytics as CSV
    pub async fn export_csv(
        &self,
        merchant_id: i64,
        filters: AnalyticsFilters,
    ) -> Result<String, ServiceError>;
}

#[derive(Debug, Serialize)]
pub struct AnalyticsReport {
    pub total_volume_usd: Decimal,
    pub successful_payments: i64,
    pub failed_payments: i64,
    pub total_fees_paid: Decimal,
    pub average_transaction_value: Decimal,
    pub by_blockchain: HashMap<String, BlockchainStats>,
}
```

### 5. Refund Service

Handles payment refunds and tracking.

```rust
pub struct RefundService {
    db_pool: PgPool,
    webhook_service: Arc<WebhookService>,
}

impl RefundService {
    /// Create a refund for a payment
    pub async fn create_refund(
        &self,
        merchant_id: i64,
        payment_id: String,
        amount: Option<Decimal>,  // None = full refund
        reason: String,
    ) -> Result<RefundResponse, ServiceError>;

    /// Update refund with transaction hash
    pub async fn complete_refund(
        &self,
        refund_id: String,
        transaction_hash: String,
    ) -> Result<(), ServiceError>;

    /// Get refund status
    pub async fn get_refund(
        &self,
        refund_id: String,
    ) -> Result<RefundResponse, ServiceError>;
}
```

### 6. Sandbox Service

Provides testing environment for merchant integration.

```rust
pub struct SandboxService {
    db_pool: PgPool,
    webhook_service: Arc<WebhookService>,
}

impl SandboxService {
    /// Create sandbox API credentials
    pub async fn create_sandbox_credentials(
        &self,
        merchant_id: i64,
    ) -> Result<SandboxCredentials, ServiceError>;

    /// Simulate payment confirmation (sandbox only)
    pub async fn simulate_confirmation(
        &self,
        payment_id: String,
        success: bool,
    ) -> Result<(), ServiceError>;

    /// Check if API key is sandbox
    pub fn is_sandbox_key(&self, api_key: &str) -> bool;
}
```


## Data Models

### Database Schema Extensions

Building on the existing payment tables, we add merchant-specific tables:

```sql
-- Merchants table
CREATE TABLE merchants (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    business_name VARCHAR(255) NOT NULL,
    api_key_hash VARCHAR(255) UNIQUE NOT NULL,
    fee_percentage DECIMAL(5,2) NOT NULL DEFAULT 1.50,  -- 1.5% default
    is_active BOOLEAN NOT NULL DEFAULT true,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_merchants_api_key ON merchants(api_key_hash);
CREATE INDEX idx_merchants_email ON merchants(email);

-- Merchant wallets (one per blockchain)
CREATE TABLE merchant_wallets (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,  -- "SOL", "USDT_BEP20", etc.
    network VARCHAR(50) NOT NULL,      -- "SOLANA", "BEP20", etc.
    address VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, crypto_type)
);

CREATE INDEX idx_merchant_wallets_merchant ON merchant_wallets(merchant_id);

-- Webhook configurations
CREATE TABLE webhook_configs (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    url VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id)
);

-- Webhook delivery log
CREATE TABLE webhook_deliveries (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,  -- "payment.confirmed", "payment.expired"
    url VARCHAR(500) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,  -- "pending", "delivered", "failed"
    attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_retry_at TIMESTAMPTZ,
    response_status INT,
    response_body TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_deliveries_merchant ON webhook_deliveries(merchant_id);
CREATE INDEX idx_webhook_deliveries_payment ON webhook_deliveries(payment_id);
CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at) 
    WHERE status = 'pending';

-- Refunds table
CREATE TABLE refunds (
    id BIGSERIAL PRIMARY KEY,
    refund_id VARCHAR(100) UNIQUE NOT NULL,  -- Public-facing ID
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    amount DECIMAL(20,8) NOT NULL,
    amount_usd DECIMAL(20,2) NOT NULL,
    reason TEXT,
    status VARCHAR(50) NOT NULL,  -- "pending", "completed", "failed"
    transaction_hash VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_refunds_merchant ON refunds(merchant_id);
CREATE INDEX idx_refunds_payment ON refunds(payment_id);
CREATE INDEX idx_refunds_status ON refunds(status);

-- IP whitelist
CREATE TABLE ip_whitelist (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    ip_address VARCHAR(45) NOT NULL,  -- Supports IPv6
    cidr_range VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ip_whitelist_merchant ON ip_whitelist(merchant_id);

-- Audit log
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT REFERENCES merchants(id) ON DELETE SET NULL,
    action_type VARCHAR(100) NOT NULL,  -- "merchant.created", "api_key.rotated", etc.
    entity_type VARCHAR(50),  -- "merchant", "payment", "refund"
    entity_id VARCHAR(100),
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_merchant ON audit_logs(merchant_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_action ON audit_logs(action_type);

-- Payment links (for hosted pages)
CREATE TABLE payment_links (
    id BIGSERIAL PRIMARY KEY,
    link_id VARCHAR(100) UNIQUE NOT NULL,  -- Short unique ID for URL
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_payment_links_link_id ON payment_links(link_id);
CREATE INDEX idx_payment_links_payment ON payment_links(payment_id);

-- Partial payments tracking
CREATE TABLE partial_payments (
    id BIGSERIAL PRIMARY KEY,
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    transaction_hash VARCHAR(255) NOT NULL,
    amount DECIMAL(20,8) NOT NULL,
    amount_usd DECIMAL(20,2) NOT NULL,
    confirmations INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL,  -- "pending", "confirmed"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

CREATE INDEX idx_partial_payments_payment ON partial_payments(payment_id);
CREATE INDEX idx_partial_payments_status ON partial_payments(status);
```

### Extended Payment Transaction Model

Extend the existing `payment_transactions` table:

```sql
ALTER TABLE payment_transactions 
ADD COLUMN merchant_id BIGINT REFERENCES merchants(id) ON DELETE CASCADE,
ADD COLUMN payment_id VARCHAR(100) UNIQUE,  -- Public-facing ID (e.g., "pay_abc123")
ADD COLUMN description TEXT,
ADD COLUMN metadata JSONB,
ADD COLUMN fee_percentage DECIMAL(5,2),
ADD COLUMN fee_amount DECIMAL(20,8),
ADD COLUMN fee_amount_usd DECIMAL(20,2),
ADD COLUMN partial_payments_enabled BOOLEAN DEFAULT false,
ADD COLUMN total_paid DECIMAL(20,8) DEFAULT 0,
ADD COLUMN remaining_balance DECIMAL(20,8);

CREATE INDEX idx_payment_transactions_merchant ON payment_transactions(merchant_id);
CREATE INDEX idx_payment_transactions_payment_id ON payment_transactions(payment_id);
```

### Rust Data Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Merchant {
    pub id: i64,
    pub email: String,
    pub business_name: String,
    pub api_key_hash: String,
    pub fee_percentage: Decimal,
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount_usd: Decimal,
    pub crypto_type: CryptoType,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expiration_minutes: Option<u32>,  // Default: 15
    pub partial_payments_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub payment_id: String,  // "pay_abc123"
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub crypto_type: CryptoType,
    pub network: String,
    pub deposit_address: String,
    pub payment_link: String,  // URL to hosted page
    pub qr_code_data: String,  // Data for QR code generation
    pub fee_amount: Decimal,
    pub fee_amount_usd: Decimal,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub transaction_hash: Option<String>,
    pub partial_payments: Option<PartialPaymentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentInfo {
    pub enabled: bool,
    pub total_paid: Decimal,
    pub remaining_balance: Decimal,
    pub payments: Vec<PartialPaymentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub payment_id: String,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub status: String,
    pub reason: Option<String>,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```


## API Endpoints

### Authentication

All API requests require an API key in the `Authorization` header:
```
Authorization: Bearer <api_key>
```

### Merchant Management

```
POST /api/v1/merchants/register
- Register a new merchant account
- Body: { email, business_name }
- Returns: { merchant_id, api_key }

POST /api/v1/merchants/api-keys/rotate
- Rotate API key
- Returns: { api_key }

PUT /api/v1/merchants/wallets
- Set wallet address for blockchain
- Body: { crypto_type, address }
- Returns: { success }

GET /api/v1/merchants/wallets
- Get all configured wallets
- Returns: { wallets: [{ crypto_type, address }] }

PUT /api/v1/merchants/webhook
- Configure webhook URL
- Body: { url }
- Returns: { success }

PUT /api/v1/merchants/ip-whitelist
- Configure IP whitelist
- Body: { ip_addresses: ["1.2.3.4", "10.0.0.0/24"] }
- Returns: { success }
```

### Payment Operations

```
POST /api/v1/payments
- Create a new payment
- Body: CreatePaymentRequest
- Returns: PaymentResponse

GET /api/v1/payments/:payment_id
- Get payment details
- Returns: PaymentResponse

POST /api/v1/payments/:payment_id/verify
- Verify payment with transaction hash
- Body: { transaction_hash }
- Returns: PaymentResponse

GET /api/v1/payments
- List payments with filters
- Query: ?status=pending&from=2024-01-01&to=2024-12-31&blockchain=solana
- Returns: { payments: [PaymentResponse], total, page }
```

### Refund Operations

```
POST /api/v1/refunds
- Create a refund
- Body: { payment_id, amount?, reason }
- Returns: RefundResponse

GET /api/v1/refunds/:refund_id
- Get refund details
- Returns: RefundResponse

POST /api/v1/refunds/:refund_id/complete
- Mark refund as completed with transaction hash
- Body: { transaction_hash }
- Returns: RefundResponse
```

### Analytics

```
GET /api/v1/analytics
- Get analytics for date range
- Query: ?from=2024-01-01&to=2024-12-31&blockchain=solana
- Returns: AnalyticsReport

GET /api/v1/analytics/export
- Export analytics as CSV
- Query: ?from=2024-01-01&to=2024-12-31
- Returns: CSV file
```

### Sandbox Operations

```
POST /api/v1/sandbox/enable
- Enable sandbox mode and get test credentials
- Returns: { sandbox_api_key }

POST /api/v1/sandbox/payments/:payment_id/simulate
- Simulate payment confirmation (sandbox only)
- Body: { success: true }
- Returns: PaymentResponse
```

### Hosted Payment Pages

```
GET /pay/:link_id
- Display hosted payment page
- Returns: HTML page with payment details and QR code
```

### Webhooks (Merchant Endpoint)

Merchants configure their webhook URL to receive notifications:

```
POST <merchant_webhook_url>
Headers:
  X-Signature: <hmac_signature>
  X-Timestamp: <unix_timestamp>
Body: WebhookPayload
```


## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property Reflection

After analyzing all acceptance criteria, I identified the following redundancies to eliminate:

- **Redundancy 1**: Requirements 2.4 and 2.7 both test payment expiration. These can be combined into a single property about expiration behavior.
- **Redundancy 2**: Requirements 4.2 and 4.3 both test webhook delivery on status changes. These can be combined into a single property about webhook delivery for any status change.
- **Redundancy 3**: Requirements 7.5 and 7.6 both test API key rotation. These can be combined into a single property.
- **Redundancy 4**: Requirements 5.6 and 5.7 both test payment page status display. These can be combined into a single property.
- **Redundancy 5**: Multiple requirements test validation (1.4, 1.6, 4.1, 18.1) but for different entities. These should remain separate as they validate different data types.
- **Redundancy 6**: Requirements 12.2 and 12.6 both involve displaying crypto and fiat amounts. 12.6 is more specific about display, so we'll keep both but note they're related.

### Core Properties

#### Property 1: Unique Merchant Identifiers
*For any* set of merchant registrations, all created merchant IDs should be unique and no two merchants should have the same ID.
**Validates: Requirements 1.1**

#### Property 2: Unique API Key Generation
*For any* set of merchants, all generated API keys should be unique across the entire system.
**Validates: Requirements 1.2**

#### Property 3: Wallet Address Validation
*For any* wallet address submission, if the address is invalid for the specified blockchain, the Gateway should reject it; if valid, it should be stored successfully.
**Validates: Requirements 1.4, 1.6**

#### Property 4: One Wallet Per Blockchain
*For any* merchant and blockchain type, the merchant should have at most one active wallet address for that blockchain type.
**Validates: Requirements 1.5**

#### Property 5: Unique Payment Identifiers
*For any* set of payment creations, all generated payment IDs should be unique across the entire system.
**Validates: Requirements 2.1**

#### Property 6: Crypto Amount Calculation
*For any* payment creation with USD amount A and exchange rate R, the calculated crypto amount should equal A / R (for non-stablecoins) or A (for stablecoins).
**Validates: Requirements 2.2**

#### Property 7: Payment Expiration Transition
*For any* payment with expiration time T, when current time exceeds T and the payment is not completed, the payment status should transition to expired.
**Validates: Requirements 2.4, 2.7**

#### Property 8: Fee Inclusion in Total
*For any* payment with base amount B and fee percentage F, the total amount should equal B + (B * F).
**Validates: Requirements 2.6, 6.1**

#### Property 9: Transaction Hash Uniqueness
*For any* transaction hash submitted for verification, if that hash has already been used for another payment, the verification should be rejected.
**Validates: Requirements 3.1, 3.5**

#### Property 10: Amount Verification
*For any* transaction verification, if the blockchain transaction amount differs from the expected payment amount by more than 0.1%, the verification should be rejected.
**Validates: Requirements 3.2, 3.5**

#### Property 11: Address Verification
*For any* transaction verification, if the recipient address does not match the merchant's wallet address for that blockchain, the verification should be rejected.
**Validates: Requirements 3.3, 3.5**

#### Property 12: Confirmation Threshold
*For any* payment with required confirmations R, when the transaction receives C confirmations where C >= R, the payment status should transition to confirmed.
**Validates: Requirements 3.4, 3.7**

#### Property 13: Webhook URL Validation
*For any* webhook URL configuration, if the URL is not a valid HTTPS endpoint, it should be rejected.
**Validates: Requirements 4.1**

#### Property 14: Webhook Delivery on Status Change
*For any* payment status change to confirmed or expired, a webhook notification should be sent to the merchant's configured webhook URL.
**Validates: Requirements 4.2, 4.3**

#### Property 15: Webhook Retry Logic
*For any* failed webhook delivery, the system should retry up to 5 times with exponential backoff between attempts.
**Validates: Requirements 4.4**

#### Property 16: Webhook Signature Inclusion
*For any* webhook sent, the request should include a valid HMAC signature in the X-Signature header that can be verified using the signing key.
**Validates: Requirements 4.5**

#### Property 17: Webhook Payload Completeness
*For any* webhook notification, the payload should include payment_id, status, amount, crypto_type, and timestamp fields.
**Validates: Requirements 4.6**

#### Property 18: Webhook Delivery Logging
*For any* webhook delivery attempt, a record should be created in the webhook_deliveries table with attempt count, status, and timestamp.
**Validates: Requirements 4.7**

#### Property 19: Unique Payment Links
*For any* set of payments, all generated payment link IDs should be unique across the system.
**Validates: Requirements 5.1**

#### Property 20: Payment Page Content
*For any* payment page request, the response should include payment amount in both crypto and fiat, QR code data, and time remaining until expiration.
**Validates: Requirements 5.2, 5.3, 5.4, 5.5**

#### Property 21: Payment Page Status Display
*For any* payment page request, if the payment is completed the page should show success status, if expired it should show expired status.
**Validates: Requirements 5.6, 5.7**

#### Property 22: Fee Percentage Bounds
*For any* merchant fee configuration, if the fee percentage is less than 0.1% or greater than 5%, it should be rejected.
**Validates: Requirements 6.4**

#### Property 23: Fee Recording on Confirmation
*For any* confirmed payment, the fee_amount and fee_amount_usd fields should be populated with the calculated fee values.
**Validates: Requirements 6.3**

#### Property 24: Fee Calculation Timing
*For any* payment, the fee should be calculated using the fiat value at payment creation time, not at confirmation time.
**Validates: Requirements 6.6**

#### Property 25: API Key Authentication
*For any* API request with a valid API key, the merchant should be authenticated successfully; for any request with invalid or missing API key, it should be rejected with 401 status.
**Validates: Requirements 7.1, 7.2**

#### Property 26: Rate Limiting
*For any* API key, when more than 100 requests are made within a 60-second window, subsequent requests should be rejected with 429 status until the window resets.
**Validates: Requirements 7.3, 7.4**

#### Property 27: API Key Rotation
*For any* API key rotation, the old key should become invalid and the new key should be valid for authentication.
**Validates: Requirements 7.5, 7.6**

#### Property 28: API Request Logging
*For any* API request, a log entry should be created with timestamp, endpoint, merchant_id, and response status.
**Validates: Requirements 7.7**

#### Property 29: Blockchain Fault Isolation
*For any* blockchain network failure, the monitoring and verification of payments on other blockchain networks should continue unaffected.
**Validates: Requirements 8.7**

#### Property 30: Refund Amount Validation
*For any* refund creation, if the refund amount exceeds the original payment amount, the refund should be rejected.
**Validates: Requirements 9.3**

#### Property 31: Refund Webhook Notification
*For any* refund status change, a webhook notification should be sent to the merchant's configured webhook URL.
**Validates: Requirements 9.5**

#### Property 32: Balance Calculation with Refunds
*For any* merchant balance calculation, the total should equal sum of confirmed payments minus sum of completed refunds.
**Validates: Requirements 9.7**

#### Property 33: Sandbox Data Isolation
*For any* sandbox API key, requests should not be able to access or modify production payment data.
**Validates: Requirements 10.6**

#### Property 34: Sandbox Payment Marking
*For any* payment created with a sandbox API key, the response should clearly indicate it is a sandbox payment.
**Validates: Requirements 10.4**

#### Property 35: Analytics Volume Calculation
*For any* analytics request with date range [start, end], the total volume should equal the sum of all confirmed payment amounts where created_at is between start and end.
**Validates: Requirements 11.1**

#### Property 36: Analytics Report Completeness
*For any* analytics report, it should include successful_payment_count, failed_payment_count, total_fees_paid, and average_transaction_value fields.
**Validates: Requirements 11.2**

#### Property 37: Average Transaction Calculation
*For any* analytics report with N successful payments totaling amount T, the average transaction value should equal T / N.
**Validates: Requirements 11.5**

#### Property 38: Price Cache Duration
*For any* price fetch, if a cached price exists and is less than 30 seconds old, the cached price should be used instead of fetching a new price.
**Validates: Requirements 12.3**

#### Property 39: Price API Fallback
*For any* price fetch when the Bybit API is unavailable, the last cached price should be used and a warning should be logged.
**Validates: Requirements 12.4**

#### Property 40: Blockchain Query Retry
*For any* blockchain query failure, the system should retry up to 3 times with exponential backoff before returning an error.
**Validates: Requirements 13.1**

#### Property 41: Circuit Breaker Behavior
*For any* external API with a circuit breaker, after N consecutive failures (where N is the threshold), the circuit should open and subsequent calls should use cached data or return degraded responses.
**Validates: Requirements 13.4, 13.5**

#### Property 42: Error Logging Context
*For any* error that occurs, the log entry should include merchant_id (if applicable), payment_id (if applicable), error message, and stack trace.
**Validates: Requirements 13.6**

#### Property 43: Migration Versioning
*For any* database migration file, it should include a timestamp, version number, and description in its metadata.
**Validates: Requirements 15.4**

#### Property 44: Schema Validation on Startup
*For any* application startup, if the database schema version does not match the expected version, the application should halt and log an error.
**Validates: Requirements 15.6**

#### Property 45: Migration Failure Handling
*For any* migration that fails during application startup, the application should halt and log the migration error.
**Validates: Requirements 15.7**

#### Property 46: IP Address Validation
*For any* IP whitelist entry, if the entry is not a valid IP address or CIDR range, it should be rejected.
**Validates: Requirements 18.1**

#### Property 47: IP Whitelist Enforcement
*For any* API request when IP whitelisting is enabled, if the request IP is not in the whitelist, it should be rejected with 403 status.
**Validates: Requirements 18.2, 18.3**

#### Property 48: IP Whitelist Limit
*For any* merchant, if they attempt to add more than 10 IP whitelist entries, the 11th entry should be rejected.
**Validates: Requirements 18.4**

#### Property 49: Empty Whitelist Behavior
*For any* merchant with an empty IP whitelist, all IP addresses should be allowed to make requests.
**Validates: Requirements 18.7**

#### Property 50: Audit Log Creation
*For any* merchant account change, API key rotation, payment creation, payment confirmation, or refund creation, an audit log entry should be created with merchant_id, action_type, timestamp, and IP address.
**Validates: Requirements 19.1, 19.2, 19.3, 19.4**

#### Property 51: Partial Payment Tracking
*For any* payment with partial payments enabled, the total_paid field should equal the sum of all confirmed partial payment amounts.
**Validates: Requirements 20.2**

#### Property 52: Partial Payment Balance
*For any* payment with partial payments enabled, the remaining_balance should equal the required amount minus total_paid.
**Validates: Requirements 20.3**

#### Property 53: Partial Payment Completion
*For any* payment with partial payments enabled, when total_paid >= required amount, the payment status should transition to completed.
**Validates: Requirements 20.4**

#### Property 54: Partial Payment Webhooks
*For any* confirmed partial payment, a webhook notification should be sent to the merchant.
**Validates: Requirements 20.5**

#### Property 55: Partial Payment Expiration Extension
*For any* payment with partial payments enabled, when a partial payment is confirmed, the expiration time should be extended by the original expiration duration.
**Validates: Requirements 20.7**


## Error Handling

### Error Response Format

All API errors follow a consistent JSON format:

```rust
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,           // Machine-readable error code
    pub message: String,        // Human-readable message
    pub details: Option<serde_json::Value>,  // Additional context
    pub request_id: String,     // For tracking and support
}
```

### Error Categories

#### 1. Authentication Errors (401)
- `INVALID_API_KEY`: API key is invalid or expired
- `MISSING_API_KEY`: Authorization header is missing
- `SANDBOX_PRODUCTION_MISMATCH`: Sandbox key used for production endpoint or vice versa

#### 2. Authorization Errors (403)
- `IP_NOT_WHITELISTED`: Request IP not in merchant's whitelist
- `INSUFFICIENT_PERMISSIONS`: Merchant lacks permission for this operation

#### 3. Validation Errors (400)
- `INVALID_WALLET_ADDRESS`: Wallet address format is invalid for blockchain
- `INVALID_AMOUNT`: Payment amount is negative or zero
- `INVALID_FEE_PERCENTAGE`: Fee percentage outside allowed range (0.1% - 5%)
- `INVALID_WEBHOOK_URL`: Webhook URL is not HTTPS or malformed
- `INVALID_IP_ADDRESS`: IP whitelist entry is not valid IP or CIDR
- `REFUND_EXCEEDS_PAYMENT`: Refund amount exceeds original payment
- `DUPLICATE_TRANSACTION_HASH`: Transaction hash already used for another payment

#### 4. Resource Errors (404)
- `PAYMENT_NOT_FOUND`: Payment ID does not exist
- `MERCHANT_NOT_FOUND`: Merchant ID does not exist
- `REFUND_NOT_FOUND`: Refund ID does not exist

#### 5. Rate Limiting (429)
- `RATE_LIMIT_EXCEEDED`: Too many requests in time window

#### 6. Payment Processing Errors (422)
- `PAYMENT_EXPIRED`: Payment has expired and cannot be verified
- `PAYMENT_ALREADY_CONFIRMED`: Payment is already confirmed
- `TRANSACTION_VERIFICATION_FAILED`: Blockchain transaction does not match payment
- `INSUFFICIENT_CONFIRMATIONS`: Transaction needs more confirmations
- `BLOCKCHAIN_NETWORK_ERROR`: Unable to query blockchain network

#### 7. Server Errors (500)
- `INTERNAL_SERVER_ERROR`: Unexpected server error
- `DATABASE_ERROR`: Database operation failed
- `EXTERNAL_API_ERROR`: External API (Bybit, blockchain RPC) failed

### Retry Strategy

#### Blockchain Queries
```rust
async fn query_blockchain_with_retry<T, F>(
    operation: F,
    max_retries: u32,
) -> Result<T, ServiceError>
where
    F: Fn() -> Future<Output = Result<T, BlockchainError>>,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                warn!("Blockchain query failed (attempt {}): {}", attempt + 1, e);
                tokio::time::sleep(delay).await;
                delay *= 2;  // Exponential backoff
                attempt += 1;
            }
            Err(e) => return Err(ServiceError::BlockchainError(e)),
        }
    }
}
```

#### Webhook Delivery
```rust
async fn deliver_webhook_with_retry(
    webhook_id: i64,
    url: &str,
    payload: &WebhookPayload,
) -> Result<(), ServiceError> {
    let max_attempts = 5;
    let mut delay = Duration::from_secs(1);
    
    for attempt in 1..=max_attempts {
        match send_webhook(url, payload).await {
            Ok(_) => {
                update_webhook_status(webhook_id, "delivered").await?;
                return Ok(());
            }
            Err(e) => {
                warn!("Webhook delivery failed (attempt {}): {}", attempt, e);
                update_webhook_attempt(webhook_id, attempt, &e).await?;
                
                if attempt < max_attempts {
                    tokio::time::sleep(delay).await;
                    delay *= 2;  // Exponential backoff
                } else {
                    update_webhook_status(webhook_id, "failed").await?;
                    return Err(ServiceError::WebhookDeliveryFailed);
                }
            }
        }
    }
    
    unreachable!()
}
```

### Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    timeout: Duration,
    state: Arc<Mutex<CircuitState>>,
}

enum CircuitState {
    Closed { failures: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<T, F>(&self, operation: F) -> Result<T, ServiceError>
    where
        F: Future<Output = Result<T, ServiceError>>,
    {
        let mut state = self.state.lock().await;
        
        match *state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err(ServiceError::CircuitBreakerOpen);
                }
            }
            _ => {}
        }
        
        drop(state);
        
        match operation.await {
            Ok(result) => {
                let mut state = self.state.lock().await;
                *state = CircuitState::Closed { failures: 0 };
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.lock().await;
                match *state {
                    CircuitState::Closed { failures } => {
                        let new_failures = failures + 1;
                        if new_failures >= self.failure_threshold {
                            *state = CircuitState::Open {
                                opened_at: Instant::now(),
                            };
                        } else {
                            *state = CircuitState::Closed {
                                failures: new_failures,
                            };
                        }
                    }
                    CircuitState::HalfOpen => {
                        *state = CircuitState::Open {
                            opened_at: Instant::now(),
                        };
                    }
                    _ => {}
                }
                Err(e)
            }
        }
    }
}
```

### Graceful Degradation

When external services fail, the gateway provides degraded but functional service:

1. **Price API Failure**: Use last cached price (up to 5 minutes old) with warning
2. **Blockchain RPC Failure**: Queue verification for retry, return "pending" status
3. **Webhook Delivery Failure**: Queue for retry, don't block payment confirmation
4. **Database Connection Pool Exhaustion**: Return 503 with retry-after header


## Testing Strategy

### Dual Testing Approach

The testing strategy employs both unit tests and property-based tests as complementary approaches:

- **Unit Tests**: Verify specific examples, edge cases, error conditions, and integration points
- **Property Tests**: Verify universal properties across all inputs through randomization

Both are necessary for comprehensive coverage. Unit tests catch concrete bugs in specific scenarios, while property tests verify general correctness across a wide input space.

### Property-Based Testing

#### Library Selection

For Rust, we will use **proptest** - a mature property-based testing library with excellent ergonomics and shrinking capabilities.

```toml
[dev-dependencies]
proptest = "1.4"
```

#### Configuration

Each property test must:
- Run minimum **100 iterations** (due to randomization)
- Include a comment tag referencing the design property
- Tag format: `// Feature: crypto-payment-gateway, Property N: <property_text>`

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        // Feature: crypto-payment-gateway, Property 1: Unique Merchant Identifiers
        #[test]
        fn test_unique_merchant_ids(
            registrations in prop::collection::vec(
                (any::<String>(), any::<String>()),
                1..20
            )
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let service = setup_test_service().await;
                let mut ids = Vec::new();
                
                for (email, business_name) in registrations {
                    let result = service.register_merchant(email, business_name).await;
                    if let Ok(response) = result {
                        ids.push(response.merchant_id);
                    }
                }
                
                // All IDs should be unique
                let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
                prop_assert_eq!(ids.len(), unique_ids.len());
            });
        }
    }
}
```

### Unit Testing Strategy

Unit tests should focus on:

1. **Specific Examples**: Concrete test cases that demonstrate correct behavior
   ```rust
   #[tokio::test]
   async fn test_create_payment_with_usdt_bsc() {
       let service = setup_test_service().await;
       let request = CreatePaymentRequest {
           amount_usd: Decimal::from(100),
           crypto_type: CryptoType::UsdtBep20,
           description: Some("Test payment".to_string()),
           metadata: None,
           expiration_minutes: Some(15),
           partial_payments_enabled: Some(false),
       };
       
       let response = service.create_payment(1, request).await.unwrap();
       assert_eq!(response.amount_usd, Decimal::from(100));
       assert_eq!(response.crypto_type, CryptoType::UsdtBep20);
   }
   ```

2. **Edge Cases**: Boundary conditions and special inputs
   ```rust
   #[tokio::test]
   async fn test_refund_exactly_equals_payment() {
       // Test refund amount exactly equal to payment amount
   }
   
   #[tokio::test]
   async fn test_payment_expires_at_exact_expiration_time() {
       // Test expiration at exact boundary
   }
   ```

3. **Error Conditions**: Invalid inputs and failure scenarios
   ```rust
   #[tokio::test]
   async fn test_invalid_wallet_address_rejected() {
       let service = setup_test_service().await;
       let result = service.set_wallet_address(
           1,
           CryptoType::UsdtBep20,
           "invalid_address".to_string()
       ).await;
       
       assert!(result.is_err());
       assert_eq!(result.unwrap_err().code(), "INVALID_WALLET_ADDRESS");
   }
   ```

4. **Integration Points**: Component interactions
   ```rust
   #[tokio::test]
   async fn test_payment_confirmation_triggers_webhook() {
       // Test that confirming a payment sends webhook
   }
   ```

### Test Organization

```
tests/
├── unit/
│   ├── merchant_service_tests.rs
│   ├── payment_service_tests.rs
│   ├── webhook_service_tests.rs
│   ├── analytics_service_tests.rs
│   └── refund_service_tests.rs
├── property/
│   ├── merchant_properties.rs
│   ├── payment_properties.rs
│   ├── webhook_properties.rs
│   ├── fee_properties.rs
│   └── partial_payment_properties.rs
├── integration/
│   ├── api_tests.rs
│   ├── webhook_delivery_tests.rs
│   └── blockchain_integration_tests.rs
└── common/
    ├── fixtures.rs
    └── test_helpers.rs
```

### Test Data Generators

For property-based testing, we need generators for domain types:

```rust
// Arbitrary implementations for proptest
impl Arbitrary for CryptoType {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(CryptoType::Sol),
            Just(CryptoType::UsdtSpl),
            Just(CryptoType::UsdtBep20),
            Just(CryptoType::UsdtArbitrum),
            Just(CryptoType::UsdtPolygon),
        ].boxed()
    }
}

fn valid_wallet_address(crypto_type: CryptoType) -> impl Strategy<Value = String> {
    match crypto_type {
        CryptoType::Sol | CryptoType::UsdtSpl => {
            // Solana addresses are base58, 32-44 chars
            "[1-9A-HJ-NP-Za-km-z]{32,44}".prop_map(|s| s)
        }
        _ => {
            // EVM addresses are 0x + 40 hex chars
            "0x[0-9a-fA-F]{40}".prop_map(|s| s)
        }
    }
}

fn payment_amount_usd() -> impl Strategy<Value = Decimal> {
    (1u64..1_000_000u64).prop_map(|n| Decimal::from(n))
}
```

### Mocking External Dependencies

For testing, we mock external dependencies:

```rust
#[async_trait]
trait BlockchainClient: Send + Sync {
    async fn get_transaction(&self, hash: &str) -> Result<BlockchainTransaction, Error>;
}

struct MockBlockchainClient {
    responses: HashMap<String, BlockchainTransaction>,
}

#[async_trait]
impl BlockchainClient for MockBlockchainClient {
    async fn get_transaction(&self, hash: &str) -> Result<BlockchainTransaction, Error> {
        self.responses.get(hash)
            .cloned()
            .ok_or(Error::TransactionNotFound)
    }
}
```

### Test Database

Use a separate test database with automatic cleanup:

```rust
async fn setup_test_db() -> PgPool {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/gateway_test".to_string());
    
    let pool = PgPool::connect(&db_url).await.unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
    
    pool
}

async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE merchants, payment_transactions, webhook_deliveries, refunds CASCADE")
        .execute(pool)
        .await
        .unwrap();
}
```

### Coverage Goals

- **Unit Test Coverage**: Minimum 80% line coverage
- **Property Test Coverage**: All 55 correctness properties implemented
- **Integration Test Coverage**: All API endpoints tested
- **Edge Case Coverage**: All boundary conditions tested

### Continuous Integration

Tests run on every commit:
```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run property tests
        run: cargo test --test property_tests -- --test-threads=1
```


## Deployment Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgres://user:pass@localhost/crypto_gateway
DATABASE_MAX_CONNECTIONS=20

# Redis (for caching and rate limiting)
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_WORKERS=4

# Blockchain RPC URLs
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
BSC_RPC_URL=https://bsc-dataseed.binance.org
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc
POLYGON_RPC_URL=https://polygon-rpc.com

# Blockchain API Keys (optional, for higher rate limits)
BSCSCAN_API_KEY=your_key_here
ARBISCAN_API_KEY=your_key_here
POLYGONSCAN_API_KEY=your_key_here

# Price API
BYBIT_PRICE_API_URL=https://api.bybit.com
PRICE_CACHE_TTL_SECONDS=30

# Webhook
WEBHOOK_SIGNING_KEY=your_secret_signing_key_here
WEBHOOK_TIMEOUT_SECONDS=10
WEBHOOK_MAX_RETRIES=5

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_BURST=20

# Circuit Breaker
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
CIRCUIT_BREAKER_TIMEOUT_SECONDS=60

# Monitoring
RUST_LOG=info,crypto_gateway=debug
METRICS_PORT=9090

# Security
API_KEY_LENGTH=32
SESSION_SECRET=your_session_secret_here

# Payment Defaults
DEFAULT_PAYMENT_EXPIRATION_MINUTES=15
DEFAULT_FEE_PERCENTAGE=1.50

# Hosted Pages
PAYMENT_PAGE_BASE_URL=https://pay.yourdomain.com
```

### Docker Configuration

#### Dockerfile
```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/crypto-gateway /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080 9090

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["crypto-gateway"]
```

#### docker-compose.yml
```yaml
version: '3.8'

services:
  gateway:
    build: .
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      DATABASE_URL: postgres://gateway:gateway@postgres:5432/crypto_gateway
      REDIS_URL: redis://redis:6379
      RUST_LOG: info,crypto_gateway=debug
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: gateway
      POSTGRES_PASSWORD: gateway
      POSTGRES_DB: crypto_gateway
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U gateway"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

  # Optional: Prometheus for metrics
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
```

### Kubernetes Deployment (Optional)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: crypto-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: crypto-gateway
  template:
    metadata:
      labels:
        app: crypto-gateway
    spec:
      containers:
      - name: gateway
        image: crypto-gateway:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: gateway-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: gateway-secrets
              key: redis-url
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: crypto-gateway
spec:
  selector:
    app: crypto-gateway
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: LoadBalancer
```

### Migration Strategy

1. **Initial Setup**: Deploy with empty database, migrations run automatically
2. **Updates**: New migrations applied on container startup
3. **Rollback**: Use migration tool's down migrations for last 5 versions
4. **Zero-Downtime**: Use blue-green deployment with backward-compatible migrations

### Monitoring and Observability

#### Health Check Endpoints

```rust
// GET /health - Liveness probe
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

// GET /health/ready - Readiness probe
async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    // Check database connection
    if state.db_pool.acquire().await.is_err() {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "status": "not_ready",
            "reason": "database_unavailable"
        })));
    }
    
    // Check Redis connection
    if state.redis.ping().await.is_err() {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "status": "not_ready",
            "reason": "redis_unavailable"
        })));
    }
    
    (StatusCode::OK, Json(json!({
        "status": "ready"
    })))
}
```

#### Prometheus Metrics

```rust
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub payments_created: Counter,
    pub payments_confirmed: Counter,
    pub webhook_deliveries: Counter,
    pub webhook_failures: Counter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Self {
        let requests_total = Counter::new(
            "http_requests_total",
            "Total HTTP requests"
        ).unwrap();
        
        let request_duration = Histogram::new(
            "http_request_duration_seconds",
            "HTTP request duration"
        ).unwrap();
        
        // Register all metrics...
        registry.register(Box::new(requests_total.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        
        Self {
            requests_total,
            request_duration,
            // ... other metrics
        }
    }
}
```

### Security Considerations

1. **API Keys**: Stored as bcrypt hashes in database
2. **HTTPS Only**: All production endpoints require TLS
3. **Rate Limiting**: Per-merchant and global rate limits
4. **IP Whitelisting**: Optional per-merchant IP restrictions
5. **Webhook Signatures**: HMAC-SHA256 signatures for webhook authenticity
6. **SQL Injection**: Prevented by sqlx parameterized queries
7. **Input Validation**: All inputs validated before processing
8. **Audit Logging**: All sensitive operations logged with IP and timestamp

