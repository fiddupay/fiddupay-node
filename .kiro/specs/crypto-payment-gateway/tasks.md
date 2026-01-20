# Implementation Plan: Crypto Payment Gateway Service

## Overview

This implementation plan transforms the existing trading bot payment functionality into a standalone multi-tenant crypto payment gateway service. The implementation will leverage existing Rust code from `trading-bot/src/payment/` and build a new service with merchant management, API authentication, webhooks, and fee collection.

## Tasks

- [x] 1. Project Setup and Repository Structure
  - Create new Rust project with Cargo workspace
  - Copy existing payment modules from trading-bot (models.rs, processor.rs, verifier.rs, blockchain_monitor.rs, sol_monitor.rs, price_fetcher.rs)
  - Set up dependencies (axum, sqlx, tokio, serde, rust_decimal, reqwest, redis)
  - Configure project structure with src/services/, src/models/, src/api/, src/middleware/
  - _Requirements: 17.1, 17.2_

- [ ] 2. Database Schema and Migrations
  - [x] 2.1 Create initial migration for merchant tables
    - Create merchants table with API key hash, fee percentage, sandbox mode
    - Create merchant_wallets table for blockchain addresses
    - Create webhook_configs table
    - Add indexes for performance
    - _Requirements: 1.1, 1.2, 1.4, 1.5, 4.1, 15.1, 15.2_
  
  - [ ]* 2.2 Write property test for merchant table constraints
    - **Property 1: Unique Merchant Identifiers**
    - **Property 4: One Wallet Per Blockchain**
    - **Validates: Requirements 1.1, 1.5**
  
  - [x] 2.3 Create migration for extended payment tables
    - Alter payment_transactions to add merchant_id, payment_id, fee fields, partial payment fields
    - Create payment_links table
    - Create partial_payments table
    - _Requirements: 2.1, 2.6, 5.1, 20.1, 20.2_
  
  - [x] 2.4 Create migration for webhook and refund tables
    - Create webhook_deliveries table with retry tracking
    - Create refunds table
    - Create audit_logs table
    - Create ip_whitelist table
    - _Requirements: 4.7, 9.1, 18.1, 19.1_
  
  - [ ]* 2.5 Write property test for database schema validation
    - **Property 44: Schema Validation on Startup**
    - **Validates: Requirements 15.6**

- [ ] 3. Core Data Models and Types
  - [x] 3.1 Define Merchant and MerchantWallet models
    - Implement Merchant struct with FromRow derive
    - Implement MerchantWallet struct
    - Add serialization/deserialization
    - _Requirements: 1.1, 1.4_
  
  - [x] 3.2 Extend existing payment models
    - Add merchant_id, payment_id, fee fields to PaymentTransaction
    - Create CreatePaymentRequest and PaymentResponse structs
    - Create PartialPaymentInfo struct
    - _Requirements: 2.1, 2.2, 20.1_
  
  - [x] 3.3 Define webhook and refund models
    - Create WebhookPayload, WebhookDelivery structs
    - Create RefundResponse struct
    - Create AnalyticsReport struct
    - _Requirements: 4.6, 9.1, 11.2_

- [ ] 4. Merchant Service Implementation
  - [x] 4.1 Implement merchant registration
    - Create register_merchant function
    - Generate unique merchant ID
    - Hash and store API key
    - _Requirements: 1.1, 1.2_
  
  - [ ]* 4.2 Write property tests for merchant registration
    - **Property 1: Unique Merchant Identifiers**
    - **Property 2: Unique API Key Generation**
    - **Validates: Requirements 1.1, 1.2**
  
  - [x] 4.3 Implement API key management
    - Create generate_api_key function
    - Implement rotate_api_key function
    - Implement authenticate function with bcrypt verification
    - _Requirements: 1.2, 7.1, 7.5, 7.6_
  
  - [ ]* 4.4 Write property test for API key rotation
    - **Property 27: API Key Rotation**
    - **Validates: Requirements 7.5, 7.6**
  
  - [x] 4.5 Implement wallet address management
    - Create set_wallet_address with validation
    - Implement get_wallet_address function
    - Add blockchain-specific address validation
    - _Requirements: 1.4, 1.5, 1.6_
  
  - [ ]* 4.6 Write property tests for wallet validation
    - **Property 3: Wallet Address Validation**
    - **Property 4: One Wallet Per Blockchain**
    - **Validates: Requirements 1.4, 1.5, 1.6**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Payment Service Implementation
  - [x] 6.1 Adapt existing PaymentProcessor for multi-tenant
    - Modify create_payment to accept merchant_id
    - Generate unique payment_id (e.g., "pay_" + nanoid)
    - Calculate fees and include in total amount
    - Use merchant's wallet address instead of platform wallet
    - _Requirements: 2.1, 2.2, 2.3, 2.6, 6.1_
  
  - [ ]* 6.2 Write property tests for payment creation
    - **Property 5: Unique Payment Identifiers**
    - **Property 6: Crypto Amount Calculation**
    - **Property 8: Fee Inclusion in Total**
    - **Validates: Requirements 2.1, 2.2, 2.6, 6.1**
  
  - [x] 6.3 Implement payment verification with merchant context
    - Adapt verify_payment_by_hash to check merchant ownership
    - Validate transaction against merchant's wallet
    - Update payment status and trigger webhooks
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.7_
  
  - [ ]* 6.4 Write property tests for payment verification
    - **Property 9: Transaction Hash Uniqueness**
    - **Property 10: Amount Verification**
    - **Property 11: Address Verification**
    - **Property 12: Confirmation Threshold**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.7**
  
  - [x] 6.5 Implement payment expiration logic
    - Create background task to check expired payments
    - Update status to expired when time elapses
    - Trigger webhook notifications for expired payments
    - _Requirements: 2.4, 2.7_
  
  - [ ]* 6.6 Write property test for payment expiration
    - **Property 7: Payment Expiration Transition**
    - **Validates: Requirements 2.4, 2.7**
  
  - [x] 6.7 Implement payment listing and filtering
    - Create list_payments with pagination
    - Add filters for status, date range, blockchain
    - _Requirements: 11.3_

- [ ] 7. Webhook Service Implementation
  - [x] 7.1 Implement webhook configuration
    - Create set_webhook_url with HTTPS validation
    - Store webhook configuration in database
    - _Requirements: 4.1_
  
  - [ ]* 7.2 Write property test for webhook URL validation
    - **Property 13: Webhook URL Validation**
    - **Validates: Requirements 4.1**
  
  - [x] 7.3 Implement webhook delivery with signatures
    - Create send_webhook function
    - Generate HMAC-SHA256 signature
    - Include X-Signature and X-Timestamp headers
    - _Requirements: 4.2, 4.3, 4.5, 4.6_
  
  - [ ]* 7.4 Write property tests for webhook delivery
    - **Property 14: Webhook Delivery on Status Change**
    - **Property 16: Webhook Signature Inclusion**
    - **Property 17: Webhook Payload Completeness**
    - **Validates: Requirements 4.2, 4.3, 4.5, 4.6**
  
  - [x] 7.5 Implement webhook retry logic
    - Create retry_failed_webhooks background task
    - Implement exponential backoff (1s, 2s, 4s, 8s, 16s)
    - Update webhook_deliveries table with attempts
    - _Requirements: 4.4, 4.7_
  
  - [ ]* 7.6 Write property test for webhook retry logic
    - **Property 15: Webhook Retry Logic**
    - **Property 18: Webhook Delivery Logging**
    - **Validates: Requirements 4.4, 4.7**

- [x] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 9. Fee Calculation and Collection
  - [x] 9.1 Implement fee calculation logic
    - Create calculate_fee function
    - Validate fee percentage bounds (0.1% - 5%)
    - Calculate fee based on fiat value at creation time
    - _Requirements: 6.1, 6.2, 6.4, 6.6_
  
  - [ ]* 9.2 Write property tests for fee calculation
    - **Property 8: Fee Inclusion in Total** (already covered in 6.2)
    - **Property 22: Fee Percentage Bounds**
    - **Property 24: Fee Calculation Timing**
    - **Validates: Requirements 6.1, 6.4, 6.6**
  
  - [x] 9.3 Implement fee recording on confirmation
    - Update payment record with fee amounts when confirmed
    - Store both crypto and USD fee amounts
    - _Requirements: 6.3_
  
  - [ ]* 9.4 Write property test for fee recording
    - **Property 23: Fee Recording on Confirmation**
    - **Validates: Requirements 6.3**

- [ ] 10. Refund Service Implementation
  - [x] 10.1 Implement refund creation
    - Create create_refund function
    - Validate refund amount doesn't exceed payment
    - Support full and partial refunds
    - Generate unique refund_id
    - _Requirements: 9.1, 9.2, 9.3_
  
  - [ ]* 10.2 Write property tests for refund validation
    - **Property 30: Refund Amount Validation**
    - **Validates: Requirements 9.3**
  
  - [x] 10.3 Implement refund completion and webhooks
    - Create complete_refund function to store transaction hash
    - Trigger webhook notification on refund
    - Update refund status
    - _Requirements: 9.5, 9.6_
  
  - [ ]* 10.4 Write property test for refund webhooks
    - **Property 31: Refund Webhook Notification**
    - **Validates: Requirements 9.5**
  
  - [x] 10.5 Implement balance calculation with refunds
    - Create calculate_merchant_balance function
    - Subtract refunded amounts from total
    - _Requirements: 9.7_
  
  - [ ]* 10.6 Write property test for balance calculation
    - **Property 32: Balance Calculation with Refunds**
    - **Validates: Requirements 9.7**

- [x] 11. Analytics Service Implementation
  - [x] 11.1 Implement analytics calculation
    - Create get_analytics function with date range filtering
    - Calculate total volume, payment counts, fees paid
    - Support filtering by blockchain and status
    - _Requirements: 11.1, 11.2, 11.3_
  
  - [ ]* 11.2 Write property tests for analytics
    - **Property 35: Analytics Volume Calculation**
    - **Property 36: Analytics Report Completeness**
    - **Property 37: Average Transaction Calculation**
    - **Validates: Requirements 11.1, 11.2, 11.5**
  
  - [x] 11.3 Implement CSV export
    - Create export_csv function
    - Format data as CSV with headers
    - Include all payment details
    - _Requirements: 11.7_

- [x] 12. Sandbox Service Implementation
  - [x] 12.1 Implement sandbox mode
    - Create create_sandbox_credentials function
    - Mark API keys as sandbox in database
    - Implement is_sandbox_key check
    - _Requirements: 10.1, 10.4_
  
  - [ ]* 12.2 Write property tests for sandbox isolation
    - **Property 33: Sandbox Data Isolation**
    - **Property 34: Sandbox Payment Marking**
    - **Validates: Requirements 10.4, 10.6**
  
  - [x] 12.3 Implement sandbox payment simulation
    - Create simulate_confirmation function
    - Allow manual status changes in sandbox
    - Skip blockchain verification for sandbox payments
    - _Requirements: 10.2, 10.5_

- [ ] 13. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 14. Partial Payments Implementation
  - [x] 14.1 Implement partial payment tracking
    - Modify payment creation to support partial_payments_enabled flag
    - Create record_partial_payment function
    - Update total_paid and remaining_balance
    - _Requirements: 20.1, 20.2, 20.3_
  
  - [ ]* 14.2 Write property tests for partial payments
    - **Property 51: Partial Payment Tracking**
    - **Property 52: Partial Payment Balance**
    - **Property 53: Partial Payment Completion**
    - **Validates: Requirements 20.2, 20.3, 20.4**
  
  - [x] 14.3 Implement partial payment webhooks and expiration
    - Send webhook for each partial payment
    - Extend expiration time on each payment
    - _Requirements: 20.5, 20.7_
  
  - [ ]* 14.4 Write property tests for partial payment features
    - **Property 54: Partial Payment Webhooks**
    - **Property 55: Partial Payment Expiration Extension**
    - **Validates: Requirements 20.5, 20.7**

- [x] 15. Payment Link and Hosted Pages
  - [x] 15.1 Implement payment link generation
    - Generate unique link_id (short alphanumeric)
    - Store in payment_links table
    - Return full URL with base domain
    - _Requirements: 5.1_
  
  - [ ]* 15.2 Write property test for unique payment links
    - **Property 19: Unique Payment Links**
    - **Validates: Requirements 5.1**
  
  - [x] 15.3 Create hosted payment page HTML template
    - Design responsive payment page with Tailwind CSS
    - Display payment amount in crypto and fiat
    - Show QR code using qrcode library
    - Display countdown timer for expiration
    - _Requirements: 5.2, 5.3, 5.4, 5.5_
  
  - [ ]* 15.4 Write property test for payment page content
    - **Property 20: Payment Page Content**
    - **Property 21: Payment Page Status Display**
    - **Validates: Requirements 5.2, 5.3, 5.4, 5.5, 5.6, 5.7**
  
  - [x] 15.5 Implement payment page status updates
    - Show success message when payment confirmed
    - Show expired message when payment expires
    - Add WebSocket or polling for real-time updates
    - _Requirements: 5.6, 5.7_

- [x] 16. API Layer with Axum
  - [x] 16.1 Set up Axum router and middleware
    - Create main router with all endpoints
    - Add authentication middleware
    - Add rate limiting middleware
    - Add request logging middleware
    - _Requirements: 7.1, 7.2, 7.3, 7.7_
  
  - [x] 16.2 Implement merchant API endpoints
    - POST /api/v1/merchants/register
    - POST /api/v1/merchants/api-keys/rotate
    - PUT /api/v1/merchants/wallets
    - GET /api/v1/merchants/wallets
    - PUT /api/v1/merchants/webhook
    - _Requirements: 1.1, 1.2, 1.4, 4.1, 7.5, 7.6_
  
  - [x] 16.3 Implement payment API endpoints
    - POST /api/v1/payments
    - GET /api/v1/payments/:payment_id
    - POST /api/v1/payments/:payment_id/verify
    - GET /api/v1/payments (with filters)
    - _Requirements: 2.1, 3.1_
  
  - [x] 16.4 Implement refund and analytics endpoints
    - POST /api/v1/refunds
    - GET /api/v1/refunds/:refund_id
    - POST /api/v1/refunds/:refund_id/complete
    - GET /api/v1/analytics
    - GET /api/v1/analytics/export
    - _Requirements: 9.1, 11.1, 11.7_
  
  - [x] 16.5 Implement sandbox endpoints
    - POST /api/v1/sandbox/enable
    - POST /api/v1/sandbox/payments/:payment_id/simulate
    - _Requirements: 10.1, 10.5_
  
  - [x] 16.6 Implement hosted payment page endpoint
    - GET /pay/:link_id
    - Render HTML template with payment data
    - _Requirements: 5.1, 5.2_

- [x] 17. Authentication and Authorization Middleware
  - [x] 17.1 Implement API key authentication
    - Extract API key from Authorization header
    - Validate against database (bcrypt hash)
    - Attach merchant to request context
    - Return 401 for invalid keys
    - _Requirements: 7.1, 7.2_
  
  - [ ]* 17.2 Write property test for authentication
    - **Property 25: API Key Authentication**
    - **Validates: Requirements 7.1, 7.2**
  
  - [x] 17.3 Implement rate limiting middleware
    - Use Redis for distributed rate limiting
    - Track requests per API key per minute
    - Return 429 when limit exceeded
    - _Requirements: 7.3, 7.4_
  
  - [ ]* 17.4 Write property test for rate limiting
    - **Property 26: Rate Limiting**
    - **Validates: Requirements 7.3, 7.4**
  
  - [x] 17.5 Implement IP whitelisting middleware
    - Check request IP against merchant's whitelist
    - Support CIDR ranges
    - Return 403 for non-whitelisted IPs
    - _Requirements: 18.2, 18.3_
  
  - [ ]* 17.6 Write property tests for IP whitelisting
    - **Property 47: IP Whitelist Enforcement**
    - **Property 49: Empty Whitelist Behavior**
    - **Validates: Requirements 18.2, 18.3, 18.7**

- [x] 18. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 19. IP Whitelist Management
  - [x] 19.1 Implement IP whitelist configuration
    - Create set_ip_whitelist function
    - Validate IP addresses and CIDR ranges
    - Enforce 10 entry limit
    - _Requirements: 18.1, 18.4_
  
  - [ ]* 19.2 Write property tests for IP whitelist
    - **Property 46: IP Address Validation**
    - **Property 48: IP Whitelist Limit**
    - **Validates: Requirements 18.1, 18.4**
  
  - [x] 19.3 Implement IP whitelist logging
    - Log all rejected requests due to IP restrictions
    - Include timestamp, IP, merchant_id
    - _Requirements: 18.6_

- [x] 20. Audit Logging Implementation
  - [x] 20.1 Implement audit log creation
    - Create log_audit_event function
    - Log merchant changes, API key operations, payment events
    - Include merchant_id, action_type, IP, timestamp
    - _Requirements: 19.1, 19.2, 19.3, 19.4, 19.5_
  
  - [ ]* 20.2 Write property test for audit logging
    - **Property 50: Audit Log Creation**
    - **Validates: Requirements 19.1, 19.2, 19.3, 19.4**
  
  - [x] 20.3 Implement audit log query endpoint
    - Create GET /api/v1/audit-logs endpoint
    - Filter by date range and action type
    - Ensure merchants can only see their own logs
    - _Requirements: 19.7_

- [x] 21. Error Handling and Retry Logic
  - [x] 21.1 Implement error types and responses
    - Create ServiceError enum with all error types
    - Implement IntoResponse for ServiceError
    - Return consistent JSON error format
    - _Requirements: 13.3_
  
  - [ ]* 21.2 Write property test for error logging
    - **Property 42: Error Logging Context**
    - **Validates: Requirements 13.6**
  
  - [x] 21.3 Implement blockchain query retry logic
    - Create query_blockchain_with_retry function
    - Retry up to 3 times with exponential backoff
    - _Requirements: 13.1_
  
  - [ ]* 21.4 Write property test for blockchain retry
    - **Property 40: Blockchain Query Retry**
    - **Validates: Requirements 13.1**
  
  - [x] 21.5 Implement circuit breaker for external APIs
    - Create CircuitBreaker struct
    - Track failures and open circuit after threshold
    - Use cached data when circuit is open
    - _Requirements: 13.4, 13.5_
  
  - [ ]* 21.6 Write property test for circuit breaker
    - **Property 41: Circuit Breaker Behavior**
    - **Validates: Requirements 13.4, 13.5**

- [x] 22. Price Fetching and Caching
  - [x] 22.1 Adapt existing PriceFetcher for caching
    - Add Redis caching layer
    - Cache prices for 30 seconds
    - Implement fallback to cached price on API failure
    - _Requirements: 12.1, 12.2, 12.3, 12.4_
  
  - [ ]* 22.2 Write property tests for price caching
    - **Property 38: Price Cache Duration**
    - **Property 39: Price API Fallback**
    - **Validates: Requirements 12.3, 12.4**
  
  - [x] 22.3 Implement multi-currency support
    - Add EUR and GBP conversion
    - Fetch exchange rates from Bybit
    - _Requirements: 12.5_

- [x] 23. Background Tasks and Monitoring
  - [x] 23.1 Implement payment monitoring background task
    - Reuse existing monitor_pending_payments logic
    - Adapt for multi-tenant (check all merchants)
    - Run continuously with 30-second intervals
    - _Requirements: 3.6_
  
  - [x] 23.2 Implement webhook retry background task
    - Create retry_failed_webhooks task
    - Check for pending webhooks past next_retry_at
    - Retry with exponential backoff
    - _Requirements: 4.4_
  
  - [x] 23.3 Implement payment expiration background task
    - Check for payments past expiration time
    - Update status to expired
    - Trigger webhook notifications
    - _Requirements: 2.4, 2.7_
  
  - [x] 23.4 Implement health check endpoints
    - Create /health endpoint (liveness)
    - Create /health/ready endpoint (readiness)
    - Check database and Redis connectivity
    - _Requirements: 16.1_
  
  - [ ]* 23.5 Implement Prometheus metrics
    - Expose /metrics endpoint
    - Track request counts, durations, payment counts
    - Track webhook delivery success/failure rates
    - _Requirements: 16.3_

- [x] 24. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 25. Docker and Deployment Configuration
  - [x] 25.1 Create Dockerfile
    - Multi-stage build with Rust builder
    - Minimal runtime image with Debian slim
    - Copy migrations and binary
    - Add healthcheck
    - _Requirements: 17.1, 17.6_
  
  - [x] 25.2 Create docker-compose.yml
    - Define gateway, postgres, redis services
    - Add volume mounts for persistence
    - Configure health checks
    - _Requirements: 17.2_
  
  - [x] 25.3 Create environment variable configuration
    - Document all required environment variables
    - Implement validation on startup
    - _Requirements: 17.3, 17.4_
  
  - [x] 25.4 Implement graceful shutdown
    - Handle SIGTERM signal
    - Drain active connections
    - Complete in-flight requests
    - _Requirements: 17.5_

- [ ] 26. Integration Testing
  - [ ]* 26.1 Write integration tests for payment flow
    - Test complete payment creation to confirmation flow
    - Test webhook delivery
    - Test expiration handling
  
  - [ ]* 26.2 Write integration tests for refund flow
    - Test refund creation and completion
    - Test balance calculations
  
  - [ ]* 26.3 Write integration tests for API authentication
    - Test API key validation
    - Test rate limiting
    - Test IP whitelisting
  
  - [ ]* 26.4 Write integration tests for sandbox mode
    - Test sandbox isolation
    - Test manual payment simulation

- [x] 27. Documentation and API Specification
  - [x] 27.1 Generate OpenAPI/Swagger specification
    - Use OpenAPI 3.0 specification
    - Document all endpoints with examples
    - Include error responses
    - _Requirements: 14.1_
  
  - [x] 27.2 Create API documentation
    - Write integration guide
    - Add code examples in JavaScript, Python, PHP
    - Document webhook payload format
    - Create Postman collection
    - _Requirements: 14.2, 14.5_
  
  - [x] 27.3 Create README and deployment guide
    - Document environment variables
    - Provide docker-compose setup instructions
    - Include troubleshooting section
    - _Requirements: 17.7_

- [ ] 28. Final Integration and Testing
  - [~] 28.1 End-to-end testing with all blockchains
    - Test payment creation for all supported chains
    - Test verification with real blockchain data (testnet)
    - Verify webhook delivery
  
  - [~] 28.2 Load testing and performance validation
    - Test rate limiting under load
    - Verify database connection pooling
    - Test concurrent payment processing
  
  - [~] 28.3 Security audit
    - Verify API key hashing
    - Test IP whitelisting
    - Validate webhook signatures
    - Check for SQL injection vulnerabilities

- [~] 29. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

---

## PHASE 2: CRITICAL FEATURES FOR LAUNCH

- [x] 30. Balance Management System
  - [x] 30.1 Implement merchant balance tracking
    - Create merchant_balances table (per currency)
    - Track available vs reserved balance
    - Update balance on payment confirmation
    - Update balance on refund
    - _Requirements: Balance tracking, merchant dashboard_
  
  - [x] 30.2 Implement balance query endpoint
    - GET /api/v1/merchants/balance
    - Return total USD value and per-currency breakdown
    - Show available vs reserved amounts
    - _Requirements: Merchant API_
  
  - [x] 30.3 Implement balance history
    - Create balance_history table
    - Record all balance changes with reason
    - Support date range queries
    - _Requirements: Analytics, audit trail_

- [x] 31. Withdrawal System
  - [x] 31.1 Implement withdrawal request
    - Create withdrawals table
    - POST /api/v1/withdrawals endpoint
    - Validate sufficient available balance
    - Validate minimum withdrawal amount ($10)
    - Generate unique withdrawal_id
    - _Requirements: Merchant payout, balance management_
  
  - [x] 31.2 Implement withdrawal approval workflow
    - Auto-approve withdrawals under $1000
    - Manual approval for larger amounts
    - Update withdrawal status (pending/approved/rejected)
    - _Requirements: Security, fraud prevention_
  
  - [x] 31.3 Implement withdrawal processing
    - Process approved withdrawals to blockchain
    - Record transaction hash
    - Update merchant balance
    - Send confirmation webhook/email
    - _Requirements: Blockchain integration_
  
  - [x] 31.4 Implement withdrawal endpoints
    - GET /api/v1/withdrawals (list with filters)
    - GET /api/v1/withdrawals/:id (details)
    - POST /api/v1/withdrawals/:id/cancel (if pending)
    - _Requirements: Merchant API_

- [x] 32. Invoice System
  - [x] 32.1 Implement invoice creation
    - Create invoices table with line items (JSONB)
    - POST /api/v1/invoices endpoint
    - Generate unique invoice_id (inv_xxx)
    - Calculate totals with tax support
    - _Requirements: B2B payments, recurring billing_
  
  - [x] 32.2 Implement invoice payment flow
    - Link invoice to payment on creation
    - Auto-mark invoice as paid when payment confirmed
    - Support partial invoice payments
    - _Requirements: Payment tracking_
  
  - [x] 32.3 Implement invoice endpoints
    - GET /api/v1/invoices (list with filters)
    - GET /api/v1/invoices/:id (details)
    - GET /invoice/:id (hosted invoice page)
    - GET /api/v1/invoices/:id/pdf (PDF generation)
    - _Requirements: Merchant tools_

- [x] 33. Email Notification System
  - [x] 33.1 Set up email service integration
    - Integrate with SendGrid or AWS SES
    - Create email templates (HTML + text)
    - Implement send_email function
    - _Requirements: Customer communication_
  
  - [x] 33.2 Implement merchant notifications
    - Payment received email
    - Withdrawal completed email
    - API key rotated email
    - Security alert emails
    - _Requirements: Merchant engagement_
  
  - [x] 33.3 Implement customer notifications
    - Payment confirmation email
    - Receipt with transaction details
    - Refund processed email
    - Invoice emails
    - _Requirements: Customer experience_
  
  - [x] 33.4 Implement email preferences
    - Allow merchants to configure which emails to receive
    - Store preferences in merchants table
    - _Requirements: User control_

- [x] 34. Two-Factor Authentication (2FA)
  - [x] 34.1 Implement TOTP setup
    - Generate TOTP secret for merchant
    - Create QR code for authenticator app
    - Store encrypted secret in database
    - POST /api/v1/merchants/2fa/enable endpoint
    - _Requirements: Security_
  
  - [x] 34.2 Implement 2FA verification
    - Verify TOTP code on sensitive operations
    - Add 2fa_code parameter to withdrawal/rotation endpoints
    - Return 403 if 2FA required but not provided
    - _Requirements: Security_
  
  - [x] 34.3 Implement 2FA recovery codes
    - Generate 10 recovery codes on 2FA setup
    - Allow one-time use of recovery codes
    - POST /api/v1/merchants/2fa/disable endpoint (with code)
    - _Requirements: Account recovery_

- [x] 35. Multi-User Account Management
  - [x] 35.1 Implement team member system
    - Create merchant_users table
    - Link users to merchant with roles (owner/admin/developer/viewer)
    - POST /api/v1/merchants/users endpoint
    - _Requirements: Team collaboration_
  
  - [x] 35.2 Implement role-based permissions
    - Define permissions per role
    - Implement permission checks in middleware
    - Restrict sensitive operations to owner/admin
    - _Requirements: Security, access control_
  
  - [x] 35.3 Implement user management endpoints
    - GET /api/v1/merchants/users (list team)
    - PUT /api/v1/merchants/users/:id (update role)
    - DELETE /api/v1/merchants/users/:id (remove user)
    - _Requirements: Team management_

---

## PHASE 3: COMPETITIVE ADVANTAGE FEATURES

- [ ] 36. Smart Payment Routing
  - [ ] 36.1 Implement network fee estimation
    - Fetch current gas prices for EVM chains
    - Fetch Solana transaction fees
    - Cache fees for 1 minute
    - _Requirements: Cost optimization_
  
  - [ ] 36.2 Implement automatic network selection
    - When merchant accepts USDT on multiple networks
    - Calculate total cost (amount + network fee)
    - Suggest cheapest network to customer
    - _Requirements: Customer savings_
  
  - [ ] 36.3 Add network selection to payment page
    - Show network options with estimated fees
    - Allow customer to choose network
    - Update QR code dynamically
    - _Requirements: User experience_

- [ ] 37. Payment Splitting
  - [ ] 37.1 Implement split payment configuration
    - Add split_config JSONB to payments table
    - Support percentage or fixed amount splits
    - Validate splits total 100% or full amount
    - _Requirements: Marketplace, affiliate payments_
  
  - [ ] 37.2 Implement automatic split distribution
    - On payment confirmation, calculate split amounts
    - Create sub-transactions for each recipient
    - Update balances for all recipients
    - _Requirements: Multi-party payments_
  
  - [ ] 37.3 Add split payment API
    - Add splits parameter to POST /api/v1/payments
    - Return split details in payment response
    - _Requirements: API flexibility_

- [ ] 38. Zero-Fee Stablecoin Payments
  - [ ] 38.1 Implement dynamic fee structure
    - Add fee_percentage to crypto_types configuration
    - Set 0% for USDT/USDC, 1.5% for volatile crypto
    - Update fee calculation logic
    - _Requirements: Competitive pricing_
  
  - [ ] 38.2 Update pricing page and documentation
    - Highlight zero-fee stablecoins
    - Show fee comparison with competitors
    - _Requirements: Marketing_

- [ ] 39. Escrow Service
  - [ ] 39.1 Implement escrow payment type
    - Add escrow_config to payments table
    - Support release conditions (manual/time-based)
    - Create escrow_releases table
    - _Requirements: Trust, dispute resolution_
  
  - [ ] 39.2 Implement escrow release endpoints
    - POST /api/v1/payments/:id/release (manual release)
    - POST /api/v1/payments/:id/dispute (dispute escrow)
    - Require both parties to confirm release
    - _Requirements: Marketplace safety_

- [ ] 40. JavaScript SDK
  - [ ] 40.1 Create NPM package
    - Initialize TypeScript project
    - Implement API client with all endpoints
    - Add TypeScript types for all models
    - _Requirements: Developer experience_
  
  - [ ] 40.2 Add convenience methods
    - createPayment() with automatic polling
    - verifyWebhook() helper
    - Event emitters for payment status
    - _Requirements: Easy integration_
  
  - [ ] 40.3 Publish and document SDK
    - Publish to NPM
    - Create SDK documentation with examples
    - Add to main documentation site
    - _Requirements: Adoption_

- [ ] 41. WooCommerce Plugin
  - [ ] 41.1 Create WordPress plugin
    - Initialize plugin structure
    - Add settings page for API key configuration
    - Integrate with WooCommerce checkout
    - _Requirements: E-commerce integration_
  
  - [ ] 41.2 Implement payment gateway
    - Add crypto payment option at checkout
    - Redirect to hosted payment page
    - Handle webhook to mark order as paid
    - _Requirements: Merchant adoption_
  
  - [ ] 41.3 Publish plugin
    - Submit to WordPress plugin directory
    - Create installation guide
    - _Requirements: Distribution_

---

## PHASE 4: SCALE & GROWTH FEATURES

- [ ] 42. Additional Blockchains
  - [ ] 42.1 Add Ethereum mainnet support
    - Implement ETH monitor
    - Add ETH price fetching
    - Update payment processor
    - _Requirements: Network expansion_
  
  - [ ] 42.2 Add Base support
    - Implement Base monitor (EVM)
    - Add USDC on Base
    - _Requirements: L2 scaling_
  
  - [ ] 42.3 Add Optimism support
    - Implement Optimism monitor
    - Add USDT/USDC on Optimism
    - _Requirements: L2 scaling_
  
  - [ ] 42.4 Add more tokens
    - USDC on all networks
    - DAI on Ethereum/Polygon
    - Native tokens (ETH, BNB, MATIC)
    - _Requirements: Currency diversity_

- [ ] 43. KYC/AML Integration
  - [ ] 43.1 Integrate KYC provider (Sumsub/Onfido)
    - Add kyc_status to merchants table
    - Implement KYC verification flow
    - Store verification results
    - _Requirements: Compliance_
  
  - [ ] 43.2 Implement transaction limits
    - Enforce $1000/month limit for non-KYC merchants
    - Track monthly volume per merchant
    - Require KYC for higher volumes
    - _Requirements: Regulatory compliance_
  
  - [ ] 43.3 Implement AML monitoring
    - Check addresses against blacklists
    - Flag suspicious transactions
    - Generate compliance reports
    - _Requirements: Risk management_

- [ ] 44. Advanced Analytics Dashboard
  - [ ] 44.1 Implement revenue forecasting
    - Calculate 30/60/90 day projections
    - Show growth trends
    - _Requirements: Merchant insights_
  
  - [ ] 44.2 Implement conversion analytics
    - Track payment page views vs completions
    - Calculate conversion rates
    - Show abandonment reasons
    - _Requirements: Optimization_
  
  - [ ] 44.3 Implement geographic analytics
    - Track customer locations (from IP)
    - Show payment distribution by country
    - _Requirements: Market insights_

- [ ] 45. Mobile Wallet Integration
  - [ ] 45.1 Implement WalletConnect support
    - Add WalletConnect SDK to payment page
    - Allow one-click payment from mobile wallets
    - Support MetaMask, Trust Wallet, Phantom
    - _Requirements: Mobile UX_
  
  - [ ] 45.2 Implement deep linking
    - Generate wallet-specific deep links
    - Auto-open wallet apps on mobile
    - _Requirements: Conversion optimization_

- [ ] 46. Tax Reporting
  - [ ] 46.1 Implement tax calculation
    - Calculate tax per jurisdiction
    - Support VAT, sales tax
    - _Requirements: Compliance_
  
  - [ ] 46.2 Generate tax reports
    - Export tax reports for accounting
    - Support multiple tax formats
    - _Requirements: Merchant tools_

---

## PHASE 5: ENTERPRISE FEATURES

- [ ] 47. White-Label Solution
  - [ ] 47.1 Implement custom branding
    - Allow merchants to customize payment page
    - Support custom domain (pay.merchant.com)
    - Custom logo and colors
    - _Requirements: Enterprise sales_

- [ ] 48. Multi-Signature Payments
  - [ ] 48.1 Implement multi-sig wallet support
    - Support payments requiring multiple approvals
    - Track approval status
    - _Requirements: Enterprise security_

- [ ] 49. Custom Reports Builder
  - [ ] 49.1 Implement report builder UI
    - Allow merchants to create custom reports
    - Support filtering and grouping
    - Schedule automated reports
    - _Requirements: Enterprise analytics_

- [ ] 50. Dedicated Support System
  - [ ] 50.1 Implement support ticket system
    - Create tickets table
    - POST /api/v1/support/tickets endpoint
    - Email integration
    - _Requirements: Customer success_

---

## Notes

- Tasks marked with `*` are optional property tests (implement after core features)
- Tasks 1-29: Core MVP (90% complete)
- Tasks 30-35: Critical for launch (Phase 1 - 2 weeks)
- Tasks 36-41: Competitive advantage (Phase 2 - Month 1)
- Tasks 42-46: Scale & growth (Phase 3 - Month 2-3)
- Tasks 47-50: Enterprise features (Phase 4 - Month 4+)
- Each task references requirements for traceability
- Property tests will be implemented after all core features are complete
- Focus on getting to market quickly with Phase 1, then iterate based on customer feedback

