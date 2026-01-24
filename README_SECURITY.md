# PayFlow - Cryptocurrency Payment Gateway

## 🔒 Security Notice

This repository contains a cryptocurrency payment gateway. Please ensure:

1. **Never commit sensitive files:**
   - `.env` files containing API keys
   - Database files
   - Log files
   - Private keys or certificates

2. **Environment files to keep secure:**
   - `.env` - Development environment
   - `.env.production` - Production secrets
   - `server.log` - Runtime logs

3. **Use `.env.example` as template** for environment setup

## 🚀 Quick Start

### Backend (Rust)
```bash
# Copy environment template
cp .env.example .env

# Edit with your API keys
nano .env

# Build and run
cargo build --release
cargo run --release
```

### Frontend (React)
```bash
cd frontend
npm install
npm run dev
```

## 📁 Project Structure

```
crypto-payment-gateway/
├── src/                    # Rust backend source
├── frontend/              # React frontend
├── migrations/            # Database migrations
├── docs/                  # Documentation
├── .env.example          # Environment template
└── .gitignore            # Git exclusions
```

## ⚠️ Important Files to Never Commit

- `.env*` - Environment files with secrets
- `server.log` - Runtime logs
- `target/` - Rust build artifacts
- `frontend/node_modules/` - Node.js dependencies
- `*.db`, `*.sqlite` - Database files
- `*.log` - Log files
- `*.key`, `*.pem` - Private keys

## 🔐 Environment Variables

Required environment variables (see `.env.example`):
- `DATABASE_URL` - PostgreSQL connection
- `ENCRYPTION_KEY` - 32-byte hex key
- `WEBHOOK_SIGNING_KEY` - 32-byte hex key
- `ETHERSCAN_API_KEY` - Unified blockchain API key
- RPC URLs for supported blockchains

## 📚 Documentation

See `/docs/` directory for complete documentation.

---

**⚠️ Security Reminder**: Always review files before committing to ensure no sensitive data is included.
