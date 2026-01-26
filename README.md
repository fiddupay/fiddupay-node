# FidduPay - Cryptocurrency Payment Gateway

**A TechyTro Software Product**

Modern, production-ready cryptocurrency payment gateway for merchants. Accept payments across multiple blockchains with automatic forwarding, real-time notifications, and comprehensive merchant tools.

##  Monorepo Structure

```
fiddupay/
├── backend/          # Rust backend API
│   ├── src/         # Rust source code
│   ├── Cargo.toml   # Rust dependencies
│   └── migrations/  # Database migrations
├── frontend/         # React frontend
│   ├── src/         # React source code
│   ├── package.json # Frontend dependencies
│   └── dist/        # Build output
└── package.json     # Monorepo scripts
```

##  Quick Start

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

##  Configuration

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

##  Documentation

- **[Platform Roadmap](ROADMAP.md)** - Strategic roadmap for dual-tier platform (Personal + Business)
- **[Node.js SDK Guide](docs/NODE_SDK.md)** - Complete Node.js SDK development documentation
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Setup Guide](docs/SETUP.md)** - Development and production setup
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment guide

##  Security

FidduPay has achieved a **10/10 security score** with:
- XSS Prevention & CSRF Protection
- SQL Injection Protection
- Advanced Rate Limiting
- Real-time Threat Detection
- Account Lockout Protection

##  Supported Cryptocurrencies

**5 Major Blockchain Networks:**
- **Solana** - SOL + USDT (SPL)
- **Ethereum** - ETH + USDT (ERC-20)
- **Binance Smart Chain** - BNB + USDT (BEP-20)
- **Polygon** - MATIC + USDT
- **Arbitrum** - ARB + USDT

**Total: 10 cryptocurrency options across 5 blockchains**

##  License

Copyright © 2026 TechyTro Software. All rights reserved.
