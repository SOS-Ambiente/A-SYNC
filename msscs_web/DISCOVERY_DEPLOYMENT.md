# Discovery Server Deployment Guide (OPTIONAL)

## ⚠️ IMPORTANT: This is 100% OPTIONAL!

**Your app already works fully serverless P2P without this!**

The discovery server is just a convenience feature for auto-discovering peers on the same local network (WiFi/LAN). 

### The App Already Works Without It:
- ✅ Uses free public PeerJS cloud for WebRTC signaling
- ✅ Uses free public STUN/TURN servers (Google, Mozilla)
- ✅ Direct P2P connections between peers
- ✅ All data encrypted and shared peer-to-peer
- ✅ NO central server sees your data
- ✅ Completely free and serverless

### When You Might Want This:
- You want peers on the same WiFi to auto-discover each other
- You want to avoid manually sharing Peer IDs
- You have multiple devices on the same network

### When You DON'T Need This:
- For internet-wide P2P (already works!)
- For privacy (already fully encrypted P2P!)
- For cost savings (already free!)

**TL;DR: Skip this entire file unless you want LAN auto-discovery!**

---

## Overview

The discovery server helps peers find each other on the same local network automatically. The app works perfectly without it using PeerJS cloud for signaling and manual Peer ID sharing.

## Quick Start (Local Testing)

```bash
cd msscs_web
node discovery-server.js
```

Server runs on `ws://localhost:9001`

## Internet-Wide Deployment

### Option 1: Deploy to a VPS (Recommended)

#### 1. Get a VPS
- DigitalOcean, Linode, AWS EC2, Google Cloud, etc.
- Minimum: 512MB RAM, 1 CPU core
- Ubuntu 20.04+ or similar

#### 2. Install Node.js
```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

#### 3. Install Dependencies
```bash
cd /opt
sudo git clone https://github.com/yourusername/msscs.git
cd msscs/msscs_web
sudo npm install ws
```

#### 4. Configure Environment
```bash
sudo nano /opt/msscs/msscs_web/.env
```

Add:
```env
PORT=9001
HOST=0.0.0.0
PUBLIC_URL=wss://discovery.yourdomain.com
USE_HTTPS=true
CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem
```

#### 5. Get SSL Certificate (Let's Encrypt)
```bash
sudo apt-get install certbot
sudo certbot certonly --standalone -d discovery.yourdomain.com
```

#### 6. Create Systemd Service
```bash
sudo nano /etc/systemd/system/msscs-discovery.service
```

```ini
[Unit]
Description=MSSCS Discovery Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/msscs/msscs_web
ExecStart=/usr/bin/node discovery-server.js
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=msscs-discovery

[Install]
WantedBy=multi-user.target
```

#### 7. Start Service
```bash
sudo systemctl daemon-reload
sudo systemctl enable msscs-discovery
sudo systemctl start msscs-discovery
sudo systemctl status msscs-discovery
```

#### 8. Configure Firewall
```bash
sudo ufw allow 9001/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### Option 2: Deploy with Nginx Reverse Proxy

#### 1. Install Nginx
```bash
sudo apt-get install nginx
```

#### 2. Configure Nginx
```bash
sudo nano /etc/nginx/sites-available/msscs-discovery
```

```nginx
upstream discovery {
    server localhost:9001;
}

server {
    listen 443 ssl http2;
    server_name discovery.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://discovery;
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

server {
    listen 80;
    server_name discovery.yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

#### 3. Enable Site
```bash
sudo ln -s /etc/nginx/sites-available/msscs-discovery /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Option 3: Deploy to Heroku

#### 1. Create Procfile
```
web: node msscs_web/discovery-server.js
```

#### 2. Deploy
```bash
heroku create msscs-discovery
heroku config:set PORT=443 USE_HTTPS=true
git push heroku main
```

### Option 4: Deploy to Railway.app

1. Connect your GitHub repo
2. Select `msscs_web` as root directory
3. Set start command: `node discovery-server.js`
4. Deploy automatically

### Option 5: Deploy to Fly.io

#### 1. Install Fly CLI
```bash
curl -L https://fly.io/install.sh | sh
```

#### 2. Create fly.toml
```toml
app = "msscs-discovery"

[build]
  dockerfile = "Dockerfile"

[[services]]
  internal_port = 9001
  protocol = "tcp"

  [[services.ports]]
    handlers = ["http"]
    port = 80

  [[services.ports]]
    handlers = ["tls", "http"]
    port = 443
```

#### 3. Create Dockerfile
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY msscs_web/discovery-server.js .
COPY msscs_web/package*.json .
RUN npm install ws
EXPOSE 9001
CMD ["node", "discovery-server.js"]
```

#### 4. Deploy
```bash
fly launch
fly deploy
```

## Client Configuration

### Web Client
```javascript
// In msscs_web/p2p.js or set in HTML
window.MSSCS_DISCOVERY_SERVER = 'wss://discovery.yourdomain.com';

// Or use localStorage
localStorage.setItem('msscs_discovery_server', 'wss://discovery.yourdomain.com');
```

### Tauri Client
```typescript
// In msscs_client/src/peerjs-bridge.ts
(window as any).MSSCS_DISCOVERY_SERVER = 'wss://discovery.yourdomain.com';
```

## Monitoring

### Check Logs
```bash
# Systemd
sudo journalctl -u msscs-discovery -f

# Nginx
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log
```

### Health Check
```bash
# Test WebSocket connection
wscat -c wss://discovery.yourdomain.com

# Send test message
{"type":"register","peerId":"test-peer","peerType":"test"}
```

## Scaling

### Multiple Servers
For high availability, deploy multiple discovery servers and use DNS round-robin or load balancer.

### Redis Backend (Optional)
For multi-server deployments, use Redis to share peer state:

```javascript
// discovery-server.js
import { createClient } from 'redis';

const redis = createClient({
    url: process.env.REDIS_URL
});

await redis.connect();

// Store peer in Redis
await redis.hSet('peers', peerId, JSON.stringify(metadata));

// Get all peers
const allPeers = await redis.hGetAll('peers');
```

## Security

### Rate Limiting
```javascript
// Add to discovery-server.js
const rateLimits = new Map();

function checkRateLimit(ip) {
    const now = Date.now();
    const limit = rateLimits.get(ip) || { count: 0, resetAt: now + 60000 };
    
    if (now > limit.resetAt) {
        limit.count = 0;
        limit.resetAt = now + 60000;
    }
    
    limit.count++;
    rateLimits.set(ip, limit);
    
    return limit.count <= 100; // 100 requests per minute
}
```

### Authentication (Optional)
```javascript
// Add API key authentication
const API_KEY = process.env.API_KEY;

ws.on('message', (data) => {
    const message = JSON.parse(data);
    
    if (message.apiKey !== API_KEY) {
        ws.send(JSON.stringify({ type: 'error', message: 'Invalid API key' }));
        ws.close();
        return;
    }
    
    // Process message...
});
```

## Troubleshooting

### Connection Refused
- Check firewall: `sudo ufw status`
- Check service: `sudo systemctl status msscs-discovery`
- Check port: `sudo netstat -tlnp | grep 9001`

### SSL Certificate Issues
- Renew certificate: `sudo certbot renew`
- Check certificate: `sudo certbot certificates`

### High Memory Usage
- Implement peer cleanup
- Add connection limits
- Use Redis for state storage

## Cost Estimates

### VPS Hosting
- DigitalOcean Droplet: $5-10/month
- Linode Nanode: $5/month
- AWS EC2 t2.micro: ~$8/month

### Serverless
- Heroku: Free tier available
- Railway.app: $5/month
- Fly.io: Free tier available

## Alternative: No Discovery Server

The app works without a discovery server by:
1. Using localStorage for local peer discovery (same browser/machine)
2. Using PeerJS cloud server for WebRTC signaling
3. Manual peer ID sharing (QR codes, URLs, copy-paste)

This is sufficient for most use cases!
