# MSSCS Discovery Server Deployment Script (Windows)
# This script helps deploy the discovery server for internet-wide P2P connectivity

Write-Host "╔════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     MSSCS Discovery Server Deployment Helper          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if Node.js is installed
try {
    $nodeVersion = node --version
    Write-Host "✅ Node.js detected: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Node.js not found. Please install Node.js first." -ForegroundColor Red
    exit 1
}

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
npm install

# Create Windows Service configuration
Write-Host "🔧 Creating Windows Service configuration..." -ForegroundColor Yellow

$serviceScript = @"
// Windows Service wrapper for MSSCS Discovery Server
const Service = require('node-windows').Service;

const svc = new Service({
  name: 'MSSCS Discovery Server',
  description: 'MSSCS P2P Discovery Server for internet-wide connectivity',
  script: require('path').join(__dirname, 'discovery-server.js'),
  env: [
    {
      name: 'NODE_ENV',
      value: 'production'
    },
    {
      name: 'PORT',
      value: '9001'
    }
  ]
});

svc.on('install', function() {
  console.log('Service installed successfully!');
  svc.start();
});

svc.on('start', function() {
  console.log('Service started!');
});

svc.install();
"@

$serviceScript | Out-File -FilePath "install-service.js" -Encoding UTF8

Write-Host "📋 Service installer created: install-service.js" -ForegroundColor Green
Write-Host ""
Write-Host "To install as Windows Service:" -ForegroundColor Cyan
Write-Host "  1. Install node-windows: npm install -g node-windows" -ForegroundColor White
Write-Host "  2. Run installer: node install-service.js" -ForegroundColor White
Write-Host "  3. Service will start automatically" -ForegroundColor White
Write-Host ""

# Create PM2 configuration (alternative to Windows Service)
Write-Host "🔧 Creating PM2 configuration..." -ForegroundColor Yellow

$pm2Config = @"
{
  "apps": [{
    "name": "msscs-discovery",
    "script": "discovery-server.js",
    "instances": 1,
    "exec_mode": "fork",
    "watch": false,
    "env": {
      "NODE_ENV": "production",
      "PORT": 9001,
      "USE_HTTPS": false
    }
  }]
}
"@

$pm2Config | Out-File -FilePath "ecosystem.config.json" -Encoding UTF8

Write-Host "📋 PM2 config created: ecosystem.config.json" -ForegroundColor Green
Write-Host ""
Write-Host "To run with PM2 (recommended):" -ForegroundColor Cyan
Write-Host "  1. Install PM2: npm install -g pm2" -ForegroundColor White
Write-Host "  2. Start server: pm2 start ecosystem.config.json" -ForegroundColor White
Write-Host "  3. Save config: pm2 save" -ForegroundColor White
Write-Host "  4. Setup startup: pm2 startup" -ForegroundColor White
Write-Host "  5. Monitor: pm2 monit" -ForegroundColor White
Write-Host ""

# Create Docker configuration
Write-Host "🐳 Creating Docker configuration..." -ForegroundColor Yellow

$dockerfile = @"
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY discovery-server.js ./

EXPOSE 9001

ENV NODE_ENV=production
ENV PORT=9001

CMD ["node", "discovery-server.js"]
"@

$dockerfile | Out-File -FilePath "Dockerfile.discovery" -Encoding UTF8

$dockerCompose = @"
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

"@

$dockerCompose | Out-File -FilePath "docker-compose.discovery.yml" -Encoding UTF8

Write-Host "📋 Docker files created:" -ForegroundColor Green
Write-Host "  - Dockerfile.discovery" -ForegroundColor White
Write-Host "  - docker-compose.discovery.yml" -ForegroundColor White
Write-Host ""
Write-Host "To deploy with Docker:" -ForegroundColor Cyan
Write-Host "  1. Install Docker Desktop for Windows" -ForegroundColor White
Write-Host "  2. Build: docker-compose -f docker-compose.discovery.yml build" -ForegroundColor White
Write-Host "  3. Run: docker-compose -f docker-compose.discovery.yml up -d" -ForegroundColor White
Write-Host ""

# Firewall configuration
Write-Host "🔥 Firewall configuration:" -ForegroundColor Yellow
Write-Host "  Run as Administrator:" -ForegroundColor White
Write-Host "  netsh advfirewall firewall add rule name=`"MSSCS Discovery`" dir=in action=allow protocol=TCP localport=9001" -ForegroundColor Cyan
Write-Host ""

# Create start script
$startScript = @"
@echo off
echo Starting MSSCS Discovery Server...
node discovery-server.js
pause
"@

$startScript | Out-File -FilePath "start-discovery.bat" -Encoding ASCII

Write-Host "📋 Start script created: start-discovery.bat" -ForegroundColor Green
Write-Host ""

Write-Host "✅ Deployment files created!" -ForegroundColor Green
Write-Host ""
Write-Host "📚 Quick Start Options:" -ForegroundColor Cyan
Write-Host ""
Write-Host "Option 1 - Simple (for testing):" -ForegroundColor Yellow
Write-Host "  Double-click start-discovery.bat" -ForegroundColor White
Write-Host ""
Write-Host "Option 2 - PM2 (recommended for production):" -ForegroundColor Yellow
Write-Host "  npm install -g pm2" -ForegroundColor White
Write-Host "  pm2 start ecosystem.config.json" -ForegroundColor White
Write-Host ""
Write-Host "Option 3 - Windows Service:" -ForegroundColor Yellow
Write-Host "  npm install -g node-windows" -ForegroundColor White
Write-Host "  node install-service.js" -ForegroundColor White
Write-Host ""
Write-Host "Option 4 - Docker:" -ForegroundColor Yellow
Write-Host "  docker-compose -f docker-compose.discovery.yml up -d" -ForegroundColor White
Write-Host ""
Write-Host "🌍 For internet-wide connectivity:" -ForegroundColor Cyan
Write-Host "  1. Configure port forwarding on your router (port 9001)" -ForegroundColor White
Write-Host "  2. Get your public IP: curl ifconfig.me" -ForegroundColor White
Write-Host "  3. Update clients with: ws://YOUR_PUBLIC_IP:9001" -ForegroundColor White
Write-Host "  4. Consider using a dynamic DNS service" -ForegroundColor White
Write-Host "  5. For production, use a VPS with static IP" -ForegroundColor White
Write-Host ""
