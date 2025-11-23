#!/bin/bash

# MSSCS Discovery Server Deployment Script
# This script helps deploy the discovery server for internet-wide P2P connectivity

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║     MSSCS Discovery Server Deployment Helper          ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo "⚠️  Warning: Running as root. Consider using a non-root user."
fi

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Create systemd service file
echo "🔧 Creating systemd service..."
cat > /tmp/msscs-discovery.service << 'EOF'
[Unit]
Description=MSSCS Discovery Server
After=network.target

[Service]
Type=simple
User=msscs
WorkingDirectory=/opt/msscs-discovery
ExecStart=/usr/bin/node discovery-server.js
Restart=always
RestartSec=10
Environment=NODE_ENV=production
Environment=PORT=9001
Environment=USE_HTTPS=false

[Install]
WantedBy=multi-user.target
EOF

echo "📋 Service file created at /tmp/msscs-discovery.service"
echo ""
echo "To install the service:"
echo "  1. Copy files to /opt/msscs-discovery"
echo "  2. Create user: sudo useradd -r -s /bin/false msscs"
echo "  3. Copy service: sudo cp /tmp/msscs-discovery.service /etc/systemd/system/"
echo "  4. Reload systemd: sudo systemctl daemon-reload"
echo "  5. Enable service: sudo systemctl enable msscs-discovery"
echo "  6. Start service: sudo systemctl start msscs-discovery"
echo "  7. Check status: sudo systemctl status msscs-discovery"
echo ""

# Nginx configuration
echo "🌐 Creating nginx configuration..."
cat > /tmp/msscs-discovery-nginx.conf << 'EOF'
# MSSCS Discovery Server - Nginx Configuration
# Place this in /etc/nginx/sites-available/msscs-discovery

upstream msscs_discovery {
    server localhost:9001;
}

server {
    listen 80;
    server_name discovery.yourdomain.com;

    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name discovery.yourdomain.com;

    # SSL Configuration (use certbot for Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/discovery.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/discovery.yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://msscs_discovery;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket timeouts
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }
}
EOF

echo "📋 Nginx config created at /tmp/msscs-discovery-nginx.conf"
echo ""
echo "To configure nginx:"
echo "  1. Install nginx: sudo apt install nginx"
echo "  2. Install certbot: sudo apt install certbot python3-certbot-nginx"
echo "  3. Get SSL cert: sudo certbot --nginx -d discovery.yourdomain.com"
echo "  4. Copy config: sudo cp /tmp/msscs-discovery-nginx.conf /etc/nginx/sites-available/msscs-discovery"
echo "  5. Enable site: sudo ln -s /etc/nginx/sites-available/msscs-discovery /etc/nginx/sites-enabled/"
echo "  6. Test config: sudo nginx -t"
echo "  7. Reload nginx: sudo systemctl reload nginx"
echo ""

# Firewall configuration
echo "🔥 Firewall configuration:"
echo "  sudo ufw allow 80/tcp"
echo "  sudo ufw allow 443/tcp"
echo "  sudo ufw allow 9001/tcp  # If not using nginx proxy"
echo ""

# Docker option
echo "🐳 Docker deployment option:"
cat > /tmp/Dockerfile.discovery << 'EOF'
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY discovery-server.js ./

EXPOSE 9001

ENV NODE_ENV=production
ENV PORT=9001

CMD ["node", "discovery-server.js"]
EOF

cat > /tmp/docker-compose.discovery.yml << 'EOF'
version: '3.8'

services:
  discovery:
    build:
      context: .
      dockerfile: Dockerfile.discovery
    ports:
      - "9001:9001"
    environment:
      - NODE_ENV=production
      - PORT=9001
      - USE_HTTPS=false
    restart: unless-stopped
    networks:
      - msscs

networks:
  msscs:
    driver: bridge
EOF

echo "📋 Docker files created:"
echo "  - /tmp/Dockerfile.discovery"
echo "  - /tmp/docker-compose.discovery.yml"
echo ""
echo "To deploy with Docker:"
echo "  1. Copy Dockerfile: cp /tmp/Dockerfile.discovery ./Dockerfile"
echo "  2. Copy compose file: cp /tmp/docker-compose.discovery.yml ./docker-compose.yml"
echo "  3. Build and run: docker-compose up -d"
echo ""

echo "✅ Deployment files created!"
echo ""
echo "📚 Next steps:"
echo "  1. Choose deployment method (systemd, nginx, or Docker)"
echo "  2. Configure your domain/IP"
echo "  3. Set up SSL certificates (recommended for production)"
echo "  4. Update client apps with your discovery server URL"
echo "  5. Test connectivity from different networks"
echo ""
echo "🌍 For internet-wide connectivity:"
echo "  - Ensure your server has a public IP"
echo "  - Configure DNS to point to your server"
echo "  - Open required ports in firewall"
echo "  - Use HTTPS/WSS for secure connections"
echo ""
