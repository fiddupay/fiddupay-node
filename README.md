# PayFlow - Cryptocurrency Payment Gateway

**A TechyTro Software Product**

Modern, production-ready cryptocurrency payment gateway for merchants. Accept payments across multiple blockchains with automatic forwarding, real-time notifications, and comprehensive merchant tools.

## 🚀 Features

### Core Payment Processing
- **Multi-blockchain Support**: Solana, BSC, Arbitrum, Polygon, Ethereum
- **Multi-currency Support**: SOL, USDT on 5 networks (SOL, BSC, Polygon, Arbitrum, ETH)
- **Temporary Deposit Addresses**: BitPay-style unique addresses per payment
- **Automatic Payment Forwarding**: Direct to merchant wallets minus fees
- **Real-time Payment Verification**: Blockchain monitoring and confirmation

### Merchant Tools
- **Invoice Management**: Create and track invoices
- **Withdrawal System**: Automated cryptocurrency withdrawals
- **Balance Tracking**: Real-time balance with USD conversion
- **Analytics Dashboard**: Comprehensive payment analytics
- **Webhook Notifications**: Real-time payment status updates
- **API Key Management**: Secure key rotation

### Security & Compliance
- **Two-Factor Authentication**: TOTP-based 2FA
- **Multi-user Accounts**: Role-based access control
- **IP Whitelisting**: Restrict API access by IP
- **Rate Limiting**: DDoS protection
- **Audit Logging**: Complete activity tracking
- **Sandbox Testing**: Safe testing environment

## 🏗️ Architecture

PayFlow uses the **BitPay deposit address model**:

1. **Generate** unique temporary address per payment
2. **Monitor** blockchain for incoming payments
3. **Verify** payment amount and confirmations
4. **Forward** funds to merchant wallet (minus fee)
5. **Notify** merchant via webhook
6. **Track** complete payment lifecycle

## 🛠️ Technology Stack

- **Language**: Rust 🦀
- **Framework**: Axum (async web framework)
- **Database**: PostgreSQL with SQLX
- **Cache**: Redis
- **Encryption**: AES-256-GCM
- **Authentication**: Argon2 password hashing
- **Blockchains**: Solana RPC, Ethereum JSON-RPC

## 📋 Prerequisites

- Rust 1.70+
- PostgreSQL 13+
- Redis 6+
- OpenSSL

## 🚀 Quick Start

### 1. Clone and Build
```bash
git clone <repository-url>
cd crypto-payment-gateway
cargo build --release
```

### 2. Database Setup
```bash
# Create database
createdb payflow

# Run migrations
sqlx migrate run
```

### 3. Environment Configuration
```bash
# Copy example environment file
cp .env.example .env

# Generate encryption keys
openssl rand -hex 32  # ENCRYPTION_KEY
openssl rand -hex 32  # WEBHOOK_SIGNING_KEY

# Edit .env with your settings
nano .env
```

### 4. Start Services
```bash
# Start Redis
redis-server

# Start PayFlow
cargo run --release
```

### 5. Test Installation
```bash
# Health check
curl http://localhost:8080/health

# Register merchant
curl -X POST http://localhost:8080/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{"business_name":"Test Business","email":"test@example.com","password":"password123"}'
```

## 📚 Documentation

- **[Complete Documentation Index](docs/DOCUMENTATION_INDEX.md)** - Navigation guide for all documentation
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Setup Guide](docs/SETUP.md)** - Development and production setup
- **[Merchant Guide](docs/MERCHANT_GUIDE.md)** - Integration guide for merchants
- **[Testing Guide](docs/TESTING.md)** - Testing procedures and guidelines
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment guide
- **[Project Structure](docs/PROJECT_STRUCTURE.md)** - Code organization and architecture
- **[Project Status](docs/PROJECT_STATUS.md)** - Current achievements and metrics
- **[Roadmap](docs/ROADMAP.md)** - Future features and development plans

## 🔧 Configuration

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string | ✅ | - |
| `REDIS_URL` | Redis connection string | ✅ | `redis://localhost:6379` |
| `ENCRYPTION_KEY` | 32-byte hex encryption key | ✅ | - |
| `WEBHOOK_SIGNING_KEY` | 32-byte hex webhook signing key | ✅ | - |
| `SOLANA_RPC_URL` | Solana RPC endpoint | ✅ | - |
| `ETHEREUM_RPC_URL` | Ethereum RPC endpoint | ✅ | - |
| `BSC_RPC_URL` | BSC RPC endpoint | ✅ | - |
| `POLYGON_RPC_URL` | Polygon RPC endpoint | ✅ | - |
| `ARBITRUM_RPC_URL` | Arbitrum RPC endpoint | ✅ | - |

### Supported Cryptocurrencies

| Currency | Network | Contract Address | Confirmations |
|----------|---------|------------------|---------------|
| SOL | Solana | Native | 32 |
| USDT_SOL | Solana | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 32 |
| USDT_ETH | Ethereum | `0xdAC17F958D2ee523a2206206994597C13D831ec7` | 12 |
| USDT_BSC | BSC | `0x55d398326f99059fF775485246999027B3197955` | 15 |
| USDT_POLYGON | Polygon | `0xc2132D05D31c914a87C6611C10748AEb04B58e8F` | 30 |
| USDT_ARBITRUM | Arbitrum | `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9` | 1 |

## 🔌 API Overview

### Authentication
```bash
# All API requests require Bearer token
Authorization: Bearer <api_key>
```

### Core Endpoints

#### Create Payment
```bash
POST /api/v1/payments
{
  "amount_usd": "100.00",
  "crypto_type": "USDT_ETH",
  "description": "Order #12345"
}
```

#### Configure Wallet
```bash
PUT /api/v1/merchants/wallets
{
  "crypto_type": "USDT_ETH",
  "address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87"
}
```

#### Set Webhook
```bash
PUT /api/v1/merchants/webhook
{
  "url": "https://your-site.com/webhook"
}
```

## 🧪 Testing

### Run Test Suite
```bash
# Run complete test suite
./tests/run_tests.sh

# Run specific test categories
./tests/run_tests.sh --unit          # Unit tests only
./tests/run_tests.sh --integration   # Integration tests only
./tests/run_tests.sh --api          # API tests only
./tests/run_tests.sh --scripts      # Test scripts only

# Individual cargo tests
cargo test                           # All Rust tests
cargo test --test payment_test       # Specific test file
```

### Test Structure
- **Unit Tests**: `tests/unit/` - Individual component tests
- **Integration Tests**: `tests/integration/` - Service interaction tests
- **API Tests**: `tests/api/` - HTTP endpoint tests
- **Test Scripts**: `tests/scripts/` - Bash test scripts
- **Master Runner**: `tests/run_tests.sh` - Comprehensive test runner

## 🚀 Deployment

### Docker Deployment
```bash
# Build image
docker build -t payflow .

# Run with docker-compose
docker-compose up -d
```

### Production Checklist
- [ ] Set strong encryption keys
- [ ] Configure SSL/TLS certificates
- [ ] Set up database backups
- [ ] Configure monitoring and logging
- [ ] Set up rate limiting
- [ ] Configure IP whitelisting
- [ ] Test webhook endpoints
- [ ] Verify blockchain RPC endpoints

## 📊 Monitoring

### Health Endpoints
- `GET /health` - Service health check
- `GET /api/v1/analytics` - Payment analytics
- `GET /api/v1/balance` - Account balances

### Logging
PayFlow provides structured logging with:
- Request/response logging
- Payment processing events
- Webhook delivery status
- Error tracking and alerts

## 🔒 Security

### Best Practices
- Use strong, unique encryption keys
- Enable 2FA for all accounts
- Implement IP whitelisting
- Regular API key rotation
- Monitor webhook signatures
- Keep dependencies updated

### Compliance
- PCI DSS considerations
- GDPR compliance for EU users
- AML/KYC integration ready
- Audit trail maintenance

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

Copyright © 2026 TechyTro Software. All rights reserved.

## 🆘 Support

- **Email**: support@techytro.com
- **Documentation**: https://docs.payflow.techytro.com
- **Status Page**: https://status.payflow.techytro.com

## 🗺️ Roadmap

### Q1 2026
- [ ] Lightning Network support
- [ ] Mobile SDK release
- [ ] Advanced analytics dashboard

### Q2 2026
- [ ] Multi-signature wallet support
- [ ] Automated compliance reporting
- [ ] Enhanced fraud detection

---

**Built with ❤️ by TechyTro Software**
