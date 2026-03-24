# FidduPay Backend

This is the core API and worker service for FidduPay, an open-source cryptocurrency payment gateway.
It handles merchant authentication, payment generation, blockchain verification, webhooks, and the Wallet-as-a-Service (WaaS) features.

## Tech Stack
* **Language**: Rust
* **Web Framework**: Axum
* **Database**: PostgreSQL (via SQLx)
* **Serialization**: Serde
* **Async Runtime**: Tokio
* **Password Hashing**: Argon2

## Directory Structure
* `src/api/` - REST API routes and handlers.
* `src/models/` - Database and API request/response structs.
* `src/payment/` - Core payment logic, crypto types, and blockchain verification.
* `src/services/` - Business logic (Webhooks, WaaS, Sandbox, Wallets).
* `src/blockchain/` - Blockchain RPC interaction (EVM, Solana).
* `src/middleware/` - Authentication and rate-limiting middleware.
* `src/utils/` - Helpers (API Keys, Encryption, Validation).

## Running Locally

1. Set up your `.env` file referencing `.env.example`.
2. Apply database migrations:
   ```bash
   sqlx db create
   sqlx migrate run
   ```
3. Run the development server:
   ```bash
   cargo run
   ```
4. Reverify a transaction:
   ```bash
   curl -X POST http://localhost:3000/api/v1/admin/transactions/reverify \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer <YOUR_ADMIN_TOKEN>" \
      -d '{
         "hash": "4rEw1VBMhp8uFAYqo5NSBtjEjyMuSXQvH3GNdWMvr7RWMJCaJxE7hjaAgFMCRuUq4hTe5eCCJzGJXhE3Gcku9zno",
         "tx_type": "customer",
         "id": 3,
         "crypto_type": "SOL",
         "sandbox_mode": false
      }'
   ```

## Key Features
- **Live & Sandbox Modes**: Full mirror testing using Sepolia and Solana Devnet.
- **Settlement Modes**: Managed (custodial) or Forwarding (instant peer-to-peer payout).
- **Wallet-as-a-Service**: Provision, monitor, and sweep unique EVM/Solana wallets for your end users.
- **Webhooks**: Real-time HTTP and Discord notifications on payment states.
