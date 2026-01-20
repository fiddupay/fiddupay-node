# Setup Instructions - Crypto Payment Gateway (Staging)

## Prerequisites

1. **Rust** (1.75 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **PostgreSQL** (15 or later)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install postgresql postgresql-contrib
   
   # macOS
   brew install postgresql@15
   ```

3. **Redis** (7 or later)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install redis-server
   
   # macOS
   brew install redis
   ```

4. **sqlx-cli** (for migrations)
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

---

## Step-by-Step Setup

### 1. Database Setup

```bash
# Start PostgreSQL (if not running)
sudo systemctl start postgresql  # Linux
brew services start postgresql@15  # macOS

# Create database
createdb crypto_gateway_staging

# Or using psql:
psql -U postgres
CREATE DATABASE crypto_gateway_staging;
\q
```

### 2. Redis Setup

```bash
# Start Redis
sudo systemctl start redis  # Linux
brew services start redis  # macOS

# Verify Redis is running
redis-cli ping
# Should return: PONG
```

### 3. Environment Configuration

The `.env` file has been created with staging defaults. **Update these values:**

```bash
# Edit .env file
nano .env  # or use your preferred editor

# REQUIRED: Update database credentials
DATABASE_URL=postgres://YOUR_USERNAME:YOUR_PASSWORD@localhost:5432/crypto_gateway_staging

# RECOMMENDED: Generate secure webhook signing key
# Run this command and copy the output:
openssl rand -hex 32

# Then update in .env:
WEBHOOK_SIGNING_KEY=<paste_generated_key_here>

# OPTIONAL: Update payment page URL if deploying to server
PAYMENT_PAGE_BASE_URL=http://your-server-ip:8080
```

### 4. Run Database Migrations

```bash
# Run migrations
sqlx migrate run

# Verify migrations
sqlx migrate info
```

### 5. Build the Application

```bash
# Build in release mode
cargo build --release

# Or for development (faster compile, slower runtime)
cargo build
```

### 6. Run the Application

```bash
# Development mode
cargo run

# Or production mode
cargo run --release

# You should see:
# 🚀 Starting Crypto Payment Gateway Service
# ✅ Configuration loaded
# ✅ Database connected
# ✅ Migrations complete
# ✅ Application state initialized
# ✅ Background tasks started
# ✅ Server listening on http://0.0.0.0:8080
```

---

## Verification

### 1. Health Check

```bash
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy"}
```

### 2. Register a Merchant

```bash
curl -X POST http://localhost:8080/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "business_name": "Test Business"
  }'

# Expected response:
# {
#   "merchant_id": 1,
#   "api_key": "abc123..."
# }
```

### 3. Create a Payment

```bash
# Save the API key from step 2
API_KEY="<your_api_key_here>"

curl -X POST http://localhost:8080/api/v1/payments \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "amount_usd": 100.00,
    "crypto_type": "SOL",
    "description": "Test payment"
  }'

# Expected response:
# {
#   "payment_id": "pay_...",
#   "status": "PENDING",
#   "amount": "...",
#   "payment_link": "http://localhost:8080/pay/lnk_...",
#   ...
# }
```

### 4. Visit Payment Page

Open the `payment_link` from step 3 in your browser. You should see a payment page with:
- Payment amount in crypto and USD
- QR code
- Countdown timer
- Payment address

---

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test payment_listing_tests

# Run with output
cargo test -- --nocapture

# Run tests in release mode (faster)
cargo test --release
```

---

## Troubleshooting

### Database Connection Error

```
Error: error connecting to database: Connection refused
```

**Solution:**
- Ensure PostgreSQL is running: `sudo systemctl status postgresql`
- Check DATABASE_URL in .env matches your PostgreSQL setup
- Verify you can connect manually: `psql -U postgres -d crypto_gateway_staging`

### Redis Connection Error

```
Error: Redis connection failed
```

**Solution:**
- Ensure Redis is running: `redis-cli ping`
- Check REDIS_URL in .env
- Start Redis: `sudo systemctl start redis`

### Migration Error

```
Error: migration ... failed
```

**Solution:**
- Check database exists: `psql -l | grep crypto_gateway`
- Revert and retry: `sqlx migrate revert && sqlx migrate run`
- Check migration files in `migrations/` directory

### Port Already in Use

```
Error: Address already in use (os error 98)
```

**Solution:**
- Change SERVER_PORT in .env to a different port (e.g., 8081)
- Or kill the process using port 8080: `sudo lsof -ti:8080 | xargs kill -9`

---

## Next Steps

1. **Test the API** - Use the verification steps above
2. **Enable Sandbox Mode** - Test without real blockchain transactions
3. **Configure Webhooks** - Set up webhook endpoints for payment notifications
4. **Set Up Monitoring** - Configure logging and metrics
5. **Production Deployment** - Follow production deployment guide

---

## Production Checklist

Before deploying to production:

- [ ] Generate secure WEBHOOK_SIGNING_KEY
- [ ] Use dedicated RPC endpoints (not public ones)
- [ ] Set up SSL/TLS certificates
- [ ] Configure firewall rules
- [ ] Set up database backups
- [ ] Configure log aggregation
- [ ] Set up monitoring and alerts
- [ ] Review and update rate limits
- [ ] Test disaster recovery procedures
- [ ] Document runbook for operations team

---

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review logs: `tail -f /var/log/crypto-gateway.log`
3. Check GitHub issues
4. Contact the development team
