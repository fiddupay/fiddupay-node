# PayFlow

**A TechyTro Software Product**

Modern cryptocurrency payment gateway for merchants. Accept payments across multiple blockchains with automatic forwarding, real-time notifications, and comprehensive merchant tools.

## Features

- Multi-blockchain support (Solana, BSC, Arbitrum, Polygon)
- Temporary deposit addresses (BitPay model)
- Automatic payment forwarding
- Multi-currency support (SOL, USDT on 4 networks)
- Invoice management
- Withdrawal system
- Balance tracking
- Email notifications
- Two-factor authentication
- Multi-user accounts with roles
- Comprehensive analytics
- Sandbox testing

## Quick Start

```bash
# 1. Install dependencies
cargo build --release

# 2. Setup database
createdb payflow
sqlx migrate run

# 3. Configure environment
cp .env.example .env
# Edit .env with your settings

# 4. Generate keys
openssl rand -hex 32  # ENCRYPTION_KEY
openssl rand -hex 32  # WEBHOOK_SIGNING_KEY

# 5. Run
cargo run --release
```

## API Documentation

- OpenAPI Spec: `openapi.yaml`
- Postman Collection: `postman_collection.json`
- Full Guide: `docs/API.md`

## Documentation

- [Setup Guide](docs/SETUP.md)
- [API Reference](docs/API.md)
- [Merchant Guide](docs/MERCHANT_GUIDE.md)
- [Testing Guide](docs/TESTING.md)
- [Deployment](docs/DEPLOYMENT.md)

## Technology Stack

- **Language:** Rust
- **Framework:** Axum
- **Database:** PostgreSQL
- **Cache:** Redis
- **Encryption:** AES-256-GCM
- **Blockchains:** Solana, BSC, Arbitrum, Polygon

## Architecture

PayFlow uses the BitPay deposit address model:
1. Generate unique temporary address per payment
2. Customer pays to temporary address
3. Gateway monitors and verifies payment
4. Automatically forwards to merchant wallet (minus fee)
5. Perfect tracking and reconciliation

## License

Copyright © 2026 TechyTro Software. All rights reserved.

## Support

- Email: support@techytro.com
- Documentation: https://docs.payflow.techytro.com
