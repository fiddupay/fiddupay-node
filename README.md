# FidduPay - Cryptocurrency Payment Gateway v2.3.8

**A TechyTro Software Product**

Modern, production-ready cryptocurrency payment gateway for merchants. Accept payments across multiple blockchains with automatic forwarding, real-time notifications, and comprehensive merchant tools.

## Monorepo Structure

```
fiddupay/
  backend/           # Rust backend API
  frontend/          # React frontend
  fiddupay-node-sdk/ # Official Node.js SDK (Mirrored to standalone repo)
  docs/              # System documentation
  scripts/           # Automation scripts (push-sdk.sh, etc.)
  package.json       # Monorepo scripts
```

> [!NOTE]
> The `fiddupay-node-sdk` folder is the source of truth for the [@fiddupay/fiddupay-node](https://github.com/fiddupay/fiddupay-node) standalone repository. Use `./scripts/push-sdk.sh` to sync changes.

## Quick Start

### Prerequisites
- Rust 1.70+
- Node.js 18+
- PostgreSQL 13+
- Redis 6+

### Development

```bash
# Install dependencies
npm run install:frontend
npm run install:backend

# Start both backend and frontend
npm run dev

# Or start individually
npm run dev:backend    # Rust API server
npm run dev:frontend   # React dev server
```

### Production Build

```bash
# Build backend
npm run build:backend

# Build frontend
npm run build:frontend
```

### Caddy Server (Reverse Proxy)

This project uses Caddy as a reverse proxy and file server. The `Caddyfile` is located in the project root.

```bash
# Check status
sudo systemctl status fiddupay
sudo systemctl restart fiddupay
# Start Caddy in the background
caddy start

# Run Caddy in the foreground
caddy run

# Reload Caddy config (zero downtime)
caddy reload

# Stop Caddy
caddy stop

# Format Caddyfile
caddy fmt --overwrite
```

### SDK Deployment

To deploy a new version of the Node.js SDK to npm:

1. **Sync Changes**: Run `./scripts/push-sdk.sh` to sync the monorepo folder with the standalone repo.
2. **Publish**:
   ```bash
   cd fiddupay-node-sdk
   npm publish --access public
   npm view @fiddupay/fiddupay-node version
   npm view @fiddupay/fiddupay-node time
   ```
   *Note: Ensure you are logged into npm (`npm login`) before publishing.*

## Configuration

### Backend (.env)
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/fiddupay
REDIS_URL=redis://localhost:6379
ENCRYPTION_KEY=your-32-byte-hex-key
SOLANA_RPC_URL=your-solana-rpc
ETHEREUM_RPC_URL=your-ethereum-rpc
```

### Frontend (.env.local)
```bash
VITE_API_URL=http://localhost:8080
```

## Documentation

- **[Platform Roadmap](ROADMAP.md)** - Strategic roadmap for dual-tier platform (Personal + Business)
- **[Node.js SDK Guide](docs/NODE_SDK.md)** - Complete Node.js SDK development documentation
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **Postman Collections**:
  - [Merchant API (SDK)](fiddupay-node-sdk/postman/FidduPay-Merchant-API.postman_collection.json) - For standard integration
  - [Complete API](docs/postman/FidduPay-Complete-API.postman_collection.json) - Full system reference (includes Admin)
- **[Setup Guide](docs/SETUP.md)** - Development and production setup
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment guide

## Security

FidduPay has achieved a **10/10 security score** with:
- XSS Prevention & CSRF Protection
- SQL Injection Protection
- Advanced Rate Limiting
- Real-time Threat Detection
- Account Lockout Protection
- **Real-Time Webhooks**: Instant transaction detection (0-conf) via WebSocket
- **Partial Payment Support**: Smart handling of underpayments with automated notifications

## Supported Cryptocurrencies

**5 Major Blockchain Networks:**
- **Solana** - SOL + USDT (SPL)
- **Ethereum** - ETH + USDT (ERC-20)
- **Binance Smart Chain** - BNB + USDT (BEP-20)
- **Polygon** - MATIC + USDT
- **Arbitrum** - ARB + USDT

**Total: 10 cryptocurrency options across 5 blockchains**

## License

Copyright © 2026 TechyTro Software. All rights reserved.
