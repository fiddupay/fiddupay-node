# FidduPay - Cryptocurrency Payment Gateway v2.6.0

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
sudo caddy fmt --overwrite /etc/caddy/Caddyfile
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl reload caddy
sudo systemctl restart caddy
sudo systemctl status caddy
# Start Caddy in the background
sudo caddy start

# Run Caddy in the foreground
sudo caddy run

# Reload Caddy config (zero downtime)
sudo caddy reload

# Stop Caddy
sudo caddy stop

# Format Caddyfile
caddy fmt --overwrite

 sudo nano /etc/caddy/Caddyfile

  find / -name "Caddyfile" 2>/dev/null
   sudo cat /etc/caddy/Caddyfile
   sudo systemctl cat caddy | grep -i caddyfile

   sudo journalctl -u fiddupay -f

   sudo journalctl -u fiddupay -n 100

   sudo journalctl -u fiddupay --since "227 minutes ago" --no-pager

   # Check API request logs
tail -f /var/log/caddy/fiddupay.api.log

# Check Payment Page request logs
tail -f /var/log/caddy/fiddupay.pay.log


sudo journalctl -u caddy --since "30 seconds ago" | grep "enabling automatic TLS"
```

### Deployment & Releases

#### Automated Release (Recommended)
This is a unified command that:
1.  Bumps versions in both the monorepo and SDK `package.json`.
2.  Commits and pushes changes to the main `fiddupay` repository.
3.  Synchronizes and tags the standalone SDK repository to trigger automated GitHub Actions and NPM publishing.

```bash
npm run release <version>  # Example: npm run release 2.4.1
```

#### Repository Sync Scripts
To sync specific code changes strictly to their respective GitHub repositories using PowerShell:

```powershell
# Sync core backend
.\scripts\push-backend.ps1 -Branch main

# Sync merchant dashboard (frontend)
.\scripts\push-frontend.ps1 -Branch main

# Sync P2P exchange frontend
.\scripts\push-p2p.ps1 -Branch main
```

#### Manual SDK Sync
To sync code changes to the standalone SDK repository without creating a release tag:

```bash
bash ./scripts/push-sdk.sh main
```

To sync code AND push a version tag manually (triggering the automated pipeline):

```bash
sudo bash ./scripts/push-sdk.sh main v2.4.6
cd fiddupay-node-sdk
npm publish --access public
```

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
- **Merchant-as-Custodian Security**: High-risk customer actions (withdraw, sweep, pay merchant) are authorized using the **Merchant's verified 4-digit Transaction PIN**. This eliminates the need for customers to manage security secrets while ensuring merchants retain full control over financial operations.
- **Financial Isolation (Sub-Accounts)**: Customer deposits into their designated wallets strictly update the customer's individual balance and do not fund the merchant's master wallet directly. This ensures clear accounting and platform-wide customer fund integrity.
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
