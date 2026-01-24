# PayFlow Setup Guide

Complete guide to setting up PayFlow cryptocurrency payment gateway for development and production environments.

## 📋 Prerequisites

### System Requirements
- **Operating System**: Linux, macOS, or Windows (WSL recommended)
- **Rust**: Version 1.70 or higher
- **PostgreSQL**: Version 13 or higher
- **Redis**: Version 6 or higher
- **OpenSSL**: For cryptographic operations

### Development Tools (Optional)
- **Docker**: For containerized deployment
- **Git**: For version control
- **curl**: For API testing
- **jq**: For JSON processing

## 🚀 Quick Start (Development)

### 1. Install Dependencies

#### Rust Installation
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### PostgreSQL Installation
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql
brew services start postgresql

# Create database
sudo -u postgres createdb payflow
```

#### Redis Installation
```bash
# Ubuntu/Debian
sudo apt install redis-server

# macOS
brew install redis
brew services start redis

# Verify Redis is running
redis-cli ping
```

### 2. Clone and Build

```bash
# Clone repository
git clone <repository-url>
cd crypto-payment-gateway

# Build project
cargo build --release
```

### 3. Environment Configuration

```bash
# Copy environment template
cp .env.example .env

# Generate encryption keys
openssl rand -hex 32  # Copy this for ENCRYPTION_KEY
openssl rand -hex 32  # Copy this for WEBHOOK_SIGNING_KEY
```

Edit `.env` file with your configuration:

```env
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost/payflow
REDIS_URL=redis://localhost:6379

# Security Keys (Generate with openssl rand -hex 32)
ENCRYPTION_KEY=your_32_byte_hex_key_here
WEBHOOK_SIGNING_KEY=your_32_byte_hex_key_here

# Server Configuration
HOST=0.0.0.0
PORT=8080
PAYMENT_LINK_BASE_URL=http://localhost:8080

# Blockchain RPC URLs
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-api-key
BSC_RPC_URL=https://bsc-dataseed.binance.org
POLYGON_RPC_URL=https://polygon-rpc.com
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# API Keys for blockchain services
ETHERSCAN_API_KEY=your_etherscan_api_key
BSCSCAN_API_KEY=your_bscscan_api_key
POLYGONSCAN_API_KEY=your_polygonscan_api_key
ARBISCAN_API_KEY=your_arbiscan_api_key

# Email Configuration (Optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_app_password

# Feature Flags
ENABLE_EMAIL_NOTIFICATIONS=false
ENABLE_2FA=true
ENABLE_IP_WHITELIST=false
```

### 4. Database Setup

```bash
# Install SQLX CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
sqlx migrate run

# Verify database setup
psql -d payflow -c "\dt"
```

### 5. Start Services

```bash
# Start Redis (if not running)
redis-server

# Start PayFlow
cargo run --release
```

### 6. Verify Installation

```bash
# Health check
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy","timestamp":"2026-01-24T15:30:00Z"}

# Register test merchant
curl -X POST http://localhost:8080/api/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{
    "business_name": "Test Business",
    "email": "test@example.com",
    "password": "password123"
  }'

# Expected response:
# {"merchant_id":1,"api_key":"your_api_key_here"}
```

## 🐳 Docker Setup

### Using Docker Compose

```bash
# Create docker-compose.yml
cat > docker-compose.yml << EOF
version: '3.8'

services:
  payflow:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://payflow:password@postgres:5432/payflow
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    volumes:
      - ./.env:/app/.env

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: payflow
      POSTGRES_USER: payflow
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
EOF

# Start services
docker-compose up -d

# Check logs
docker-compose logs -f payflow
```

### Manual Docker Build

```bash
# Build image
docker build -t payflow .

# Run container
docker run -d \
  --name payflow \
  -p 8080:8080 \
  --env-file .env \
  payflow
```

## 🏭 Production Setup

### 1. Server Requirements

**Minimum Requirements:**
- CPU: 2 cores
- RAM: 4GB
- Storage: 50GB SSD
- Network: 100 Mbps

**Recommended Requirements:**
- CPU: 4+ cores
- RAM: 8GB+
- Storage: 100GB+ SSD
- Network: 1 Gbps

### 2. Security Configuration

#### SSL/TLS Setup
```bash
# Install Certbot
sudo apt install certbot

# Get SSL certificate
sudo certbot certonly --standalone -d api.yourdomain.com

# Configure reverse proxy (Nginx)
sudo apt install nginx

cat > /etc/nginx/sites-available/payflow << EOF
server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/api.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}

server {
    listen 80;
    server_name api.yourdomain.com;
    return 301 https://\$server_name\$request_uri;
}
EOF

sudo ln -s /etc/nginx/sites-available/payflow /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

#### Firewall Configuration
```bash
# Configure UFW
sudo ufw allow ssh
sudo ufw allow 80
sudo ufw allow 443
sudo ufw enable
```

### 3. Database Configuration

#### PostgreSQL Optimization
```sql
-- /etc/postgresql/15/main/postgresql.conf
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
```

#### Database Backup
```bash
# Create backup script
cat > /usr/local/bin/backup-payflow.sh << EOF
#!/bin/bash
BACKUP_DIR="/var/backups/payflow"
DATE=\$(date +%Y%m%d_%H%M%S)
mkdir -p \$BACKUP_DIR

pg_dump payflow | gzip > \$BACKUP_DIR/payflow_\$DATE.sql.gz

# Keep only last 7 days
find \$BACKUP_DIR -name "payflow_*.sql.gz" -mtime +7 -delete
EOF

chmod +x /usr/local/bin/backup-payflow.sh

# Add to crontab
echo "0 2 * * * /usr/local/bin/backup-payflow.sh" | sudo crontab -
```

### 4. Monitoring Setup

#### System Service
```bash
# Create systemd service
sudo cat > /etc/systemd/system/payflow.service << EOF
[Unit]
Description=PayFlow Cryptocurrency Payment Gateway
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=payflow
WorkingDirectory=/opt/payflow
ExecStart=/opt/payflow/target/release/crypto-payment-gateway
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable payflow
sudo systemctl start payflow
```

#### Log Management
```bash
# Configure log rotation
sudo cat > /etc/logrotate.d/payflow << EOF
/var/log/payflow/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 payflow payflow
    postrotate
        systemctl reload payflow
    endscript
}
EOF
```

### 5. Performance Tuning

#### Redis Configuration
```bash
# /etc/redis/redis.conf
maxmemory 512mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
```

#### Application Tuning
```env
# Production .env settings
RUST_LOG=info
TOKIO_WORKER_THREADS=4
DATABASE_MAX_CONNECTIONS=20
REDIS_POOL_SIZE=10
```

## 🧪 Testing Setup

### Run Test Suite
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# API tests
./test_complete_flow.sh

# Load testing (optional)
# Install wrk
sudo apt install wrk

# Run load test
wrk -t12 -c400 -d30s --header "Authorization: Bearer your_api_key" \
    http://localhost:8080/api/v1/payments
```

### Test Scripts
```bash
# Make scripts executable
chmod +x test_*.sh

# Run basic API tests
./test_basic_api.sh

# Run complete flow tests
./test_complete_flow.sh

# Run service layer tests
./test_service_layer.sh
```

## 🔧 Troubleshooting

### Common Issues

#### Database Connection Issues
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check database exists
sudo -u postgres psql -l | grep payflow

# Test connection
psql -d payflow -c "SELECT version();"
```

#### Redis Connection Issues
```bash
# Check Redis status
sudo systemctl status redis

# Test Redis connection
redis-cli ping

# Check Redis logs
sudo journalctl -u redis -f
```

#### Build Issues
```bash
# Clean build
cargo clean
cargo build --release

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

#### SSL Certificate Issues
```bash
# Renew certificates
sudo certbot renew

# Test certificate
openssl s_client -connect api.yourdomain.com:443
```

### Performance Issues

#### High CPU Usage
- Check for infinite loops in background tasks
- Monitor blockchain RPC call frequency
- Optimize database queries

#### High Memory Usage
- Check for memory leaks
- Monitor Redis memory usage
- Optimize data structures

#### Slow API Responses
- Check database query performance
- Monitor external API calls
- Implement caching where appropriate

## 📞 Support

### Getting Help
- **Documentation**: Check all `.md` files in the project
- **Logs**: Check application logs for error details
- **Database**: Verify database schema and data
- **Network**: Test external API connectivity

### Reporting Issues
When reporting issues, include:
- Operating system and version
- Rust version (`rustc --version`)
- Error messages and logs
- Steps to reproduce
- Configuration (without sensitive data)

## 🔄 Updates and Maintenance

### Updating PayFlow
```bash
# Backup database
./backup-payflow.sh

# Pull latest changes
git pull origin main

# Update dependencies
cargo update

# Run migrations
sqlx migrate run

# Rebuild and restart
cargo build --release
sudo systemctl restart payflow
```

### Regular Maintenance
- Monitor disk space
- Review and rotate logs
- Update SSL certificates
- Backup database regularly
- Monitor system performance
- Update dependencies
- Review security settings

This setup guide provides everything needed to get PayFlow running in both development and production environments.
