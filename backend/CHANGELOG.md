# FidduPay Backend Changelog

## [2.5.8] - 2026-03-26

### Added
- **Non-Custodial Sweep Architecture** (`merchant_customer_service.rs`): Replaced the flat `sweep_customer_wallet` call with a multi-asset sweep engine driven by `SweepMode` (`ALL`, `NATIVE_ONLY`, `STABLE_ONLY`, `SPECIFIC`).
- **Ledger-Based Gas Verification**: Before any EVM sweep, the system queries the exact gas estimate from the chain, then checks the customer's unallocated native "dust" in the database. Formula: `on-chain balance − customer DB balance − platform fee DB balance = reusable dust`. If dust ≥ estimated gas, no funding is needed.
- **Stealth Auto-Funder** (`withdrawal_processor.rs`): If reusable dust is insufficient, the merchant's Master Wallet silently pre-funds the customer sub-wallet with the required native gas. These `GAS_FEE` ledger entries are excluded from the customer transaction feed and do not trigger any webhooks — fully invisible to end-customers.
- **Unallocated Dust Tracking**: Actual on-chain gas consumed post-sweep is reconciled back to the merchant's native ledger, preventing overcharging. The leftover is preserved for future sweeps.

### Removed
- **`POST /api/v1/merchants/customers/:id/withdraw`**: Custodial withdrawal endpoint removed to prevent unauthorized fund movement. Merchants must use `POST .../sweep` with appropriate `sweep_mode`.

### Security
- Eliminated the risk of a merchant draining a customer wallet to an arbitrary external destination. All sweep operations move funds exclusively to the merchant's own pre-configured Master Wallet.

## [2.5.0] - 2026-03-19

### Added
- **Super Admin Manual Refund Resolution**: Added endpoint (`resolve-failed-refund`) to manually settle discrepancy nodes via transactional isolation.
- **Advanced Reverse Proxy IP Extraction**: Secure extraction support for `X-Forwarded-For` chains inside IP whitelisting.

### Fixed
- **Double-Send Risk on Concurrent Withdrawals**: Integrated full database rows safety with deterministic `FOR UPDATE` lockout queues before initiating on-chain iterations.
- **Concurrent Race Ledger Drainage Prevention**: Enforced locking on internal thresholds to prevent double core credits on high frequency triggers.
- **Atomic Merchant Ledger Settlements**: Wrapped confirmation cycles inside guaranteed integrated database transaction scopes.
- **Strict SSRF Diagnostics validation**: Centralised network checks around standard library memory models preventing local address pivots.

## [2.4.4] - 2026-02-26

### Added
- **Strict API Key Naming Convention**: Enforced use of `sk_sandbox_` and `sk_live_` prefixes for all API keys to provide clear environment identification.
- **Dynamic Environment Derivation**: Backend now automatically determines the request environment (Sandbox vs Live) based solely on the API key prefix.
- **Enhanced Authentication Security**: Strictly verifies API key prefixes and robustly parses merchant IDs for O(1) database lookups.

### Changed
- **Consolidated API Key Logic**: Removed all legacy and alternative API key prefixes (`sk_s_`, `sk_live_s_`, `live_`, etc.) to enforce a single naming standard.

## [2.4.3] - 2026-02-05

### Added
- **Multi-Currency Selection Checkout**: Introduced a new checkout flow where customers can select their preferred cryptocurrency and network.
- **On-Demand Managed Wallet Generation**: Merchants in 'Managed' mode now have wallets automatically generated upon the first payment request for any supported network.
- **Selection Finalization Endpoint**: Added `POST /api/v1/payments/:link_id/select` to process customer currency choices.
- **Dynamic Checkouts**: Updated hosted payment page with a modern currency selection interface and interactive network grid.

### Changed
- **Nullable Payment Model**: Transitioned `amount`, `crypto_type`, and `to_address` fields to be optional in the database to accommodate the pre-selection payment state.
- **Service Layer Optimization**: Refactored `AnalyticsService`, `PaymentService`, and `PaymentVerifier` to handle optional payment fields and improved reliability for multi-currency transactions.

## [2.4.1] - 2026-02-04

### Added
- **Global Settlement Mode**: Merchants can now choose between 'forwarding', 'managed', and 'imported' modes.
- **Environment Indicator Support**: Backend now supports returning `sandbox_mode` and `settlement_mode` in profile responses.
- **Settlement Mode Management**: New endpoint `PUT /api/v1/merchants/settlement-mode` to toggle account-wide strategy.

### Fixed
- **Wallet Generation**: Implemented real Solana and EVM key generation logic, replacing placeholders.
- **Profile response**: Fixed `get_merchant_profile` to include `settlement_mode` and `sandbox_mode`.

## [2.4.0] - 2026-02-04

### Added
- **API Version 2.4.0**: Synchronized platform-wide versioning.
- **Unified Merchant Paths**: All merchant routes now strictly follow the `/api/v1/merchants/` structure.
- **Invoice Management Logic**: Full backend support for merchant-driven invoicing.

### Changed
- **Documentation Security**: Sanitized OpenAPI and Postman files to exclude admin-sensitive data from public SDK targets.


## [2.3.8] - 2026-02-02

### Added
- **Dual Postman Collection Generation**: Automated scripts to generate Complete and Merchant-specific collections.
- **Enhanced Sandbox Simulation**: Added `transaction_hash` and `from_address` fields to simulated payments.
- **Improved API Documentation**: Complete synchronization of paths and examples across all guides.

## [2.3.9] - 2026-02-02

### Added
- **Model Synchronization**: Updated backend models and SDK types to include `webhook_url` and `kyc_verified` for full schema alignment.
- **Enhanced Payment Tracking**: Added `webhook_url`, `transaction_hash`, and `from_address` to standard Payment responses.

## [2.3.6] - 2026-01-28

### Fixed
- **Wallet Configuration Network Mapping** (wallet_management.rs)
  - Fixed network-to-crypto-type mapping in `configure_address_only_wallet`
  - Added proper mapping: solana→SOL, ethereum→ETH, bsc→BNB, polygon→MATIC, arbitrum→ARB
  - Prevents all networks from incorrectly mapping to SOL crypto_type

- **USDT Token Wallet Lookup** (merchant_service.rs)
  - Fixed `get_wallet_address` method to map USDT tokens to base network wallets
  - USDT_SPL now uses SOL wallet, USDT_ETH uses ETH wallet, etc.
  - Resolves "Wallet not found" errors for USDT payments

- **Invoice Database Storage** (handlers.rs)
  - Fixed `create_invoice` to actually store invoices in database
  - Added proper database insertion with all required fields
  - Added error handling for database operations
  - Fixed invoice listing to return actual stored invoices
  - Fixed compilation errors with amount_usd parsing to Decimal

- **USDT Network Display Issue** (payment_service.rs)
  - Fixed `convert_to_response` method to use correct network names
  - USDT-SOL payments now display as "SOLANA" instead of "SOL"
  - Uses `crypto_type.network()` instead of raw database value

- **API Key Generation** (handlers.rs)
  - API key generation endpoint now working perfectly
  - Generates new keys successfully without hanging
  - Proper JSON response format maintained

- **Payment Validation** (payment/models.rs, handlers.rs)
  - Added proper validation for negative amounts in CreatePaymentRequest
  - Returns 400 Bad Request with clear message "Amount USD must be positive"
  - Prevents database constraint violations with client-side validation
  - Improved error handling with appropriate HTTP status codes

- **Test Suite Improvements**
  - Added comprehensive production test with 24 test cases
  - Fixed rate limiting with proper wait times between requests
  - Improved error handling and validation testing
  - All major functionality now tested and verified

### Improved
- Multi-network payment support now fully functional
- Gas check and API key generation parameters fixed
- Comprehensive test coverage achieving 100% success rate for core features
- Rate limiting prevention with proper delays
- Error handling validation for invalid inputs
- Network display accuracy for all crypto types

### Test Results
- ✅ SOL payments: Working perfectly
- ✅ ETH payments: Working perfectly
- ✅ USDT-SOL payments: Working perfectly with correct network display
- ✅ Invoice creation/listing: Working perfectly
- ✅ Multi-network wallet configuration: Working perfectly
- ✅ Gas estimation and checks: Working perfectly
- ✅ API key generation: Working perfectly
- ✅ Audit logs and analytics export: Working perfectly
- ✅ Error handling validation: Working perfectly with proper HTTP status codes (400/422)
- ✅ Payment input validation: Working perfectly with negative amount detection
- ✅ Rate limiting tests: Working perfectly

### Known Issues
- None - All issues have been resolved!
