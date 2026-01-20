# Crypto Payment Gateway - Docker Deployment

## Quick Start

```bash
# 1. Set environment variables
cp .env.example .env
# Edit .env with your configuration

# 2. Start all services
docker-compose up -d

# 3. Run migrations
docker-compose exec gateway sqlx migrate run

# 4. Check health
curl http://localhost:8080/health
```

## Services

- **gateway**: Main application (port 8080)
- **postgres**: PostgreSQL database (port 5432)
- **redis**: Redis cache (port 6379)

## Environment Variables

Required variables in `.env`:

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

- Change default passwords
- Use strong `WEBHOOK_SIGNING_KEY`
- Enable SSL/TLS (use reverse proxy like nginx)
- Set up firewall rules
- Use managed database (AWS RDS, etc.)
- Use managed Redis (AWS ElastiCache, etc.)

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
