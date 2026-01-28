# fiddupay - Project Structure

This document outlines the clean, organized structure of the fiddupay cryptocurrency payment gateway codebase.

##  Directory Structure

```
crypto-payment-gateway/
  src/                          # Main source code
     api/                      # HTTP API layer
       handlers.rs              # Request handlers
       routes.rs                # Route definitions
       state.rs                 # Application state
       mod.rs                   # Module exports
   
     services/                 # Business logic layer
       merchant_service.rs      # Merchant management
       payment_service.rs       # Payment processing
       balance_service.rs       # Balance management
       withdrawal_service.rs    # Withdrawal processing
       webhook_service.rs       # Webhook notifications
       analytics_service.rs     # Analytics & reporting
       audit_service.rs         # Audit logging
       email_service.rs         # Email notifications
       two_factor_service.rs    # 2FA authentication
       multi_user_service.rs    # Multi-user management
       sandbox_service.rs       # Sandbox testing
       ip_whitelist_service.rs  # IP whitelisting
       invoice_service.rs       # Invoice management
       refund_service.rs        # Refund processing
       deposit_address_service.rs # Address generation
       price_cache_service.rs   # Price caching
       mod.rs                   # Service exports
   
     payment/                  # Payment processing core
       models.rs                # Payment data models
       processor.rs             # Payment processor
       verifier.rs              # Payment verification
       blockchain_monitor.rs    # Blockchain monitoring
       sol_monitor.rs           # Solana monitoring
       price_fetcher.rs         # Price fetching
       fee_calculator.rs        # Fee calculations
       mod.rs                   # Payment exports
   
     models/                   # Data models
       merchant.rs              # Merchant models
       payment.rs               # Payment models
       analytics.rs             # Analytics models
       webhook.rs               # Webhook models
       refund.rs                # Refund models
       mod.rs                   # Model exports
   
     middleware/               # HTTP middleware
       auth.rs                  # Authentication
       logging.rs               # Request logging
       rate_limit.rs            # Rate limiting
       ip_whitelist.rs          # IP filtering
       mod.rs                   # Middleware exports
   
     utils/                    # Utility functions
       encryption.rs            # Encryption utilities
       keygen.rs                # Key generation
       retry.rs                 # Retry logic
       circuit_breaker.rs       # Circuit breaker
       mod.rs                   # Utility exports
   
    main.rs                      # Application entry point
    config.rs                    # Configuration management
    error.rs                     # Error handling
    feature_flags.rs             # Feature flags
    background_tasks.rs          # Background processing
    lib.rs                       # Library root

  tests/                        # Integration tests
    api_endpoints_test.rs        # API endpoint tests
    payment_test.rs              # Payment flow tests
    withdrawal_test.rs           # Withdrawal tests
    services_test.rs             # Service layer tests
    utils_test.rs                # Utility tests
    workflows_test.rs            # End-to-end workflows
    complete_endpoint_test.rs    # Complete API tests
    comprehensive_service_test.rs # Service integration
    database_integration_test.rs # Database tests
    endpoints_test.rs            # Endpoint validation
    full_integration_test.rs     # Full integration
    payment_listing_tests.rs    # Payment listing
    analytics_service_tests.rs  # Analytics tests
    standalone_tests.rs         # Standalone tests

  docs/                         # Documentation
    API.md                       # Original API docs
    API_REFERENCE.md             # Comprehensive API reference
    SETUP.md                     # Setup guide
    MERCHANT_GUIDE.md            # Merchant integration
    TESTING.md                   # Testing guide
    DEPLOYMENT.md                # Deployment guide
    SECURITY.md                  # Security guide

  migrations/                   # Database migrations
    001_initial.sql              # Initial schema
    002_add_webhooks.sql         # Webhook tables
    ...                         # Additional migrations

  scripts/                      # Utility scripts
    test.sh                      # Test runner
    setup_infrastructure.sh     # Infrastructure setup
    security_audit.sh           # Security audit
    run_tests.sh                 # Test execution

  Configuration Files
 Cargo.toml                       # Rust dependencies
 Cargo.lock                       # Dependency lock file
 .env.example                     # Environment template
 .env                             # Environment variables
 .gitignore                       # Git ignore rules
 README.md                        # Project overview
 FINAL_FIXES_SUMMARY.md           # Fix summary
 PROJECT_STRUCTURE.md             # This file

  Test Scripts
 test_api.sh                      # API testing
 test_basic_api.sh                # Basic API tests
 test_complete_flow.sh            # Complete flow tests
 test_service_layer.sh            # Service tests
 test_final_complete.sh           # Final tests
 test_redis.sh                    # Redis tests
 fix_build.sh                     # Build fixes
```

##  Architecture Layers

### 1. **API Layer** (`src/api/`)
- **Purpose**: HTTP request/response handling
- **Components**:
  - `handlers.rs` - Request handlers for all endpoints
  - `routes.rs` - Route definitions and middleware setup
  - `state.rs` - Application state management
- **Responsibilities**:
  - Request validation
  - Response formatting
  - Authentication middleware
  - Rate limiting

### 2. **Service Layer** (`src/services/`)
- **Purpose**: Business logic implementation
- **Components**:
  - Core services (merchant, payment, balance, withdrawal)
  - Supporting services (webhook, analytics, audit)
  - Security services (2FA, IP whitelist, sandbox)
- **Responsibilities**:
  - Business rule enforcement
  - Data validation
  - External API integration
  - Transaction management

### 3. **Payment Core** (`src/payment/`)
- **Purpose**: Cryptocurrency payment processing
- **Components**:
  - Payment models and data structures
  - Blockchain monitoring and verification
  - Fee calculation and price fetching
- **Responsibilities**:
  - Payment lifecycle management
  - Blockchain interaction
  - Price and fee calculations
  - Payment verification

### 4. **Data Layer** (`src/models/`)
- **Purpose**: Data models and structures
- **Components**:
  - Database models
  - API request/response models
  - Business domain models
- **Responsibilities**:
  - Data serialization/deserialization
  - Database mapping
  - Type safety

### 5. **Middleware Layer** (`src/middleware/`)
- **Purpose**: Cross-cutting concerns
- **Components**:
  - Authentication and authorization
  - Request logging
  - Rate limiting
  - IP whitelisting
- **Responsibilities**:
  - Security enforcement
  - Request/response processing
  - Monitoring and logging

### 6. **Utility Layer** (`src/utils/`)
- **Purpose**: Common utilities and helpers
- **Components**:
  - Encryption utilities
  - Key generation
  - Retry logic
  - Circuit breaker
- **Responsibilities**:
  - Reusable functionality
  - Error handling patterns
  - Performance optimizations

##  Key Design Principles

### 1. **Separation of Concerns**
- Each layer has a specific responsibility
- Clear boundaries between components
- Minimal coupling between layers

### 2. **Dependency Injection**
- Services injected through application state
- Easy testing and mocking
- Flexible configuration

### 3. **Error Handling**
- Centralized error types in `error.rs`
- Consistent error responses
- Proper error propagation

### 4. **Security First**
- Authentication middleware on all protected routes
- Input validation at API layer
- Secure key management

### 5. **Testability**
- Comprehensive test coverage
- Integration and unit tests
- Test utilities and helpers

##  Data Flow

```
HTTP Request
    ↓
API Layer (handlers.rs)
    ↓
Middleware (auth, logging, rate limit)
    ↓
Service Layer (business logic)
    ↓
Payment Core (blockchain interaction)
    ↓
Database/External APIs
    ↓
Response (JSON)
```

##  Background Processing

### Background Tasks (`background_tasks.rs`)
- **Webhook Retries**: Retry failed webhook deliveries
- **Payment Expiration**: Mark expired payments
- **Cleanup Tasks**: Remove old data

### Task Scheduling
- Tokio-based async task scheduling
- Configurable intervals
- Error handling and logging

##  Database Schema

### Core Tables
- `merchants` - Merchant accounts
- `payments` - Payment transactions
- `balances` - Account balances
- `withdrawals` - Withdrawal requests
- `webhooks` - Webhook configurations
- `audit_logs` - Audit trail

### Supporting Tables
- `deposit_addresses` - Temporary addresses
- `merchant_wallets` - Wallet configurations
- `webhook_deliveries` - Webhook delivery tracking
- `refunds` - Refund transactions

##  Testing Strategy

### Test Categories
1. **Unit Tests** - Individual function testing
2. **Integration Tests** - Service interaction testing
3. **API Tests** - HTTP endpoint testing
4. **End-to-End Tests** - Complete workflow testing

### Test Organization
- Tests mirror source structure
- Shared test utilities
- Database test fixtures
- Mock external services

##  Performance Considerations

### Caching
- Redis for session data
- Price caching for crypto rates
- Database query optimization

### Async Processing
- Tokio async runtime
- Non-blocking I/O operations
- Background task processing

### Database Optimization
- Proper indexing
- Connection pooling
- Query optimization

##  Security Features

### Authentication & Authorization
- API key-based authentication
- Role-based access control
- IP whitelisting

### Data Protection
- AES-256-GCM encryption
- Secure key storage
- Input sanitization

### Audit & Monitoring
- Comprehensive audit logging
- Request/response logging
- Error tracking

##  Deployment

### Production Readiness
- Environment-based configuration
- Health check endpoints
- Graceful shutdown handling
- Error monitoring

### Scalability
- Stateless design
- Database connection pooling
- Horizontal scaling support

This structure ensures maintainability, testability, and scalability while following Rust best practices and clean architecture principles.
