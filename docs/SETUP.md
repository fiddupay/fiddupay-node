# PayFlow - Setup Guide

**TechyTro Software**

## Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Redis 7+
- OpenSSL

## Installation

### 1. Clone and Build

```bash
git clone <repository>
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

### 3. Redis Setup

```bash
# Install Redis
# Ubuntu/Debian
sudo apt install redis-server

# macOS
brew install redis

# Start Redis
redis-server
```

### 4. Environment Configuration

```bash
cp .env.example .env
```

Edit `.env` with your settings:

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/payflow

# Redis
REDIS_URL=redis://localhost:6379

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Security Keys (GENERATE NEW ONES!)
ENCRYPTION_KEY=$(openssl rand -hex 32)
WEBHOOK_SIGNING_KEY=$(openssl rand -hex 32)

# Blockchain RPCs (use testnets for development)
SOLANA_RPC_URL=https://api.devnet.solana.com
BSC_RPC_URL=https://data-seed-prebsc-1-s1.binance.org:8545
ARBITRUM_RPC_URL=https://goerli-rollup.arbitrum.io/rpc
POLYGON_RPC_URL=https://rpc-mumbai.maticvigil.com

# Feature Flags
EMAIL_ENABLED=false  # Enable when SMTP configured
TWO_FACTOR_ENABLED=true
DEPOSIT_ADDRESS_ENABLED=true
WITHDRAWAL_ENABLED=true
INVOICE_ENABLED=true
MULTI_USER_ENABLED=true
```

### 5. Email Configuration (Optional)

For Gmail:
```bash
EMAIL_ENABLED=true
EMAIL_FROM=noreply@yourdomain.com
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_app_password
```

Generate Gmail app password: https://myaccount.google.com/apppasswords

### 6. Run Application

```bash
cargo run --release
```

Application starts on http://localhost:8080

## Verification

```bash
# Health check
curl http://localhost:8080/health

# Should return: {"status":"healthy"}
```

## Next Steps

1. Read [API Documentation](API.md)
2. Read [Merchant Guide](MERCHANT_GUIDE.md)
3. Test with [Postman Collection](../postman_collection.json)
4. Deploy with [Docker Guide](DEPLOYMENT.md)
