// Peer discovery server using WebSocket (supports both local and internet-wide discovery)
import { createServer } from 'http';
import { createServer as createHttpsServer } from 'https';
import { readFileSync } from 'fs';
import { WebSocketServer } from 'ws';

const PORT = process.env.PORT || 9001;
const USE_HTTPS = process.env.USE_HTTPS === 'true';
const CERT_PATH = process.env.CERT_PATH || './cert.pem';
const KEY_PATH = process.env.KEY_PATH || './key.pem';
const PUBLIC_URL = process.env.PUBLIC_URL || `ws://localhost:${PORT}`;

const peers = new Map(); // peerId -> { ws, type, timestamp, metadata }

// Create HTTP or HTTPS server based on configuration
let server;
if (USE_HTTPS) {
    try {
        const options = {
            cert: readFileSync(CERT_PATH),
            key: readFileSync(KEY_PATH)
        };
        server = createHttpsServer(options);
        console.log('✅ HTTPS server enabled');
    } catch (error) {
        console.warn('⚠️  Failed to load SSL certificates, falling back to HTTP');
        server = createServer();
    }
} else {
    server = createServer();
}

const wss = new WebSocketServer({ 
    server,
    perMessageDeflate: {
        zlibDeflateOptions: {
            chunkSize: 1024,
            memLevel: 7,
            level: 3
        },
        zlibInflateOptions: {
            chunkSize: 10 * 1024
        },
        clientNoContextTakeover: true,
        serverNoContextTakeover: true,
        serverMaxWindowBits: 10,
        concurrencyLimit: 10,
        threshold: 1024
    }
});

wss.on('connection', (ws, req) => {
    let peerId = null;
    let peerType = null;
    const clientIp = req.socket.remoteAddress;

    console.log(`📥 New connection from ${clientIp}`);

    ws.on('message', (data) => {
        try {
            const message = JSON.parse(data.toString());

            switch (message.type) {
                case 'register':
                    peerId = message.peerId;
                    peerType = message.peerType || 'unknown';
                    const metadata = {
                        ws,
                        type: peerType,
                        timestamp: Date.now(),
                        ip: clientIp,
                        userAgent: req.headers['user-agent'] || 'unknown'
                    };
                    peers.set(peerId, metadata);
                    console.log(`✅ Peer registered: ${peerId} (${peerType}) from ${clientIp}`);
                    
                    // Send current peer list (with privacy - don't expose IPs)
                    const peerList = Array.from(peers.entries())
                        .filter(([id]) => id !== peerId)
                        .map(([id, info]) => ({ 
                            peerId: id, 
                            type: info.type,
                            online: Date.now() - info.timestamp < 60000
                        }));
                    
                    ws.send(JSON.stringify({
                        type: 'peer-list',
                        peers: peerList,
                        totalPeers: peers.size
                    }));
                    
                    // Notify other peers
                    broadcast({
                        type: 'peer-joined',
                        peerId,
                        peerType
                    }, peerId);
                    break;

                case 'signal':
                    // Forward signaling data to target peer (for WebRTC)
                    const targetPeer = peers.get(message.targetPeerId);
                    if (targetPeer && targetPeer.ws.readyState === 1) {
                        targetPeer.ws.send(JSON.stringify({
                            type: 'signal',
                            fromPeerId: peerId,
                            data: message.data
                        }));
                    } else {
                        ws.send(JSON.stringify({
                            type: 'error',
                            message: 'Target peer not available'
                        }));
                    }
                    break;

                case 'heartbeat':
                    if (peerId && peers.has(peerId)) {
                        peers.get(peerId).timestamp = Date.now();
                        ws.send(JSON.stringify({ 
                            type: 'heartbeat-ack',
                            timestamp: Date.now(),
                            connectedPeers: peers.size
                        }));
                    }
                    break;

                case 'get-stats':
                    // Return server statistics
                    ws.send(JSON.stringify({
                        type: 'stats',
                        totalPeers: peers.size,
                        peerTypes: Array.from(peers.values()).reduce((acc, p) => {
                            acc[p.type] = (acc[p.type] || 0) + 1;
                            return acc;
                        }, {})
                    }));
                    break;
            }
        } catch (error) {
            console.error('Error handling message:', error);
            ws.send(JSON.stringify({
                type: 'error',
                message: 'Invalid message format'
            }));
        }
    });

    ws.on('close', () => {
        if (peerId) {
            peers.delete(peerId);
            console.log(`❌ Peer disconnected: ${peerId}`);
            
            // Notify other peers
            broadcast({
                type: 'peer-left',
                peerId
            }, peerId);
        }
    });

    ws.on('error', (error) => {
        console.error('WebSocket error:', error);
    });
});

function broadcast(message, excludePeerId = null) {
    const data = JSON.stringify(message);
    for (const [peerId, peer] of peers.entries()) {
        if (peerId !== excludePeerId && peer.ws.readyState === 1) {
            peer.ws.send(data);
        }
    }
}

// Clean up stale peers every 30 seconds
setInterval(() => {
    const now = Date.now();
    const staleTimeout = 60000; // 60 seconds
    
    for (const [peerId, peer] of peers.entries()) {
        if (now - peer.timestamp > staleTimeout) {
            console.log(`🧹 Removing stale peer: ${peerId}`);
            peer.ws.close();
            peers.delete(peerId);
            
            broadcast({
                type: 'peer-left',
                peerId
            }, peerId);
        }
    }
}, 30000);

server.listen(PORT, '0.0.0.0', () => {
    const protocol = USE_HTTPS ? 'wss' : 'ws';
    const host = process.env.HOST || 'localhost';
    
    console.log(`
╔════════════════════════════════════════════════════════╗
║           MSSCS Discovery Server                       ║
╠════════════════════════════════════════════════════════╣
║  WebSocket server: ${protocol}://${host}:${PORT.toString().padEnd(20)}║
║  Public URL: ${PUBLIC_URL.padEnd(37)}║
║  Protocol: ${(USE_HTTPS ? 'WSS (Secure)' : 'WS (Insecure)').padEnd(37)}║
║  Listening on: 0.0.0.0:${PORT.toString().padEnd(31)}║
║                                                        ║
║  This server helps peers discover each other           ║
║  - Local network discovery (same machine/LAN)          ║
║  - Internet-wide discovery (with public deployment)    ║
║                                                        ║
║  Environment Variables:                                ║
║  - PORT: ${PORT.toString().padEnd(43)}║
║  - USE_HTTPS: ${(USE_HTTPS ? 'true' : 'false').padEnd(38)}║
║  - HOST: ${host.padEnd(43)}║
║  - PUBLIC_URL: ${PUBLIC_URL.padEnd(37)}║
╚════════════════════════════════════════════════════════╝

📊 Server Status:
   ✓ Ready to accept connections from anywhere
   ✓ Compression enabled (per-message deflate)
   ✓ Heartbeat monitoring active
   ✓ Auto-cleanup enabled (60s timeout)

💡 Tips for Internet-Wide Deployment:
   1. Deploy on a public server (VPS, cloud, etc.)
   2. Set USE_HTTPS=true and provide SSL certificates
   3. Configure firewall to allow port ${PORT}
   4. Set PUBLIC_URL to your public domain/IP
   5. Use reverse proxy (nginx/caddy) for SSL termination
   
   Example nginx config:
   location /discovery {
       proxy_pass http://localhost:${PORT};
       proxy_http_version 1.1;
       proxy_set_header Upgrade $http_upgrade;
       proxy_set_header Connection "upgrade";
   }
    `);
});
