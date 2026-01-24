#!/bin/bash
# PayFlow - Production Infrastructure Setup Script

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     PayFlow - Production Infrastructure Setup             ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo -e "${RED}❌ Do not run as root${NC}"
   exit 1
fi

echo "=== Step 1: Generate Production Keys ==="
echo ""

# Generate encryption key
ENCRYPTION_KEY=$(openssl rand -hex 32)
echo -e "${GREEN}✅ Generated ENCRYPTION_KEY${NC}"

# Generate webhook signing key
WEBHOOK_SIGNING_KEY=$(openssl rand -hex 32)
echo -e "${GREEN}✅ Generated WEBHOOK_SIGNING_KEY${NC}"

# Generate JWT secret (for future use)
JWT_SECRET=$(openssl rand -hex 32)
echo -e "${GREEN}✅ Generated JWT_SECRET${NC}"

echo ""
echo "=== Step 2: Create Production Environment File ==="
echo ""

cat > .env.production <<EOF
# ============================================================================
# PRODUCTION ENVIRONMENT CONFIGURATION
# ============================================================================
# Generated: $(date)
# WARNING: Keep this file secure! Contains sensitive keys.

# Database Configuration
DATABASE_URL=postgresql://payflow_user:CHANGE_THIS_PASSWORD@localhost:5432/payflow_production
DATABASE_MAX_CONNECTIONS=50

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=20

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_WORKERS=8

# Production Keys (GENERATED - DO NOT SHARE)
ENCRYPTION_KEY=${ENCRYPTION_KEY}
WEBHOOK_SIGNING_KEY=${WEBHOOK_SIGNING_KEY}
JWT_SECRET=${JWT_SECRET}

# Blockchain RPC URLs (CONFIGURE WITH YOUR PROVIDERS)
# Get from: QuickNode, Alchemy, Infura
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
BSC_RPC_URL=https://bsc-dataseed.binance.org
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc
POLYGON_RPC_URL=https://polygon-rpc.com

# API Keys for Blockchain Explorers (Optional)
BSCSCAN_API_KEY=
ARBISCAN_API_KEY=
POLYGONSCAN_API_KEY=

# Price API
BYBIT_PRICE_API_URL=https://api.bybit.com
PRICE_CACHE_TTL_SECONDS=30

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
RATE_LIMIT_BURST=100

# Logging
RUST_LOG=info,crypto_payment_gateway=info

# Payment Configuration
DEFAULT_PAYMENT_EXPIRATION_MINUTES=15
DEFAULT_FEE_PERCENTAGE=1.50

# Hosted Pages
PAYMENT_PAGE_BASE_URL=https://pay.yourdomain.com

# Feature Flags
EMAIL_ENABLED=true
EMAIL_FROM=noreply@yourdomain.com
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=YOUR_SENDGRID_API_KEY

TWO_FACTOR_ENABLED=true
DEPOSIT_ADDRESS_ENABLED=true
WITHDRAWAL_ENABLED=true
WITHDRAWAL_MIN_AMOUNT=10.00
WITHDRAWAL_AUTO_APPROVE_THRESHOLD=1000.00
WITHDRAWAL_FEE_PERCENT=0.5

INVOICE_ENABLED=true
MULTI_USER_ENABLED=true
MAINTENANCE_MODE=false

# SQLx
SQLX_OFFLINE=true
EOF

echo -e "${GREEN}✅ Created .env.production${NC}"
echo ""

echo "=== Step 3: Security Checklist ==="
echo ""
echo "File Permissions:"
chmod 600 .env.production
echo -e "${GREEN}✅ Set .env.production to 600 (owner read/write only)${NC}"

echo ""
echo "=== Step 4: Database Setup Instructions ==="
echo ""
echo "Run these commands to setup production database:"
echo ""
echo "  # Create database user"
echo "  sudo -u postgres psql -c \"CREATE USER payflow_user WITH PASSWORD 'STRONG_PASSWORD_HERE';\""
echo ""
echo "  # Create database"
echo "  sudo -u postgres psql -c \"CREATE DATABASE payflow_production OWNER payflow_user;\""
echo ""
echo "  # Grant privileges"
echo "  sudo -u postgres psql -c \"GRANT ALL PRIVILEGES ON DATABASE payflow_production TO payflow_user;\""
echo ""
echo "  # Run migrations"
echo "  DATABASE_URL='postgresql://payflow_user:PASSWORD@localhost:5432/payflow_production' sqlx migrate run"
echo ""

echo "=== Step 5: Redis Setup Instructions ==="
echo ""
echo "Configure Redis for production:"
echo ""
echo "  # Edit Redis config"
echo "  sudo nano /etc/redis/redis.conf"
echo ""
echo "  # Enable persistence (add these lines):"
echo "  save 900 1"
echo "  save 300 10"
echo "  save 60 10000"
echo "  appendonly yes"
echo "  appendfsync everysec"
echo ""
echo "  # Restart Redis"
echo "  sudo systemctl restart redis"
echo ""

echo "=== Step 6: SSL/TLS Certificate ==="
echo ""
echo "Get SSL certificate (choose one):"
echo ""
echo "  Option 1: Let's Encrypt (Free)"
echo "  sudo apt install certbot"
echo "  sudo certbot certonly --standalone -d yourdomain.com"
echo ""
echo "  Option 2: Cloudflare (Free + DDoS protection)"
echo "  - Add domain to Cloudflare"
echo "  - Enable SSL/TLS (Full mode)"
echo "  - Use Cloudflare Origin Certificate"
echo ""

echo "=== Step 7: Firewall Configuration ==="
echo ""
echo "Configure UFW firewall:"
echo ""
echo "  sudo ufw allow 22/tcp    # SSH"
echo "  sudo ufw allow 80/tcp    # HTTP"
echo "  sudo ufw allow 443/tcp   # HTTPS"
echo "  sudo ufw allow 8080/tcp  # PayFlow (if not behind proxy)"
echo "  sudo ufw enable"
echo ""

echo "=== Step 8: Systemd Service ==="
echo ""
echo "Create systemd service file:"
echo ""
cat > payflow.service <<EOF
[Unit]
Description=PayFlow Cryptocurrency Payment Gateway
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=$(whoami)
WorkingDirectory=$(pwd)
Environment="RUST_LOG=info"
EnvironmentFile=$(pwd)/.env.production
ExecStart=$(pwd)/target/release/crypto-payment-gateway
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

echo -e "${GREEN}✅ Created payflow.service${NC}"
echo ""
echo "Install service:"
echo "  sudo cp payflow.service /etc/systemd/system/"
echo "  sudo systemctl daemon-reload"
echo "  sudo systemctl enable payflow"
echo "  sudo systemctl start payflow"
echo ""

echo "=== Step 9: Nginx Reverse Proxy (Optional) ==="
echo ""
echo "Create Nginx config:"
echo ""
cat > payflow.nginx <<EOF
server {
    listen 80;
    server_name yourdomain.com;
    
    # Redirect to HTTPS
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;
    
    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    # Rate Limiting
    limit_req_zone \$binary_remote_addr zone=api:10m rate=100r/m;
    limit_req zone=api burst=20 nodelay;
    
    # Proxy to PayFlow
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # Health check endpoint (no rate limit)
    location /health {
        proxy_pass http://127.0.0.1:8080/health;
        access_log off;
    }
}
EOF

echo -e "${GREEN}✅ Created payflow.nginx${NC}"
echo ""
echo "Install Nginx config:"
echo "  sudo cp payflow.nginx /etc/nginx/sites-available/payflow"
echo "  sudo ln -s /etc/nginx/sites-available/payflow /etc/nginx/sites-enabled/"
echo "  sudo nginx -t"
echo "  sudo systemctl restart nginx"
echo ""

echo "=== Step 10: Monitoring Setup ==="
echo ""
echo "Install Prometheus & Grafana:"
echo ""
echo "  # Prometheus"
echo "  wget https://github.com/prometheus/prometheus/releases/download/v2.45.0/prometheus-2.45.0.linux-amd64.tar.gz"
echo "  tar xvfz prometheus-*.tar.gz"
echo "  cd prometheus-*"
echo "  ./prometheus --config.file=prometheus.yml"
echo ""
echo "  # Add PayFlow metrics endpoint to prometheus.yml:"
echo "  scrape_configs:"
echo "    - job_name: 'payflow'"
echo "      static_configs:"
echo "        - targets: ['localhost:8080']"
echo ""

echo "=== Step 11: Backup Configuration ==="
echo ""
echo "Setup automated backups:"
echo ""
cat > backup.sh <<'BACKUP_EOF'
#!/bin/bash
# PayFlow Database Backup Script

BACKUP_DIR="/var/backups/payflow"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="payflow_production"
DB_USER="payflow_user"

mkdir -p $BACKUP_DIR

# Backup database
pg_dump -U $DB_USER $DB_NAME | gzip > $BACKUP_DIR/payflow_$DATE.sql.gz

# Keep only last 30 days
find $BACKUP_DIR -name "payflow_*.sql.gz" -mtime +30 -delete

echo "Backup completed: payflow_$DATE.sql.gz"
BACKUP_EOF

chmod +x backup.sh
echo -e "${GREEN}✅ Created backup.sh${NC}"
echo ""
echo "Add to crontab (daily at 2 AM):"
echo "  0 2 * * * $(pwd)/backup.sh"
echo ""

echo "╔════════════════════════════════════════════════════════════╗"
echo "║              Infrastructure Setup Complete                 ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✅ Production keys generated${NC}"
echo -e "${GREEN}✅ Environment file created${NC}"
echo -e "${GREEN}✅ Service file created${NC}"
echo -e "${GREEN}✅ Nginx config created${NC}"
echo -e "${GREEN}✅ Backup script created${NC}"
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT: Update these in .env.production:${NC}"
echo "  1. DATABASE_URL password"
echo "  2. SMTP credentials"
echo "  3. RPC endpoints (QuickNode/Alchemy)"
echo "  4. Domain name"
echo ""
echo -e "${YELLOW}⚠️  NEXT STEPS:${NC}"
echo "  1. Review .env.production"
echo "  2. Setup production database"
echo "  3. Configure Redis persistence"
echo "  4. Get SSL certificate"
echo "  5. Install systemd service"
echo "  6. Configure Nginx (optional)"
echo "  7. Setup monitoring"
echo "  8. Configure backups"
echo "  9. Run security audit (./security_audit.sh)"
echo ""
echo "Files created:"
echo "  • .env.production"
echo "  • payflow.service"
echo "  • payflow.nginx"
echo "  • backup.sh"
echo ""
