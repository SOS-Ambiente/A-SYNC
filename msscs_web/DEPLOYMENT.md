# MSSCS Web Deployment Guide

## Quick Start (Local Development)

### Windows
```powershell
.\start.ps1
```

### Linux/Mac
```bash
chmod +x start.sh
./start.sh
```

### Alternative (Python)
```bash
python -m http.server 8000
```

Then open `http://localhost:8000` in your browser.

## Production Deployment

### 1. Static Hosting (Recommended)

Deploy to any static hosting service:

#### Netlify
```bash
# Install Netlify CLI
npm install -g netlify-cli

# Deploy
netlify deploy --prod
```

#### Vercel
```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
vercel --prod
```

#### GitHub Pages
```bash
# Push to GitHub
git init
git add .
git commit -m "Initial commit"
git branch -M main
git remote add origin https://github.com/yourusername/msscs-web.git
git push -u origin main

# Enable GitHub Pages in repository settings
# Set source to main branch
```

#### Cloudflare Pages
1. Push code to GitHub
2. Connect repository to Cloudflare Pages
3. Build settings: None (static site)
4. Deploy

### 2. Docker Deployment

Create `Dockerfile`:
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY . .
EXPOSE 8000
CMD ["node", "server.js"]
```

Build and run:
```bash
docker build -t msscs-web .
docker run -p 8000:8000 msscs-web
```

### 3. Nginx Deployment

Create `/etc/nginx/sites-available/msscs-web`:
```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/msscs-web;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # Enable CORS for WebRTC
    add_header Access-Control-Allow-Origin *;
}
```

Enable and restart:
```bash
sudo ln -s /etc/nginx/sites-available/msscs-web /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### 4. Apache Deployment

Create `.htaccess`:
```apache
RewriteEngine On
RewriteBase /
RewriteRule ^index\.html$ - [L]
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule . /index.html [L]

# Enable CORS
Header set Access-Control-Allow-Origin "*"
```

## HTTPS Setup (Required for WebRTC)

WebRTC requires HTTPS in production. Use Let's Encrypt:

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal
sudo certbot renew --dry-run
```

## Environment Configuration

### Custom Port
```bash
PORT=3000 node server.js
```

### Custom STUN/TURN Servers

Edit `p2p.js`:
```javascript
this.peer = new Peer({
    config: {
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' },
            { 
                urls: 'turn:your-turn-server.com:3478',
                username: 'user',
                credential: 'pass'
            }
        ]
    }
});
```

## Performance Optimization

### 1. Enable Compression

Nginx:
```nginx
gzip on;
gzip_types text/plain text/css application/json application/javascript;
```

### 2. Cache Static Assets

Nginx:
```nginx
location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

### 3. CDN Integration

Use a CDN for static assets:
- Cloudflare
- AWS CloudFront
- Fastly

## Monitoring

### Basic Logging

Add to `server.js`:
```javascript
import { appendFile } from 'fs/promises';

server.on('request', async (req) => {
    const log = `${new Date().toISOString()} ${req.method} ${req.url}\n`;
    await appendFile('access.log', log);
});
```

### Error Tracking

Integrate Sentry:
```html
<script src="https://browser.sentry-cdn.com/7.x.x/bundle.min.js"></script>
<script>
  Sentry.init({ dsn: 'YOUR_DSN' });
</script>
```

## Security Considerations

### 1. Content Security Policy

Add to `index.html`:
```html
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; 
               connect-src 'self' wss: https:; 
               script-src 'self' 'unsafe-inline' https://unpkg.com;">
```

### 2. Rate Limiting

Use Nginx:
```nginx
limit_req_zone $binary_remote_addr zone=one:10m rate=10r/s;
limit_req zone=one burst=20;
```

### 3. DDoS Protection

Use Cloudflare or similar service for:
- DDoS mitigation
- Bot protection
- Rate limiting

## Scaling

### Horizontal Scaling

Deploy multiple instances behind a load balancer:

```nginx
upstream msscs_web {
    server 10.0.0.1:8000;
    server 10.0.0.2:8000;
    server 10.0.0.3:8000;
}

server {
    location / {
        proxy_pass http://msscs_web;
    }
}
```

### TURN Server Setup

For better NAT traversal, deploy your own TURN server:

```bash
# Install coturn
sudo apt install coturn

# Configure /etc/turnserver.conf
listening-port=3478
fingerprint
lt-cred-mech
user=username:password
realm=your-domain.com
```

## Backup Strategy

### IndexedDB Backup

Add export functionality:
```javascript
async exportData() {
    const files = await this.storage.getAllFiles();
    const blob = new Blob([JSON.stringify(files)], { type: 'application/json' });
    // Download blob
}
```

### Automated Backups

Users should periodically export their encryption keys:
```javascript
const key = await app.crypto.exportKey();
// Save key securely
```

## Troubleshooting

### WebRTC Not Working
- Ensure HTTPS is enabled
- Check firewall settings
- Verify STUN/TURN servers are accessible
- Test with different browsers

### Storage Quota Issues
- Clear old data
- Implement storage limits
- Add quota monitoring

### Performance Issues
- Enable compression
- Optimize chunk size
- Implement lazy loading
- Use Web Workers for encryption

## Maintenance

### Regular Updates
```bash
# Update dependencies
npm update

# Security audit
npm audit
```

### Monitoring Checklist
- [ ] Server uptime
- [ ] SSL certificate expiry
- [ ] Storage usage
- [ ] Error rates
- [ ] User connections

## Support

For deployment issues:
1. Check browser console for errors
2. Verify network connectivity
3. Test with different browsers
4. Check server logs
5. Review firewall rules
