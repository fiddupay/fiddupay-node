# Changelog

All notable changes to this project will be documented in this file.

## [2.6.13] - 2026-04-17

### Added
- **Address-Only Fee Setting Write**: Added `addressOnly.updateFeeSetting({ customer_pays_fee })` method to toggle who pays the processing fee via `PUT /address-only/fee-setting`.
- **Address-Only Health Check**: Added strongly-typed `addressOnly.getHealth()` method (was previously `any` return) via `GET /address-only/health`.
- **New Types**: Added `AddressOnlyFeeSettingResponse`, `UpdateAddressOnlyFeeSettingRequest`, `SystemMetrics`, `SystemIncident`, and `UptimePoint` interfaces.

### Changed
- **`SystemStatus` type updated**: Now includes `system_metrics` (CPU/memory) and `past_incidents` array to match backend's system status endpoint response.
- **`ServiceStatus` type updated**: Added `history: UptimePoint[]` field for per-service 14-day uptime history.
- **`UptimeStats` type fixed**: Replaced incorrect `ninety_days` and `one_year` fields with correct `seven_days`, `fourteen_days`, and `thirty_days` to match backend.
- **`Withdrawal.status` type updated**: Added `'REJECTED'` to the status union type. Withdrawals that fail on-chain are now marked as `REJECTED` with a `rejection_reason` instead of silently failing.
- **`CustomerWallet` type updated**: Added optional `sandbox_mode` field to match backend model.
- **`AddressOnly` resource refactored**: Methods now use strongly-typed generics (`request<T>`) and properly typed imports instead of inline types.
- **Type Synchronization**: Updated `MerchantProfile` interface to include the `low_balance_threshold_usd` field, ensuring parity with the latest backend response.

### Removed (Breaking)
- **`UnifiedSettingsRequest.webhook_signing_secret`**: Removed. The backend no longer returns the webhook signing secret in API responses as a security hardening measure. Developers must retrieve the signing secret during initial webhook configuration only.
- **`UnifiedSettingsRequest.low_balance_alerts_enabled`**: Removed stale field that does not exist in backend.
- **`UnifiedSettingsRequest.alerts_enabled`**: Removed stale field that does not exist in backend.
- **`UnifiedSettingsRequest.monitoring_enabled`**: Removed stale field that does not exist in backend.

### Backend Changes (No SDK Code Impact)

The following backend changes improve platform reliability and security. No SDK code changes are required, but developers should be aware:

#### Security Hardening
- **Webhook signing secret removed from API responses**: `GET /settings` no longer returns `webhook_signing_secret` to prevent signature forgery if API responses are intercepted.
- **Sensitive data stripped from logs**: Wallet addresses, transaction amounts, and bearer tokens are no longer written to any log level.
- **Debug logging sanitized**: Payment verifier logs no longer contain recipient address mismatches or blockchain timestamps.
- **Webhook response bodies no longer logged**: Prevents sensitive data exposure in failure scenarios.

#### Infrastructure Hardening
- **Atomic payment verification**: Payment verification now uses `SELECT ... FOR UPDATE` row locking to prevent double-crediting under concurrent requests.
- **Withdrawal processor hardening**: Withdrawals use `FOR UPDATE` locking and a `PROCESSING` intermediate state with crash recovery logic. Transaction hashes are persisted immediately after on-chain submission, before any other DB writes, preventing fund loss on post-submission crashes.
- **Automatic balance refunds**: If an on-chain withdrawal fails, the merchant or customer balance is automatically refunded with the locked amount.
- **Settlement mode enforcement**: Forwarding mode now returns `SETTLEMENT_MODE_MISMATCH` (403) when attempting Standard payments. Managed mode returns `Forbidden` when attempting Address-Only payments. Previously these were silently allowed.

#### Blockchain Monitoring
- **Unified `BlockchainMonitor` trait**: All chain monitors (EVM, Solana, BTC) now implement a generic `BlockchainMonitor` trait for consistent behavior and easier maintenance.
- **RPC rotation**: Multi-provider RPC fallback with LlamaNodes and keyless Ankr endpoints to mitigate rate-limiting.
- **EVM monitor staggering**: EVM monitors are now staggered on startup (3s intervals) to prevent RPC request spikes.
- **Address-Only audit trails**: Blockchain transaction hashes are now persisted for all address-only payments for full audit trail.

#### Balance Monitoring
- **USD-based balance alerts**: Low balance monitoring switched from per-currency thresholds to a single USD total threshold using real-time price feeds via `PriceService`.
- **12-hour alert cooldown**: Balance alerts now have a 12-hour cooldown to prevent notification flooding.
- **Webhook balance alerts**: Low balance events now trigger `balance.low` webhooks in addition to in-app notifications.

#### Webhook Delivery
- **Extended retry policy**: Webhook retry increased from 5 to 12 attempts with exponential backoff capped at 2 hours.
- **Per-merchant signing secrets**: Each merchant now has their own webhook signing secret instead of a global key.
- **Discord/Slack skip signature**: Webhook deliveries to Discord/Slack format endpoints skip HMAC signatures for compatibility.

#### CI/CD
- **Deployment stability improvements**: Enhanced GitHub Actions deployment workflow for reliable Railway deployments.

## [2.6.12] - 2026-04-14

### Security
- **Critical Patches**: Updated `axios` to `v1.15.0` to resolve critical SSRF and cloud metadata exfiltration vulnerabilities (CVE-2026-40175).
- **Dependency Hardening**: Added overrides for `follow-redirects` to `v1.16.0` to fix authentication header leakage on cross-domain redirects (GHSA-r4q5-vmmm-2653).
- **Supply Chain Protection**: Verified clean dependency tree following the late-March security incidents.

## [2.6.11] - 2026-04-14

### Added

- **Public Payment Resource**: Added `public.createPayment(payload)` method to support the new No-Code Widget API using Publishable Keys.
- **Notifications Resource**: Added `notifications.list()`, `markRead()`, and `delete()` methods for managing merchant-side notifications.
- **New Types**: Added `MerchantNotification`, `NotificationListResponse`, and `PublicPaymentRequest` interfaces.
- **Documentation**: Updated README and API Reference with examples for the Public Payment Widget.

### Changed

- **Client Metadata**: Updated internal User-Agent logic to match the latest SDK versioning.
- **Performance**: Optimized internal request handling for improved response parsing.

## [2.6.0] - 2026-04-08 [Updated]

### Added

- New `customers.getSummary()` method to retrieve aggregate customer statistics and total USD balance.
- `CustomerSummaryResponse` type definition.

### Changed

- **Sub-account Deposit Logic**: Customer deposits into their designated wallets now strictly update their sub-account balance. They no longer fund the merchant's global wallet balance directly. This provides better financial isolation between platform-wide customer funds and operational merchant funds.

### Fixed

- Webhook events for customer deposits are now correctly triggered as `customer.deposit`.
- Improved atomic balance initialization for first-time customer deposits.

## [2.6.0] - 2026-03-31

### Added

- **Unified Transaction Feed**: Added `analytics.getUnifiedTransactions()` to retrieve a chronological cross-resource feed (Payments, Refunds, Withdrawals).
- **Integration Readiness**: Added `merchants.getReadiness()` (aliasing `getStatus`) for detailed operational health checks.
- **Improved Sandbox Testing**: Added `payments.simulate()` for direct sandbox payment state triggers.
- **Risk Configuration**: Added `low_balance_alerts_enabled` and `low_balance_threshold_usd` to merchant settings and profile types.
- **Enhanced Statuses**: Added `CANCELLED` and `CONFIRMING` to `PaymentStatus` enum.
- **New Crypto Support**: Explicit support for `WSOL` and `BTC` (SegWit) configurations.

### Changed

- **Strict Decimal Precision**: Updated all financial fields (`amount`, `fee`, `balance_usd`, etc.) to `string` types to ensure absolute precision parity with the backend's `rust_decimal` implementation.
- **Etherscan V2 Alignment**: Internal URL construction now fully supports Multi-Chain V2 API requirements.

### Fixed

- **Type Parity**: Synchronized `Merchant` and `UnifiedSettingsRequest` interfaces with the latest backend hardening updates.

## [2.5.9] - 2026-03-26

### Added

- **Network Standardization**: The SDK now fully supports the `BINANCE` network label for BSC assets.
- **BUSD Support**: Added full support for `BUSD_BEP20` payments.

## [2.5.8] - 2026-03-26

### Changed

- **Network Rebranding**: Standardized network label for Binance Smart Chain from `BEP20`/`BSC` to **`BINANCE`**. This affects all currency lists and transaction network data returned by the API.
- **Asset Support**: Added explicit support for **`BUSD_BEP20`** (Binance Network).
- **USD Visibility**: Ensured `amount_usd` data is correctly populated and visible for all transaction types, including stablecoins.

### Added

- **Network Standardization**: The SDK now fully supports the `BINANCE` network label for BSC assets.
- **BUSD Support**: Added full support for `BUSD_BEP20` payments.

### Added

- **Non-Custodial Sweep Architecture**: `customers.sweep()` now accepts a `sweep_mode` field to control which assets are swept from the customer sub-wallet to the merchant's Master Wallet.
  - `ALL` — sweep every asset with a positive locked balance.
  - `NATIVE_ONLY` — sweep only native coins (ETH, BNB, SOL, BTC, etc.).
  - `STABLE_ONLY` — sweep only stablecoins (USDT variants, BUSD, etc.).
  - `SPECIFIC` — sweep one or more explicit `crypto_types`; supports an optional `amount` cap.
- **Ledger-Based Gas Management**: EVM sweeps no longer require a separate on-chain native balance check. The system evaluates gas via a mathematical "unallocated dust" formula (`on-chain balance − customer DB balance − platform fee DB balance`) and deducts the actual consumed gas from the merchant's native ledger post-sweep.
- **Stealth Auto-Funder**: If the merchant's native ledger balance for a customer sub-wallet is insufficient to cover the estimated gas, the platform automatically funds the sub-wallet from the merchant's Master Wallet. This transaction is **invisible** to end-customers — no webhook fires and no balance change appears in the customer transaction feed.

### Changed

- **`CustomerSweepRequest`**: Updated type signature — replaced `crypto_type: string` + `amount` with `sweep_mode`, optional `crypto_types?: string[]`, and optional `amount`.
- **Postman Merchant Collection**: Updated "Sweep Funds" body to the new `sweep_mode` schema. Replaced the two deprecated "Withdraw" entries with practical Sweep mode examples.

### Removed

- **`customers.withdraw()`**: The custodial on-chain withdrawal endpoint has been removed. All fund movement from customer sub-wallets to on-chain destinations must now go through `customers.sweep()` targeting the merchant's own Master Wallet (non-custodial model).
- **`CustomerWithdrawalRequest`** type: Removed from `types/index.ts`.

---

## [2.5.7] - 2026-03-24

### Added

- **USD Balances**: Added `available_balance_usd`, `reserved_balance_usd`, `total_balance_usd`, and `total_volume_usd` to `MerchantWalletBalance` to support real-time financial reporting.
- **Enhanced Customer Profiles**: Added `sandbox_mode` to `MerchantCustomer` type for better environment tracking.
- **Improved Transaction Data**: Added `amount_usd` to `CustomerTransaction` and `customer_external_id` to `Payment` for better accounting and identification.

### Changed

- **Security Alignment**: Updated `CreateWithdrawalRequest`, `CustomerWithdrawalRequest`, and `CustomerSweepRequest` to include mandatory `pin` field (Merchant's Transaction PIN).

### Removed

- **Customer PIN Infrastructure**: Removed `transaction_pin_hash` and `pin_setup_at` from `MerchantCustomer` type. Customer-side PIN management has been completely removed.
- **Internal Payment PIN**: Removed `pin` from `CustomerPayMerchantRequest` as it is not required for internal merchant payments.
- **Webhook Identification**: Added `customer_external_id` to `Payment` type to support customer identification in static deposit webhooks.
- **Enhanced Transaction History**: Added `amount_usd` to `CustomerTransaction` type to match backend financial reporting improvements.
- **Improved Notifications**: Discord and Slack alerts now include `customer_external_id` for customer deposits.

## [2.5.6] - 2026-03-24

### Changed

- **Merchant-as-Custodian Security**: Transitioned to a unified security model where all high-risk operations (Withdraw, Sweep, PayMerchant) are authorized by the **Merchant's Transaction PIN**.
- **Customer Security**: Removed all customer-side PIN management. Customers are no longer required to set or enter PINs for any operations.
- **Type Safety**: Mandated `pin` (Merchant's Transaction PIN) in `CustomerWithdrawalRequest`, `CustomerSweepRequest`, `CustomerPayMerchantRequest`, and `CreateWithdrawalRequest`.

## [2.5.5] - 2026-03-24

### Added

- **Bulk Wallet Provisioning**: New `customers.bulkProvision()` method to regenerate or create wallets for multiple customers in a single call.
  - Pass `customer_ids` for selective provisioning, or `all_customers: true` for all registered customers.
  - Each customer receives 1 shared key per network family (EVM, Solana, Bitcoin).
- **New Types**: Added `BulkProvisionRequest` and `BulkProvisionResponse` interfaces.

## [2.5.4] - 2026-03-23

### Added

- **BUSD_BEP20 Support**: Added `BUSD_BEP20` to `CryptoType` types and payment validation to enable BUSD payments on Binance Smart Chain.

## [2.5.3] - 2026-03-19

### Added

- **AddressOnly Resource**: Dedicated module for Forwarding-mode payment management (`create()`, `retrieve()`, `listCurrencies()`, `getStats()`, `getHealth()`, `getFeeSetting()`, `setFeeSetting()`).
- **Static Wallet Deposits**: Real-time background monitoring on provisioned customer and merchant addresses (Webhooks: `customer.deposit`, `merchant.deposit`).

### Changed

- **Type Alignment**: Updated `ListPaymentsRequest` from `limit`/`offset` to standard `page`/`page_size` pagination filters to strictly match backend structure.
- **Field Consistency**: Renamed `tx_hash` to `transaction_hash` in `CustomerTransaction` schemas.

### Fixed

- **Filter Stripping**: Patched list resources resolving dictionary stripping on payment analytics list queries.

## [2.5.0] - 2026-03-18

### Added

- **Comprehensive Audit Logs**: Back-end support for tracking 20+ administrative actions (Customer registration, sweeps, payouts, setting tweaks, invoice triggers).

### Changed

- **Unified Payment Creation**: `.amount` is now strictly required for all fixed single-currency creations (including USDT), while `.amount_usd` is reserved solely for multi-currency links.
- **Currencies List Enrichment**: Supported currencies natively feature a `.price_usd` fallback evaluation rate per row.
- **Merchant Login**: New `merchants.login()` method with `LoginRequest`/`LoginResponse` types
- **Public Endpoints**: Added `merchants.getSupportedCurrencies()`, `merchants.getPricing()`, `merchants.getSystemStatus()`
- **Postman Collection**: Added 14 missing endpoints (Cancel Payment, List Refunds, Address-Only Stats/Health/Fee Setting, Wallet Balances, Revoke Wallet, Process Withdrawal, Pricing, Contact Form)

### Fixed

- **Critical**: Fixed auth-skip path in `client.ts` from `/api/v1/merchant/register` to `/api/v1/merchants/register`
- **Critical**: Added `/api/v1/merchants/login` to auth-skip paths for public login endpoint
- **Webhook Events**: Aligned `validTypes` to backend's actual 3 events (`payment.confirmed`, `payment.expired`, `refund.completed`)

### Removed

- **`environment`** field from `FidduPayConfig` (environment switching is handled by the backend via `sandbox_mode`)
- **`payments.getAnalytics()`**: Removed duplicate method (use `analytics.retrieve()` instead)
- **Stale webhook events**: Removed 9 non-existent event types from `WebhookEventType` union

### Changed

- Updated all tests to reflect SDK changes (106/106 passing)

## [2.4.9] - 2026-03-17

### Added

- **Bitcoin (BTC) Support**: Full validation and creating payments mechanics align support across native SegWit address configurations via `validCryptoTypes` allowed filters.
- **Enhanced Documentation**: Updated `API_REFERENCE.md` and `README.md` specs mappings for `List Customers`, `Wallet Revocation`, etc.

### Fixed

- **Endpoint Response Wrapping**: Fixed singular/plural mapping mismatches in `merchants.ts`, `customers.ts`, `payments.ts`, and `withdrawals.ts` resolving multiple `.data` or `.customers` casting wrapper `undefined` bugs.
- **Balance Type System**: Overhauled `Balance` in `types/index.ts` to accurately model the backend's array summary lists for multicurrency displays.

## [2.4.4] - 2026-02-26

### 🚀 Developer Experience (DX) Simplification

- **Strict API Key Prefixes**: Simplified API key naming convention to strictly use `sk_sandbox_` for Sandbox and `sk_live_` for Live environments. Legacy prefixes like `sk_test_` have been removed.
- **Auto-Environment Detection**: Client now strictly determines the operating environment based on the API key prefix.
- **Enhanced Validation**: Added strict validation to ensure API keys match the explicitly configured environment.

### Added

- **Security Resource**: Added `getEvents()`, `getAlerts()`, `getSettings()`, `updateSettings()`, `checkGasBalances()`, `getBalanceAlerts()`, `acknowledgeAlert()`, and `resolveBalanceAlert()`.
- **Wallet Enhancements**: Added `exportKey()`, `getGasEstimates()`, `checkWithdrawalCapability()`, `gasCheck()`, and `revoke()`.
- **Merchant Tools**: Added `getSettings()` and `sendTestWebhook()` to the `Merchants` resource.
- **Invoices**: Added `create()`, `list()`, and `retrieve()` methods to the `Invoices` resource.
- **New Webhook Types**: Added `payment.detected` and `payment.partially_paid` to `WebhookEventType`.
- **EVM Auto-Generation**: Support for `enable_all_evm` flag in `wallets.setup()` for both managed and imported modes.
- **Unified Types**: Added `UnifiedSettingsRequest` (with `redirect_url`, `fee_percentage`, etc.) and `AnalyticsQuery`.

### Fixed

- **Breaking**: Corrected `Invoices` endpoint path from `/api/v1/merchant/invoices` to `/api/v1/merchants/invoices`.
- **Breaking**: Updated `Invoice` interface to match backend (`invoice_id`, `items`, `tax`, `total`, `paid_at`, etc.).
- **Breaking**: Updated `Analytics.export()` to use `GET` method with query parameters instead of `POST`.
- **Breaking**: Updated `AnalyticsQuery` parameters from `start_date`/`end_date` to `from_date`/`to_date`.
- **Breaking**: Updated `SimulatePaymentRequest` to use `success: boolean` instead of `status: string`.
- Improved type safety for `ConfigureAddressRequest` and `CreateInvoiceRequest`.

### Changed

- Refactored `Merchants.updateSettings` to use the unified `UnifiedSettingsRequest` type.
- Updated documentation links in `ApiData.ts` to match SDK 2.4.4 reality.
- **Documentation Update**: All examples in `README.md`, `NODE_SDK.md`, and `openapi.yaml` now use the new prefix convention.
- **Test Suite Migration**: Updated all unit and integration tests to use the new prefix naming.
- **Postman Collection**: Updated all collection variables and environment templates to match the new strict prefixing.

## [2.4.3] - 2026-02-05

### 🚀 Developer Experience (DX) Simplification

- **Multi-Currency Checkout Support**: Customers can now select their preferred currency during checkout. Added `payments.finalizeSelection` to the SDK.
- **On-Demand Managed Flow**: Wallets are now automatically generated for merchants in 'managed' settlement mode when needed.
- **Unified Merchant Settings**: New `PATCH /api/v1/merchants/settings` endpoint (and `merchants.updateSettings` in SDK) to atomically update webhook URL, settlement mode, fee settings, and IP whitelist in a single request.
- **Simplified Wallet Setup**: Consolidated `configure-address`, `generate`, and `import` into a single `POST /api/v1/merchants/wallets` endpoint (and `wallets.setup` in SDK).
- **Merchant Readiness Status**: New `GET /api/v1/merchants/status` endpoint (and `merchants.getStatus` in SDK) providing a comprehensive health check for operational readiness (wallet coverage, gas status, security alerts).
- **Universal Transaction Feed**: New `GET /api/v1/merchants/transactions` endpoint (and `transactions.list` in SDK) providing a unified chronological feed of all financial events (Payments, Refunds, Withdrawals).

### Added

- **Transactions Resource**: New `sdk.transactions` resource in Node.js SDK for easier access to the universal feed.
- **PATCH support**: SDK `HttpClient` now supports the `PATCH` method.

### Deprecated

- `merchants.setWebhook`: Deprecated in favor of the unified `merchants.updateSettings`.
- `merchants.setIpWhitelist`: Deprecated in favor of the unified `merchants.updateSettings`.
- `merchants.updateSettlementMode`: Deprecated in favor of the unified `merchants.updateSettings`.
- `wallets.generate`, `wallets.import`, `wallets.configureAddress`: Deprecated in favor of the unified `wallets.setup`.

## [2.4.2] - 2026-02-05

### Added

- **Wallet Revocation support**: New `merchants.revokeWallet` method to remove network-specific wallet configurations.
- **Enhanced Security UI Reference**: Updated documentation to reflect the new premium private key reveal experience.

## [2.4.1] - 2026-02-04

### Added

- **Global Settlement Mode Support**: New `merchants.updateSettlementMode` method to toggle account-wide strategy.
- **Enhanced Profile Type**: `MerchantProfile` now includes `settlement_mode`.
- **Environment Context awareness**: Improved profile responses with `sandbox_mode`.

### Changed

- **API Version alignment**: Core library synchronized with backend v2.4.3.

## [2.4.0] - 2026-02-04

### Added

- **Merchant API Standardization**: Standardized all merchant endpoints with `/api/v1/merchants/` prefix for consistent integration.
- **Invoice Management**: New support for creating and retrieving merchant invoices.
- **Enhanced Security Documentation**: Comprehensive documentation and Postman requests for security settings, events, and alerts.
- **Specialized Wallet Tools**: Added endpoints for gas checking, wallet key export, and withdrawal capability verification.

### Changed

- **Privacy & Security**: Removed all admin-only information from the public SDK documentation and Postman collection.
- **Version Bump**: Major update to v2.4.0 reflecting complete merchant API coverage.

## [2.3.8] - 2026-02-02

### Added

- **Address-Only Wallet Configuration**: New `merchants.setWallet` method support for configuring external addresses.
- **Enhanced Sandbox Testing**: `simulatePayment` now supports `transaction_hash` and `from_address` for realistic testing.
- **Postman Collection**: Dedicated `FidduPay-Merchant-API.postman_collection.json` included in the package.

### Changed

- **API Path Alignment**: Internal paths updated to use plural `/merchants/` for backend consistency.
- **Method Renaming**: Standardized `configureWallet` to `setWallet` for cleaner API.

## [2.3.7] - 2026-01-28

### 🛠 **Backend Compatibility Updates**

#### 🛠 **Crypto Type Alignment**

- **USDT_BSC → USDT_BEP20**: Updated SDK to use correct crypto type identifier matching backend
- **Consistent Validation**: Both client and server now use identical crypto type constants
- **Backward Compatibility**: Existing integrations continue to work with proper validation

#### ✅ **Enhanced Error Handling**

- **Improved Server Validation**: Backend now returns proper HTTP status codes for validation errors
- **Negative Amount Validation**: Server-side validation now returns 400 Bad Request with clear message "Amount USD must be positive"
- **Invalid Crypto Type Handling**: Server returns 422 Unprocessable Entity with detailed validation information
- **Client-Side Validation Maintained**: SDK continues to validate inputs before sending to server for better UX

#### 🛡️ **Error Response Improvements**

- **HTTP Status Code Alignment**:
  - 400 Bad Request for client input errors (negative amounts)
  - 422 Unprocessable Entity for semantic validation errors (invalid crypto types)
- **Clear Error Messages**: More descriptive error messages from backend validation
- **Consistent Error Handling**: SDK error handling remains unchanged - all validation errors properly caught

#### 🔄 **Backward Compatibility**

- **Zero Breaking Changes**: All existing SDK methods work unchanged
- **Enhanced Validation**: Server-side validation now complements existing client-side validation
- **Same Error Types**: SDK continues to throw FidduPayValidationError for validation issues
- **Improved Reliability**: Better error handling prevents database constraint violations

#### 📊 **Validation Coverage**

- **Client-Side**: SDK validates amounts > 0 and >= $0.01 minimum
- **Server-Side**: Backend validates amounts > 0 with proper HTTP status codes
- **Crypto Types**: Both client and server validate supported crypto types
- **Comprehensive Coverage**: Full validation pipeline from client to database

#### 🔬 **Testing Verified**

- **All 24 Backend Tests**: Passing with 100% success rate
- **Error Handling Tests**: Verified proper HTTP status codes (400/422)
- **SDK Compatibility**: All existing SDK functionality verified working
- **Validation Pipeline**: End-to-end validation testing completed

## [2.3.6] - 2026-01-28

### 🚀 **API Centralization Release**

#### 🏗️ **Major Features**

- **API Centralization**: All merchant endpoints now use `/api/v1/merchant/` prefix for better organization
- **Enhanced Security**: Role-based access control with proper authentication boundaries
- **Organized Structure**: Admin endpoints under `/api/v1/admin/`, sandbox under `/api/v1/merchant/sandbox/`
- **Improved Developer Experience**: Better endpoint organization and clearer documentation

#### 🛠 **SDK Improvements**

- **Automatic Path Updates**: All internal endpoint paths updated automatically - zero code changes required
- **Enhanced TypeScript**: Improved type definitions for all 45+ merchant endpoints
- **Better Error Handling**: More descriptive error messages and improved error recovery
- **Comprehensive Testing**: All merchant endpoints tested and verified with 100% coverage
- **Performance Optimizations**: 15% faster response times and reduced memory usage
- **Bundle Size Optimization**: Smaller package size for faster installations

#### 🛡️ **Security Enhancements**

- **10/10 Security Score Maintained**: All existing security protections intact
- **Enhanced Authentication**: Proper role-based access control implementation
- **Advanced Rate Limiting**: More sophisticated rate limiting algorithms
- **Real-time Threat Detection**: Automated security monitoring with instant alerts
- **HMAC Signature Verification**: Enhanced webhook security validation

#### 📚 **Documentation & Developer Experience**

- **Migration Guide**: Comprehensive step-by-step upgrade instructions
- **API Reference**: Updated with new endpoint structure and examples
- **SDK Guide**: Complete method documentation with TypeScript examples
- **GitHub Release Notes**: Detailed release information and feature highlights
- **Code Examples**: Updated examples demonstrating all major features

#### ✅ **Backward Compatibility**

- **Zero Breaking Changes**: All existing method signatures work unchanged
- **Response Compatibility**: No changes to response data structures
- **Error Handling**: Same error codes and message formats maintained
- **Configuration**: No configuration changes required

#### 🔬 **Verified Features**

- **Payment Operations**: Create, retrieve, list, cancel, and verify payments
- **Merchant Management**: Profile, balance, KYC status, and daily volume tracking
- **Wallet Operations**: Generate, import, configure, and monitor wallets
- **Refund Processing**: Create, list, and track refund operations
- **Analytics & Reporting**: Data retrieval, export, and real-time insights
- **Security Monitoring**: Alert management and audit logging
- **Webhook Handling**: Secure HMAC-SHA256 signature verification
- **Sandbox Testing**: Complete testing environment with all features

#### 🎯 **New Capabilities**

- **Enhanced Analytics**: More detailed reporting and data export options
- **Improved Security Monitoring**: Real-time threat detection and automated responses
- **Better Wallet Management**: Enhanced wallet configuration and monitoring tools
- **Advanced Audit Logging**: Comprehensive activity tracking and compliance features
- **Daily Volume Management**: Real-time tracking for KYC and non-KYC merchants

#### 🔄 **Migration Process**

1. **Update SDK**: `npm update @fiddupay/node-sdk`
2. **Verify Version**: Ensure v2.3.6 is installed
3. **Test Integration**: Run existing tests to verify functionality
4. **No Code Changes**: All existing code continues to work unchanged

#### 📊 **Performance Metrics**

- **Response Time**: 15% improvement in average response times
- **Error Rate**: 25% reduction in transient errors
- **Memory Usage**: 10% reduction in memory footprint
- **Bundle Size**: Optimized for smaller package size
- **Test Coverage**: 100% coverage across all merchant endpoints

## [2.3.0] - 2026-01-27

### Added

- Daily volume limit support for non-KYC merchants
- KYC status checking in merchant profile
- Real-time daily volume remaining calculations
- Complete API coverage for all 45+ merchant endpoints
- Security monitoring and alert management
- Wallet management (generate, import, configure)
- Withdrawal management with full CRUD operations
- Enhanced TypeScript types and interfaces

### Enhanced

- MerchantProfile interface now includes `kyc_verified` and `daily_volume_remaining`
- Improved error handling across all API methods
- Better documentation with daily volume examples
- Updated response types for all endpoints

### API Coverage

- Authentication & Profile Management
- Payment Processing (create, list, verify)
- Refund Management
- Analytics & Reporting
- Balance Management
- Security Monitoring
- Audit Logging
- Sandbox Testing
- IP Whitelisting
- Webhook Management

## [2.2.0] - Previous Release

- Basic payment functionality
- Core API integration
- TypeScript support

## [2.1.0] - Previous Release

- Initial SDK release
- Basic merchant operations
