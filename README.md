# Crypto Payment Gateway Service

A standalone Rust-based cryptocurrency payment gateway that enables merchants to accept payments across multiple blockchains.

## Features

- **Multi-blockchain support**: Solana, BSC, Arbitrum, Polygon
- **Multiple cryptocurrencies**: SOL, USDT (on multiple networks)
- **RESTful API**: Complete API for payment management
- **Webhook notifications**: Real-time payment status updates
- **Hosted payment pages**: QR code payment pages
- **Sandbox mode**: Test integrations without real transactions
- **Fee-based revenue**: Configurable transaction fees
- **Analytics**: Comprehensive payment analytics and reporting

## Supported Payment Methods

1. **SOL** - Solana native token
2. **USDT (BEP20)** - USDT on Binance Smart Chain
3. **USDT (Arbitrum)** - USDT on Arbitrum One
4. **USDT (Polygon)** - USDT on Polygon
5. **USDT (SPL)** - USDT on Solana

## Technology Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL with sqlx
- **Cache**: Redis
- **Blockchain Integration**: Native RPC clients
- **Price Data**: Bybit API

## Project Structure

```
crypto-payment-gateway/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types
│   ├── models/              # Data models
│   │   ├── merchant.rs
│   │   ├── payment.rs
│   │   ├── webhook.rs
│   │   ├── refund.rs
│   │   └── analytics.rs
│   ├── services/            # Business logic
│   │   ├── merchant_service.rs
│   │   ├── payment_service.rs
│   │   ├── webhook_service.rs
│   │   ├── refund_service.rs
│   │   ├── analytics_service.rs
│   │   └── sandbox_service.rs
│   ├── api/                 # HTTP API
│   │   ├── routes.rs
│   │   ├── handlers.rs
│   │   └── state.rs
│   ├── middleware/          # HTTP middleware
│   │   ├── auth.rs
│   │   ├── rate_limit.rs
│   │   ├── ip_whitelist.rs
│   │   └── logging.rs
│   └── payment/             # Payment processing (from trading-bot)
│       ├── models.rs
│       ├── processor.rs
│       ├── verifier.rs
│       ├── blockchain_monitor.rs
│       ├── sol_monitor.rs
│       └── price_fetcher.rs
├── migrations/              # Database migrations
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## Getting Started

### Prerequisites

- Rust 1.75 or later
- PostgreSQL 15 or later
- Redis 7 or later

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd crypto-payment-gateway
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` with your configuration

4. Run database migrations:
```bash
cargo install sqlx-cli
sqlx migrate run
```

5. Build and run:
```bash
cargo build --release
cargo run --release
```

## Configuration

See `.env.example` for all available configuration options.

Key environment variables:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `SOLANA_RPC_URL` - Solana RPC endpoint
- `WEBHOOK_SIGNING_KEY` - Secret key for webhook signatures

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run property-based tests
cargo test --test property_tests

# Run with logging
RUST_LOG=debug cargo test
```

### Database Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## API Documentation

API documentation will be available at `/docs` when the server is running.

## License

[License information]

## Contributing

[Contributing guidelines]
