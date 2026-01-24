# PayFlow - Cryptocurrency Payment Gateway

**A TechyTro Software Product**

Modern, production-ready cryptocurrency payment gateway for merchants. Accept payments across multiple blockchains with automatic forwarding, real-time notifications, and comprehensive merchant tools.

## 🏗️ Monorepo Structure

```
payflow/
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

## 🚀 Quick Start

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

## 🔧 Configuration

### Backend (.env)
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/payflow
REDIS_URL=redis://localhost:6379
ENCRYPTION_KEY=your-32-byte-hex-key
SOLANA_RPC_URL=your-solana-rpc
ETHEREUM_RPC_URL=your-ethereum-rpc
```

### Frontend (.env.local)
```bash
VITE_API_URL=http://localhost:8080
```

## 📚 Documentation

- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Setup Guide](docs/SETUP.md)** - Development and production setup
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment guide

## 🔒 Security

PayFlow has achieved a **10/10 security score** with:
- XSS Prevention & CSRF Protection
- SQL Injection Protection
- Advanced Rate Limiting
- Real-time Threat Detection
- Account Lockout Protection

## 🌍 Supported Cryptocurrencies

- **SOL** (Solana)
- **USDT** on 5 networks: Ethereum, BSC, Polygon, Arbitrum, Solana

## 📄 License

Copyright © 2026 TechyTro Software. All rights reserved.
