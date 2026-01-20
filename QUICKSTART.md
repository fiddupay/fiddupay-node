# PayFlow - Quick Start Guide

## Server Status

**✅ Server Running**
- PID: Check with `ps aux | grep crypto-payment-gateway`
- Port: 8080
- Logs: `/tmp/server.log`

## Quick Commands

### Start Server
```bash
cd /home/vibes/crypto-payment-gateway
SQLX_OFFLINE=true cargo run --release
```

### Stop Server
```bash
pkill -f crypto-payment-gateway
```

### View Logs
```bash
tail -f /tmp/server.log
```

### Run Tests
```bash
SQLX_OFFLINE=true cargo test --test standalone_tests
```

### Build Release
```bash
SQLX_OFFLINE=true cargo build --release
```

## API Testing

### Health Check
```bash
curl http://localhost:8080/health
```

### Create Merchant (requires proper setup)
```bash
curl -X POST http://localhost:8080/api/v1/merchants \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "business_name": "My Business",
    "email": "business@example.com"
  }'
```

### Create Payment
```bash
curl -X POST http://localhost:8080/api/v1/payments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "amount_usd": "100.00",
    "crypto_type": "SOL",
    "customer_email": "customer@example.com"
  }'
```

## Database Access

### Connect to Database
```bash
psql "postgresql://vibes:Soledayo%402001@localhost:5432/payflow"
```

### Common Queries
```sql
-- List merchants
SELECT id, business_name, email, is_active FROM merchants;

-- List payments
SELECT payment_id, merchant_id, amount, crypto_type, status FROM payment_transactions LIMIT 10;

-- Check balances
SELECT * FROM merchant_balances;
```

## Troubleshooting

### Port Already in Use
Change port in `.env`:
```bash
SERVER_PORT=3000
```

### Database Connection Error
Check PostgreSQL is running:
```bash
systemctl status postgresql
```

### Compilation Errors
Ensure SQLX offline mode:
```bash
export SQLX_OFFLINE=true
```

### Missing Dependencies
```bash
cargo clean
cargo build --release
```

## Development Workflow

1. **Make Changes** to source code
2. **Run Tests**: `SQLX_OFFLINE=true cargo test --test standalone_tests`
3. **Build**: `SQLX_OFFLINE=true cargo build --release`
4. **Restart Server**: `pkill -f crypto-payment-gateway && ./target/release/crypto-payment-gateway &`
5. **Test API**: `./test_api.sh`

## Production Checklist

- [ ] Generate production encryption keys
- [ ] Setup Redis for caching
- [ ] Configure RPC endpoints (QuickNode/Alchemy)
- [ ] Setup SMTP for emails
- [ ] Configure domain and SSL
- [ ] Test on blockchain testnets
- [ ] Setup monitoring and alerts
- [ ] Configure backups
- [ ] Load test the API
- [ ] Security audit

## Support

- Documentation: `docs/`
- API Reference: `docs/API.md`
- Merchant Guide: `docs/MERCHANT_GUIDE.md`
- Deployment: `docs/DEPLOYMENT.md`

---

**Current Status:** ✅ Server running on port 8080
**Last Updated:** 2026-01-20
