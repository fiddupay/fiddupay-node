# fiddupay Production Deployment Guide

Complete guide for deploying fiddupay cryptocurrency payment gateway to production.

## Prerequisites

### System Requirements
- **OS**: Ubuntu 20.04+ or CentOS 8+
- **CPU**: 4+ cores
- **RAM**: 8GB+
- **Storage**: 100GB+ SSD
- **Network**: 1 Gbps

### Dependencies
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+
- Nginx
- SSL certificates

## Quick Deployment

### 1. Server Setup
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 2. Application Deployment
```bash
# Clone repository
git clone <repository-url>
cd crypto-payment-gateway

# Create production environment
cp .env.example .env.production

# Edit production configuration
nano .env.production
```

### 3. Production Environment
```env
# Database
DATABASE_URL=postgresql://fiddupay_user:STRONG_PASSWORD@postgres:5432/fiddupay_production
REDIS_URL=redis://redis:6379

# Security (Generate with: openssl rand -hex 32)
ENCRYPTION_KEY=your_32_byte_production_encryption_key
WEBHOOK_SIGNING_KEY=your_32_byte_production_webhook_key

# Server
HOST=0.0.0.0
PORT=8080
PAYMENT_LINK_BASE_URL=https://pay.yourdomain.com

# Blockchain RPC URLs (Use paid providers for production)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
BSC_RPC_URL=https://bsc-dataseed.binance.org
POLYGON_RPC_URL=https://polygon-rpc.com
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# API Keys (Unified)
ETHERSCAN_API_KEY=your_etherscan_api_key

**Benefits of Unified API:**
- Single API key for all supported blockchains
- Simplified configuration and management
- Better rate limits and cost efficiency
- Automatic support for new chains as Etherscan adds them

# Email (Optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_app_password

# Features
ENABLE_EMAIL_NOTIFICATIONS=true
ENABLE_2FA=true
ENABLE_IP_WHITELIST=true
```

### 4. Docker Compose Setup
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  fiddupay:
    build: .
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://fiddupay_user:${DB_PASSWORD}@postgres:5432/fiddupay_production
      - REDIS_URL=redis://redis:6379
    env_file:
      - .env.production
    depends_on:
      - postgres
      - redis
    volumes:
      - ./logs:/app/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:15
    restart: unless-stopped
    environment:
      POSTGRES_DB: fiddupay_production
      POSTGRES_USER: fiddupay_user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"

  nginx:
    image: nginx:alpine
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
      - ./logs:/var/log/nginx
    depends_on:
      - fiddupay

volumes:
  postgres_data:
  redis_data:
```

### 5. SSL Configuration
```bash
# Install Certbot
sudo apt install certbot

# Get SSL certificate
sudo certbot certonly --standalone -d api.yourdomain.com

# Configure auto-renewal
echo "0 12 * * * /usr/bin/certbot renew --quiet" | sudo crontab -
```

### 6. Nginx Configuration
```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream fiddupay {
        server fiddupay:8080;
    }

    server {
        listen 80;
        server_name api.yourdomain.com;
        return 301 https://$server_name$request_uri;
    }

    server {
        listen 443 ssl http2;
        server_name api.yourdomain.com;

        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;

        location / {
            proxy_pass http://fiddupay;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }

        location /health {
            proxy_pass http://fiddupay/health;
            access_log off;
        }
    }
}
```

### 7. Deploy Application
```bash
# Set environment variables
export DB_PASSWORD=$(openssl rand -base64 32)
export REDIS_PASSWORD=$(openssl rand -base64 32)

# Start services
docker-compose -f docker-compose.prod.yml up -d

# Check status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f fiddupay
```

## Security Hardening

### 1. Firewall Configuration
```bash
# Configure UFW
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80
sudo ufw allow 443
sudo ufw enable
```

### 2. Database Security
```sql
-- Create dedicated user
CREATE USER fiddupay_user WITH PASSWORD 'STRONG_PASSWORD';
CREATE DATABASE fiddupay_production OWNER fiddupay_user;
GRANT ALL PRIVILEGES ON DATABASE fiddupay_production TO fiddupay_user;

-- Restrict connections
ALTER USER fiddupay_user CONNECTION LIMIT 20;
```

### 3. Redis Security
```bash
# Configure Redis authentication
echo "requirepass STRONG_REDIS_PASSWORD" >> /etc/redis/redis.conf
echo "bind 127.0.0.1" >> /etc/redis/redis.conf
sudo systemctl restart redis
```

### 4. Application Security
```bash
# Set proper file permissions
sudo chown -R fiddupay:fiddupay /opt/fiddupay
sudo chmod 600 /opt/fiddupay/.env.production
sudo chmod 755 /opt/fiddupay/target/release/crypto-payment-gateway
```

## Monitoring & Logging

### 1. Application Monitoring
```yaml
# docker-compose.monitoring.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  grafana_data:
```

### 2. Log Management
```bash
# Configure log rotation
sudo cat > /etc/logrotate.d/fiddupay << EOF
/opt/fiddupay/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 fiddupay fiddupay
    postrotate
        docker-compose -f /opt/fiddupay/docker-compose.prod.yml restart fiddupay
    endscript
}
EOF
```

### 3. Health Checks
```bash
# Create health check script
cat > /opt/fiddupay/health-check.sh << EOF
#!/bin/bash
HEALTH_URL="https://api.yourdomain.com/health"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $HEALTH_URL)

if [ $RESPONSE -eq 200 ]; then
    echo "$(date): fiddupay is healthy"
else
    echo "$(date): fiddupay is unhealthy (HTTP $RESPONSE)"
    # Send alert (email, Slack, etc.)
fi
EOF

chmod +x /opt/fiddupay/health-check.sh

# Add to crontab
echo "*/5 * * * * /opt/fiddupay/health-check.sh >> /var/log/fiddupay-health.log" | crontab -
```

## Backup & Recovery

### 1. Database Backup
```bash
# Create backup script
cat > /opt/fiddupay/backup-db.sh << EOF
#!/bin/bash
BACKUP_DIR="/opt/fiddupay/backups"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

docker exec fiddupay_postgres_1 pg_dump -U fiddupay_user fiddupay_production | gzip > $BACKUP_DIR/fiddupay_$DATE.sql.gz

# Keep only last 7 days
find $BACKUP_DIR -name "fiddupay_*.sql.gz" -mtime +7 -delete

# Upload to S3 (optional)
aws s3 cp $BACKUP_DIR/fiddupay_$DATE.sql.gz s3://your-backup-bucket/
EOF

chmod +x /opt/fiddupay/backup-db.sh

# Schedule daily backups
echo "0 2 * * * /opt/fiddupay/backup-db.sh" | crontab -
```

### 2. Application Backup
```bash
# Backup configuration and data
tar -czf fiddupay-backup-$(date +%Y%m%d).tar.gz \
  /opt/fiddupay/.env.production \
  /opt/fiddupay/docker-compose.prod.yml \
  /opt/fiddupay/nginx.conf \
  /opt/fiddupay/ssl/
```

## Performance Optimization

### 1. Database Tuning
```sql
-- PostgreSQL configuration
ALTER SYSTEM SET shared_buffers = '2GB';
ALTER SYSTEM SET effective_cache_size = '6GB';
ALTER SYSTEM SET maintenance_work_mem = '512MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
SELECT pg_reload_conf();
```

### 2. Redis Optimization
```bash
# Redis configuration
echo "maxmemory 1gb" >> /etc/redis/redis.conf
echo "maxmemory-policy allkeys-lru" >> /etc/redis/redis.conf
echo "save 900 1" >> /etc/redis/redis.conf
echo "save 300 10" >> /etc/redis/redis.conf
```

### 3. Application Tuning
```env
# Production optimizations
RUST_LOG=info
TOKIO_WORKER_THREADS=8
DATABASE_MAX_CONNECTIONS=50
REDIS_POOL_SIZE=20
```

## Troubleshooting

### Common Issues

#### High CPU Usage
```bash
# Check container resources
docker stats

# Check application logs
docker-compose logs fiddupay | grep ERROR

# Monitor database queries
docker exec -it fiddupay_postgres_1 psql -U fiddupay_user -d fiddupay_production -c "SELECT query, calls, total_time FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"
```

#### Memory Issues
```bash
# Check memory usage
free -h
docker exec fiddupay_redis_1 redis-cli info memory

# Check for memory leaks
docker exec fiddupay_fiddupay_1 ps aux --sort=-%mem | head
```

#### Database Connection Issues
```bash
# Check database connections
docker exec fiddupay_postgres_1 psql -U fiddupay_user -d fiddupay_production -c "SELECT count(*) FROM pg_stat_activity;"

# Check connection limits
docker exec fiddupay_postgres_1 psql -U fiddupay_user -d fiddupay_production -c "SHOW max_connections;"
```

## Maintenance

### Regular Tasks
- Monitor disk space and clean logs
- Update SSL certificates
- Review and rotate API keys
- Update dependencies
- Monitor performance metrics
- Review security logs
- Test backup restoration

### Updates
```bash
# Update application
git pull origin main
docker-compose -f docker-compose.prod.yml build --no-cache
docker-compose -f docker-compose.prod.yml up -d

# Update dependencies
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

This deployment guide provides a production-ready setup with security, monitoring, and maintenance considerations.
sudo systemctl start fiddupay

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
sudo cp fiddupay.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable fiddupay
sudo systemctl start fiddupay
```

### 7. Nginx (20 min) - Optional
```bash
sudo cp fiddupay.nginx /etc/nginx/sites-available/fiddupay
sudo ln -s /etc/nginx/sites-available/fiddupay /etc/nginx/sites-enabled/
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
-  Argon2 password hashing (OWASP recommended)
-  AES-256-GCM encryption for sensitive data
-  API key authentication
-  TOTP 2FA support
-  Webhook signature verification (HMAC-SHA256)
-  IP whitelisting
-  Rate limiting (configurable)
-  Audit logging

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
# - fiddupay.service (systemd service)
# - fiddupay.nginx (reverse proxy config)
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
