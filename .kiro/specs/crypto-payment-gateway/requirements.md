# Requirements Document: Crypto Payment Gateway Service

## Introduction

This document specifies the requirements for transforming an existing crypto payment gateway (built for a trading bot SaaS) into a standalone payment service that enables merchants to accept cryptocurrency payments. The service will support multiple blockchains, provide merchant management capabilities, and generate revenue through transaction fees.

## Glossary

- **Gateway**: The crypto payment gateway service system
- **Merchant**: A business or individual using the Gateway to accept crypto payments
- **Payment**: A cryptocurrency transaction initiated by a customer to pay a merchant
- **Webhook**: An HTTP callback that delivers real-time payment notifications to merchants
- **API_Key**: A unique authentication token issued to each merchant for API access
- **Transaction_Hash**: A unique blockchain identifier for a cryptocurrency transaction
- **Blockchain_Monitor**: The component that watches blockchain networks for payment confirmations
- **Payment_Link**: A unique URL that directs customers to a payment page
- **Fee_Split**: The division of payment amounts between merchant wallet and platform fee collection
- **Confirmation**: A blockchain validation that a transaction has been successfully processed
- **Sandbox_Mode**: A testing environment where merchants can test integrations without real transactions

## Requirements

### Requirement 1: Merchant Registration and Management

**User Story:** As a business owner, I want to register as a merchant and receive API credentials, so that I can integrate crypto payments into my application.

#### Acceptance Criteria

1. WHEN a user submits valid registration information, THE Gateway SHALL create a merchant account with a unique identifier
2. WHEN a merchant account is created, THE Gateway SHALL generate a unique API_Key for authentication
3. WHEN a merchant requests their API credentials, THE Gateway SHALL provide the API_Key securely
4. WHEN a merchant provides wallet addresses for supported blockchains, THE Gateway SHALL validate and store them
5. THE Gateway SHALL support multiple wallet addresses per merchant (one per blockchain)
6. WHEN a merchant updates their wallet addresses, THE Gateway SHALL validate the new addresses before saving

### Requirement 2: Payment Creation and Processing

**User Story:** As a merchant, I want to create payment requests via API, so that my customers can pay me in cryptocurrency.

#### Acceptance Criteria

1. WHEN a merchant creates a payment request with valid parameters, THE Gateway SHALL generate a unique payment identifier
2. WHEN a payment is created, THE Gateway SHALL calculate the crypto amount using real-time exchange rates
3. WHEN a payment is created, THE Gateway SHALL generate a payment address for the selected blockchain
4. WHEN a payment is created with an expiration time, THE Gateway SHALL mark it as expired after that time elapses
5. THE Gateway SHALL support payment creation for SOL, USDT on Solana, USDT on BSC, USDT on Arbitrum, and USDT on Polygon
6. WHEN calculating payment amounts, THE Gateway SHALL include the platform fee in the total
7. WHEN a payment expires without completion, THE Gateway SHALL update its status to expired

### Requirement 3: Payment Verification and Confirmation

**User Story:** As a merchant, I want payments to be automatically verified on the blockchain, so that I can trust the payment is legitimate.

#### Acceptance Criteria

1. WHEN a transaction hash is submitted for a payment, THE Gateway SHALL verify it exists on the specified blockchain
2. WHEN verifying a transaction, THE Gateway SHALL confirm the amount matches the expected payment amount
3. WHEN verifying a transaction, THE Gateway SHALL confirm the recipient address matches the merchant's wallet
4. WHEN a transaction receives sufficient blockchain confirmations, THE Gateway SHALL mark the payment as confirmed
5. IF a transaction hash is invalid or does not match payment details, THEN THE Gateway SHALL reject the verification
6. THE Gateway SHALL monitor pending payments continuously until they are confirmed or expired
7. WHEN a payment is confirmed, THE Gateway SHALL update the payment status to completed

### Requirement 4: Webhook Notifications

**User Story:** As a merchant, I want to receive real-time notifications when payment status changes, so that I can update my application immediately.

#### Acceptance Criteria

1. WHEN a merchant configures a webhook URL, THE Gateway SHALL validate it is a valid HTTPS endpoint
2. WHEN a payment status changes to confirmed, THE Gateway SHALL send a webhook notification to the merchant's configured URL
3. WHEN a payment status changes to expired, THE Gateway SHALL send a webhook notification to the merchant's configured URL
4. WHEN a webhook delivery fails, THE Gateway SHALL retry with exponential backoff up to 5 attempts
5. THE Gateway SHALL include a signature in webhook requests for merchants to verify authenticity
6. WHEN a webhook is sent, THE Gateway SHALL include payment details, status, and timestamp in the payload
7. THE Gateway SHALL log all webhook delivery attempts and their results

### Requirement 5: Payment Links and Hosted Pages

**User Story:** As a merchant, I want to generate payment links with QR codes, so that customers can easily pay without custom integration.

#### Acceptance Criteria

1. WHEN a merchant creates a payment, THE Gateway SHALL generate a unique Payment_Link
2. WHEN a customer visits a Payment_Link, THE Gateway SHALL display a hosted payment page with payment details
3. WHEN displaying a payment page, THE Gateway SHALL show a QR code containing the payment address and amount
4. WHEN displaying a payment page, THE Gateway SHALL show the payment amount in both crypto and fiat currency
5. WHEN displaying a payment page, THE Gateway SHALL show the time remaining until expiration
6. WHEN a payment is completed, THE Gateway SHALL update the payment page to show success status
7. WHEN a payment expires, THE Gateway SHALL update the payment page to show expired status

### Requirement 6: Fee Calculation and Collection

**User Story:** As the platform operator, I want to collect fees on each transaction, so that the service generates revenue.

#### Acceptance Criteria

1. WHEN calculating a payment amount, THE Gateway SHALL add the platform fee percentage to the total
2. THE Gateway SHALL support configurable fee percentages per merchant tier
3. WHEN a payment is confirmed, THE Gateway SHALL record the fee amount collected
4. THE Gateway SHALL support fee percentages between 0.1% and 5%
5. WHEN a merchant is assigned a pricing tier, THE Gateway SHALL apply the corresponding fee percentage
6. THE Gateway SHALL calculate fees based on the fiat value of the payment at creation time

### Requirement 7: API Authentication and Security

**User Story:** As a merchant, I want secure API access with authentication, so that only authorized requests can access my payment data.

#### Acceptance Criteria

1. WHEN a request includes a valid API_Key in the authorization header, THE Gateway SHALL authenticate the merchant
2. WHEN a request includes an invalid or missing API_Key, THE Gateway SHALL reject it with a 401 status code
3. THE Gateway SHALL implement rate limiting of 100 requests per minute per API_Key
4. WHEN rate limits are exceeded, THE Gateway SHALL reject requests with a 429 status code
5. THE Gateway SHALL support API_Key rotation without service interruption
6. WHEN a merchant requests API_Key regeneration, THE Gateway SHALL invalidate the old key and generate a new one
7. THE Gateway SHALL log all API requests with timestamp, endpoint, and merchant identifier

### Requirement 8: Multi-Blockchain Support

**User Story:** As a merchant, I want to accept payments on multiple blockchains, so that customers can pay using their preferred network.

#### Acceptance Criteria

1. THE Gateway SHALL support payment monitoring on Solana, Binance Smart Chain, Arbitrum, and Polygon
2. WHEN monitoring Solana payments, THE Gateway SHALL support both SOL native and USDT SPL tokens
3. WHEN monitoring BSC payments, THE Gateway SHALL support USDT BEP20 tokens
4. WHEN monitoring Arbitrum payments, THE Gateway SHALL support USDT tokens
5. WHEN monitoring Polygon payments, THE Gateway SHALL support USDT tokens
6. THE Gateway SHALL use the existing blockchain monitoring code for transaction verification
7. WHEN a blockchain network is unavailable, THE Gateway SHALL continue monitoring other networks

### Requirement 9: Refund System

**User Story:** As a merchant, I want to issue refunds for completed payments, so that I can handle customer disputes and returns.

#### Acceptance Criteria

1. WHEN a merchant requests a refund for a completed payment, THE Gateway SHALL create a refund record
2. WHEN creating a refund, THE Gateway SHALL support full or partial refund amounts
3. WHEN a refund is created, THE Gateway SHALL validate the refund amount does not exceed the original payment
4. THE Gateway SHALL track refund status (pending, completed, failed)
5. WHEN a refund is issued, THE Gateway SHALL send a webhook notification to the merchant
6. THE Gateway SHALL store the refund transaction hash once processed on the blockchain
7. WHEN calculating merchant balances, THE Gateway SHALL subtract refunded amounts

### Requirement 10: Sandbox Testing Mode

**User Story:** As a merchant, I want to test my integration in a sandbox environment, so that I can verify functionality without real transactions.

#### Acceptance Criteria

1. WHEN a merchant enables Sandbox_Mode, THE Gateway SHALL generate test API credentials
2. WHEN operating in Sandbox_Mode, THE Gateway SHALL simulate payment confirmations without blockchain verification
3. WHEN in Sandbox_Mode, THE Gateway SHALL send webhook notifications to test endpoints
4. THE Gateway SHALL clearly distinguish sandbox payments from production payments in all responses
5. WHEN in Sandbox_Mode, THE Gateway SHALL allow manual payment status changes for testing
6. THE Gateway SHALL prevent sandbox API keys from accessing production payment data
7. WHEN a merchant switches from sandbox to production, THE Gateway SHALL require explicit confirmation

### Requirement 11: Analytics and Reporting

**User Story:** As a merchant, I want to view transaction analytics and reports, so that I can track my payment volume and revenue.

#### Acceptance Criteria

1. WHEN a merchant requests analytics, THE Gateway SHALL provide total transaction volume for a specified time period
2. WHEN generating reports, THE Gateway SHALL include successful payment count, failed payment count, and total fees paid
3. THE Gateway SHALL support filtering analytics by date range, blockchain, and payment status
4. WHEN displaying analytics, THE Gateway SHALL show amounts in both crypto and fiat currency
5. THE Gateway SHALL calculate and display average transaction value
6. THE Gateway SHALL provide daily, weekly, and monthly aggregated reports
7. WHEN a merchant exports analytics, THE Gateway SHALL provide data in CSV format

### Requirement 12: Price Fetching and Currency Conversion

**User Story:** As a merchant, I want accurate real-time crypto prices, so that payment amounts reflect current market rates.

#### Acceptance Criteria

1. THE Gateway SHALL fetch real-time cryptocurrency prices from Bybit API
2. WHEN creating a payment, THE Gateway SHALL use the current exchange rate to calculate crypto amounts
3. THE Gateway SHALL cache price data for up to 30 seconds to reduce API calls
4. WHEN the price API is unavailable, THE Gateway SHALL use the last cached price and log a warning
5. THE Gateway SHALL support price conversion for USD, EUR, and GBP fiat currencies
6. WHEN displaying payment amounts, THE Gateway SHALL show both the crypto amount and fiat equivalent
7. THE Gateway SHALL refresh cached prices every 30 seconds for active payment sessions

### Requirement 13: Error Handling and Retry Logic

**User Story:** As a merchant, I want reliable payment processing with automatic retries, so that temporary failures don't result in lost payments.

#### Acceptance Criteria

1. WHEN a blockchain query fails, THE Gateway SHALL retry up to 3 times with exponential backoff
2. WHEN webhook delivery fails, THE Gateway SHALL retry up to 5 times with exponential backoff
3. WHEN a database operation fails, THE Gateway SHALL log the error and return an appropriate error response
4. THE Gateway SHALL implement circuit breakers for external API calls (Bybit, blockchain RPCs)
5. WHEN a circuit breaker opens, THE Gateway SHALL use cached data or return a service degraded response
6. THE Gateway SHALL log all errors with context including merchant ID, payment ID, and error details
7. WHEN critical errors occur, THE Gateway SHALL send alerts to system administrators

### Requirement 14: API Documentation and Developer Experience

**User Story:** As a developer integrating the Gateway, I want comprehensive API documentation, so that I can implement the integration quickly.

#### Acceptance Criteria

1. THE Gateway SHALL provide OpenAPI/Swagger specification for all API endpoints
2. THE Gateway SHALL include code examples in JavaScript, Python, and PHP for common operations
3. THE Gateway SHALL provide interactive API documentation with a "try it out" feature
4. THE Gateway SHALL document all error codes and their meanings
5. THE Gateway SHALL provide webhook payload examples with all possible fields
6. THE Gateway SHALL include integration guides for common platforms (e-commerce, SaaS)
7. THE Gateway SHALL maintain a changelog documenting API version changes

### Requirement 15: Database Schema and Migrations

**User Story:** As a system administrator, I want versioned database migrations, so that schema changes can be applied safely across environments.

#### Acceptance Criteria

1. THE Gateway SHALL use PostgreSQL for data persistence
2. THE Gateway SHALL implement database migrations using a migration tool (e.g., sqlx, diesel)
3. WHEN deploying a new version, THE Gateway SHALL apply pending migrations automatically
4. THE Gateway SHALL version all schema changes with timestamps and descriptions
5. THE Gateway SHALL support rollback of migrations for the last 5 versions
6. THE Gateway SHALL validate database schema matches expected version on startup
7. WHEN a migration fails, THE Gateway SHALL halt startup and log the error

### Requirement 16: Monitoring and Observability

**User Story:** As a system administrator, I want comprehensive monitoring and logging, so that I can diagnose issues and ensure system health.

#### Acceptance Criteria

1. THE Gateway SHALL expose health check endpoints for liveness and readiness probes
2. THE Gateway SHALL log all payment state transitions with timestamps
3. THE Gateway SHALL expose Prometheus metrics for request rates, error rates, and latency
4. THE Gateway SHALL track and expose metrics for blockchain confirmation times per network
5. THE Gateway SHALL log all external API calls with duration and response status
6. WHEN error rates exceed thresholds, THE Gateway SHALL trigger alerts
7. THE Gateway SHALL provide structured JSON logging for all application logs

### Requirement 17: Deployment and Infrastructure

**User Story:** As a DevOps engineer, I want containerized deployment with Docker, so that the service can be deployed consistently across environments.

#### Acceptance Criteria

1. THE Gateway SHALL provide a Dockerfile for building container images
2. THE Gateway SHALL provide docker-compose configuration for local development
3. THE Gateway SHALL support configuration via environment variables
4. THE Gateway SHALL validate all required environment variables on startup
5. THE Gateway SHALL support graceful shutdown with connection draining
6. THE Gateway SHALL include health checks in the Docker configuration
7. THE Gateway SHALL document all required environment variables and their purposes

### Requirement 18: IP Whitelisting and Access Control

**User Story:** As a merchant, I want to restrict API access to specific IP addresses, so that I can enhance security for my integration.

#### Acceptance Criteria

1. WHEN a merchant configures IP whitelist entries, THE Gateway SHALL validate they are valid IP addresses or CIDR ranges
2. WHEN IP whitelisting is enabled for a merchant, THE Gateway SHALL reject requests from non-whitelisted IPs
3. WHEN a request is rejected due to IP restrictions, THE Gateway SHALL return a 403 status code
4. THE Gateway SHALL support up to 10 IP whitelist entries per merchant
5. THE Gateway SHALL allow merchants to disable IP whitelisting
6. THE Gateway SHALL log all rejected requests due to IP restrictions
7. WHEN IP whitelist is empty, THE Gateway SHALL allow requests from any IP address

### Requirement 19: Audit Logging

**User Story:** As a compliance officer, I want detailed audit logs of all system actions, so that I can track and investigate security incidents.

#### Acceptance Criteria

1. THE Gateway SHALL log all merchant account changes (creation, updates, deletions)
2. THE Gateway SHALL log all API_Key generation and rotation events
3. THE Gateway SHALL log all payment creation, confirmation, and refund events
4. THE Gateway SHALL log all webhook configuration changes
5. THE Gateway SHALL include merchant ID, timestamp, action type, and IP address in audit logs
6. THE Gateway SHALL retain audit logs for at least 90 days
7. THE Gateway SHALL provide an API endpoint for merchants to query their own audit logs

### Requirement 20: Partial Payments

**User Story:** As a merchant, I want to accept partial payments, so that customers can pay in multiple installments.

#### Acceptance Criteria

1. WHEN creating a payment, THE Gateway SHALL support enabling partial payment mode
2. WHEN partial payments are enabled, THE Gateway SHALL track the total amount paid across multiple transactions
3. WHEN a partial payment is received, THE Gateway SHALL update the remaining balance
4. WHEN the total paid amount reaches or exceeds the required amount, THE Gateway SHALL mark the payment as completed
5. THE Gateway SHALL send webhook notifications for each partial payment received
6. THE Gateway SHALL allow merchants to query the current paid amount and remaining balance
7. WHEN partial payments are enabled, THE Gateway SHALL extend expiration time with each payment received
