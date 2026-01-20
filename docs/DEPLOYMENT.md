# PayFlow - Production Deployment Guide

## Quick Start

```bash
# 1. Run infrastructure setup
./setup_infrastructure.sh

# 2. Update .env.production
nano .env.production

# 3. Setup database
sudo -u postgres psql -c "CREATE USER payflow_user WITH PASSWORD 'STRONG_PASSWORD';"
sudo -u postgres psql -c "CREATE DATABASE payflow_production OWNER payflow_user;"
DATABASE_URL='postgresql://payflow_user:PASSWORD@localhost:5432/payflow_production' sqlx migrate run

# 4. Build and deploy
SQLX_OFFLINE=true cargo build --release
sudo cp payflow.service /etc/systemd/system/
sudo systemctl enable payflow
sudo systemctl start payflow

# 5. Check health
curl http://localhost:8080/health
```

## Pre-Deployment Checklist

### 1. Configuration (30 min)
- [ ] Review .env.production
- [ ] Update DATABASE_URL password
- [ ] Configure SMTP credentials
- [ ] Update domain name
- [ ] Configure RPC endpoints

### 2. Database Setup (15 min)
- [ ] Create production database
- [ ] Run migrations
- [ ] Verify schema

### 3. Redis (10 min)
- [ ] Enable persistence
- [ ] Restart Redis

### 4. SSL Certificate (30 min)
- [ ] Get Let's Encrypt cert OR
- [ ] Use Cloudflare

### 5. Firewall (10 min)
```bash
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### 6. Service (15 min)
```bash
sudo cp payflow.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable payflow
sudo systemctl start payflow
```

### 7. Nginx (20 min) - Optional
```bash
sudo cp payflow.nginx /etc/nginx/sites-available/payflow
sudo ln -s /etc/nginx/sites-available/payflow /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

## Environment Variables

Required in `.env.production`:

```bash
# Database
DATABASE_URL=postgresql://gateway:gateway@postgres:5432/crypto_gateway

# Redis
REDIS_URL=redis://redis:6379

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
PAYMENT_PAGE_BASE_URL=http://localhost:8080

# Security
WEBHOOK_SIGNING_KEY=your-secret-key-here

# Blockchain RPCs (testnet)
SOLANA_RPC_URL=https://api.devnet.solana.com
BSC_RPC_URL=https://data-seed-prebsc-1-s1.binance.org:8545
ARBITRUM_RPC_URL=https://goerli-rollup.arbitrum.io/rpc
POLYGON_RPC_URL=https://rpc-mumbai.maticvigil.com
```

## Production Deployment

### 1. Build for Production

```bash
docker build -t crypto-gateway:latest .
```

### 2. Use Production RPC Endpoints

Update `.env` with mainnet RPCs:

```bash
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
BSC_RPC_URL=https://bsc-dataseed.binance.org
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc
POLYGON_RPC_URL=https://polygon-rpc.com
```

### 3. Secure Your Deployment

**Generate Production Keys:**
```bash
# Generate encryption key (32 bytes)
openssl rand -hex 32

# Generate webhook signing key (32 bytes)
openssl rand -hex 32
```

**Security Features Built-In:**
- ✅ Argon2 password hashing (OWASP recommended)
- ✅ AES-256-GCM encryption for sensitive data
- ✅ API key authentication
- ✅ TOTP 2FA support
- ✅ Webhook signature verification (HMAC-SHA256)
- ✅ IP whitelisting
- ✅ Rate limiting (configurable)
- ✅ Audit logging

**Production Checklist:**
- [ ] Change default passwords
- [ ] Generate new ENCRYPTION_KEY
- [ ] Generate new WEBHOOK_SIGNING_KEY
- [ ] Enable SSL/TLS (use reverse proxy like nginx)
- [ ] Set up firewall rules (ports 22, 80, 443 only)
- [ ] Use managed database (AWS RDS, etc.)
- [ ] Use managed Redis (AWS ElastiCache, etc.)
- [ ] Configure backup strategy
- [ ] Setup monitoring and alerts
- [ ] Run security audit: `./security_audit.sh`

**Infrastructure Setup:**
```bash
# Run automated setup
./setup_infrastructure.sh

# This generates:
# - .env.production (with secure keys)
# - payflow.service (systemd service)
# - payflow.nginx (reverse proxy config)
```

**Security Audit:**
```bash
# Run comprehensive security check
./security_audit.sh

# Checks:
# - No hardcoded secrets
# - Proper encryption usage
# - Secure password hashing
# - File permissions
# - Dependency vulnerabilities
```

### 4. Scale with Kubernetes

See `k8s/` directory for Kubernetes manifests (coming soon).

## Monitoring

### Health Checks

```bash
# Liveness
curl http://localhost:8080/health

# Readiness (checks DB and Redis)
curl http://localhost:8080/health/ready
```

### Logs

```bash
# View logs
docker-compose logs -f gateway

# View specific service
docker-compose logs -f postgres
```

### Metrics

Prometheus metrics available at `/metrics` (if enabled).

## Backup & Restore

### Database Backup

```bash
docker-compose exec postgres pg_dump -U gateway crypto_gateway > backup.sql
```

### Database Restore

```bash
docker-compose exec -T postgres psql -U gateway crypto_gateway < backup.sql
```

## Troubleshooting

### Gateway won't start

1. Check logs: `docker-compose logs gateway`
2. Verify database is healthy: `docker-compose ps`
3. Check migrations: `docker-compose exec gateway sqlx migrate info`

### Database connection errors

1. Verify DATABASE_URL is correct
2. Check postgres is running: `docker-compose ps postgres`
3. Test connection: `docker-compose exec postgres psql -U gateway -d crypto_gateway`

### Redis connection errors

1. Verify REDIS_URL is correct
2. Check redis is running: `docker-compose ps redis`
3. Test connection: `docker-compose exec redis redis-cli ping`

## Maintenance

### Update Application

```bash
# Pull latest code
git pull

# Rebuild and restart
docker-compose up -d --build gateway
```

### Run Migrations

```bash
docker-compose exec gateway sqlx migrate run
```

### Clean Up

```bash
# Stop all services
docker-compose down

# Remove volumes (WARNING: deletes data)
docker-compose down -v
```
