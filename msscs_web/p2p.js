export class P2PNetwork {
    constructor() {
        this.peer = null;
        this.connections = new Map();
        this.eventHandlers = new Map();
        this.peerId = null;
        this.discoveryWs = null;
        this.discoveryEnabled = true;
    }

    async init() {
        // Use PeerJS for WebRTC signaling
        return new Promise((resolve, reject) => {
            // Import PeerJS from CDN
            if (!window.Peer) {
                console.log('📦 Loading PeerJS library from CDN...');
                const script = document.createElement('script');
                script.src = 'https://unpkg.com/peerjs@1.5.2/dist/peerjs.min.js';
                script.onload = () => {
                    console.log('✅ PeerJS library loaded');
                    this.initPeer(resolve, reject);
                };
                script.onerror = () => {
                    console.error('❌ Failed to load PeerJS library from CDN');
                    reject(new Error('Failed to load PeerJS library'));
                };
                document.head.appendChild(script);
            } else {
                console.log('✅ PeerJS library already loaded');
                this.initPeer(resolve, reject);
            }
        });
    }

    initPeer(resolve, reject) {
        try {
            // Enhanced ICE servers for global connectivity with multiple TURN servers
            // This configuration ensures connectivity even behind strict NATs and firewalls
            const iceServers = [
                // ═══════════════════════════════════════════════════════════
                // STUN SERVERS (for NAT type detection and public IP discovery)
                // ═══════════════════════════════════════════════════════════
                
                // Google STUN servers (multiple for redundancy and load balancing)
                { urls: 'stun:stun.l.google.com:19302' },
                { urls: 'stun:stun1.l.google.com:19302' },
                { urls: 'stun:stun2.l.google.com:19302' },
                { urls: 'stun:stun3.l.google.com:19302' },
                { urls: 'stun:stun4.l.google.com:19302' },
                
                // Mozilla STUN (reliable alternative)
                { urls: 'stun:stun.services.mozilla.com' },
                
                // Additional public STUN servers for better global coverage
                { urls: 'stun:stun.stunprotocol.org:3478' },
                { urls: 'stun:stun.voip.blackberry.com:3478' },
                { urls: 'stun:stun.ekiga.net' },
                { urls: 'stun:stun.ideasip.com' },
                
                // ═══════════════════════════════════════════════════════════
                // TURN SERVERS (for relay when direct connection fails)
                // These work even with symmetric NATs and strict firewalls
                // ═══════════════════════════════════════════════════════════
                
                // OpenRelay TURN servers (free, public, reliable)
                // UDP transport (fastest, preferred)
                {
                    urls: 'turn:openrelay.metered.ca:80',
                    username: 'openrelayproject',
                    credential: 'openrelayproject'
                },
                {
                    urls: 'turn:openrelay.metered.ca:443',
                    username: 'openrelayproject',
                    credential: 'openrelayproject'
                },
                
                // TCP transport (works when UDP is blocked)
                {
                    urls: 'turn:openrelay.metered.ca:80?transport=tcp',
                    username: 'openrelayproject',
                    credential: 'openrelayproject'
                },
                {
                    urls: 'turn:openrelay.metered.ca:443?transport=tcp',
                    username: 'openrelayproject',
                    credential: 'openrelayproject'
                },
                
                // TLS transport (works through HTTPS-only proxies)
                {
                    urls: 'turns:openrelay.metered.ca:443?transport=tcp',
                    username: 'openrelayproject',
                    credential: 'openrelayproject'
                }
            ];

            // WebRTC configuration for optimal internet-wide connectivity
            const rtcConfig = {
                iceServers,
                iceTransportPolicy: 'all',      // Use both STUN and TURN (try direct first, fallback to relay)
                iceCandidatePoolSize: 10,       // Pre-gather ICE candidates for faster connections
                bundlePolicy: 'max-bundle',     // Bundle all media on single transport (efficiency)
                rtcpMuxPolicy: 'require'        // Multiplex RTP and RTCP (reduces ports needed)
            };

            // ═══════════════════════════════════════════════════════════
            // PEER SERVER SELECTION (Signaling Server)
            // ═══════════════════════════════════════════════════════════
            
            const isLocalhost = window.location.hostname === 'localhost' || 
                               window.location.hostname === '127.0.0.1';
            
            if (isLocalhost) {
                // LOCAL DEVELOPMENT MODE
                // Try local PeerServer first for same-network testing
                console.log('╔════════════════════════════════════════════════════════════════╗');
                console.log('║  🏠 Local Development Mode                                    ║');
                console.log('╚════════════════════════════════════════════════════════════════╝');
                console.log('🔍 Attempting local PeerServer connection...');
                console.log('   • Host: localhost:9000');
                console.log('   • Use for LAN testing');
                console.log('   • Will fallback to cloud if unavailable');
                
                const peerOptions = {
                    host: 'localhost',
                    port: 9000,
                    path: '/peerjs',
                    config: rtcConfig,
                    debug: 2 // Enable debug logging
                };
                
                this.peer = new Peer(peerOptions);
                
                // Fallback to cloud after 3 seconds if local server not available
                let localServerFailed = false;
                setTimeout(() => {
                    if (!this.peerId && !localServerFailed) {
                        localServerFailed = true;
                        console.warn('⚠️  Local PeerServer not available (this is normal)');
                        console.log('🌍 Switching to cloud PeerJS server for internet-wide connectivity');
                        this.peer.destroy();
                        this.peer = new Peer({ 
                            config: rtcConfig,
                            debug: 2
                        });
                        this.setupPeerHandlers(resolve, reject);
                    }
                }, 3000);
            } else {
                // PRODUCTION MODE - Cloud PeerJS for global connectivity
                console.log('╔════════════════════════════════════════════════════════════════╗');
                console.log('║  🌍 Global P2P Network Mode (Internet-Wide)                  ║');
                console.log('╚════════════════════════════════════════════════════════════════╝');
                console.log('');
                console.log('📡 Network Configuration:');
                console.log('   • Signaling: Cloud PeerJS server (global)');
                console.log('   • STUN servers: 10 configured (NAT detection)');
                console.log('   • TURN servers: 5 configured (relay fallback)');
                console.log('   • ICE candidate pool: 10 (faster connections)');
                console.log('');
                console.log('🔧 Connectivity Features:');
                console.log('   ✓ Works behind NAT/firewalls');
                console.log('   ✓ No port forwarding needed');
                console.log('   ✓ Automatic relay fallback');
                console.log('   ✓ Direct P2P when possible');
                console.log('   ✓ Mobile network compatible (4G/5G)');
                console.log('');
                
                this.peer = new Peer({ 
                    config: rtcConfig,
                    debug: 2
                });
            }

            this.setupPeerHandlers(resolve, reject);
        } catch (error) {
            console.error('❌ Failed to initialize peer:', error);
            reject(error);
        }
    }

    setupPeerHandlers(resolve, reject) {
        let resolved = false;

        this.peer.on('open', (id) => {
            this.peerId = id;
            console.log('');
            console.log('╔════════════════════════════════════════════════════════════════╗');
            console.log('║  ✅ P2P Node Successfully Connected                           ║');
            console.log('╚════════════════════════════════════════════════════════════════╝');
            console.log('');
            console.log('🆔 Your Peer ID:', id);
            console.log('');
            console.log('🌐 You are now part of the global P2P network!');
            console.log('');
            console.log('📍 How to connect with others:');
            console.log('   1. Share your Peer ID with others');
            console.log('   2. Or share this URL:', `${window.location.origin}?peer=${id}`);
            console.log('   3. Others can connect by entering your Peer ID');
            console.log('');
            console.log('🔍 Peer Discovery:');
            console.log('   • Local network: Automatic (via localStorage)');
            console.log('   • Internet: Share Peer ID or URL');
            console.log('   • Discovery server: Optional (faster discovery)');
            console.log('');
            
            this.emit('ready', id);
            if (!resolved) {
                resolved = true;
                resolve();
            }
            
            // Auto-discover local peers via localStorage broadcast
            console.log('🔄 Starting peer discovery...');
            this.broadcastPresence();
            this.discoverLocalPeers();
            
            // Connect to discovery server (optional, for faster discovery)
            this.connectToDiscoveryServer();
            
            // Auto-connect from URL if peer ID provided
            this.autoConnectFromUrl();
        });

        this.peer.on('connection', (conn) => {
            this.handleConnection(conn);
        });

        this.peer.on('error', (err) => {
            console.error('❌ Peer error:', err.type || 'unknown', '-', err.message);
            
            // Log specific error types for debugging
            if (err.type === 'network') {
                console.error('   Network error - check internet connection');
            } else if (err.type === 'peer-unavailable') {
                console.error('   Peer unavailable - they may be offline');
            } else if (err.type === 'server-error') {
                console.error('   PeerJS server error - trying fallback...');
            }
            
            this.emit('error', err.message);
            if (!resolved) {
                resolved = true;
                // Don't reject - allow offline mode
                console.warn('⚠️  P2P connection failed, running in offline mode');
                console.log('💡 You can still use the app locally');
                resolve();
            }
        });

        // Timeout after 10 seconds
        setTimeout(() => {
            if (!resolved) {
                resolved = true;
                console.warn('⚠️  P2P connection timeout, running in offline mode');
                resolve();
            }
        }, 10000);
    }

    // Broadcast presence to local network via localStorage
    broadcastPresence() {
        if (!this.peerId) return;
        
        const presence = {
            peerId: this.peerId,
            timestamp: Date.now()
        };
        
        // Store in localStorage for cross-tab communication
        localStorage.setItem('msscs_peer_' + this.peerId, JSON.stringify(presence));
        
        // Broadcast every 5 seconds
        setInterval(() => {
            presence.timestamp = Date.now();
            localStorage.setItem('msscs_peer_' + this.peerId, JSON.stringify(presence));
        }, 5000);
        
        // Clean up old entries
        this.cleanupOldPeers();
    }

    // Discover local peers via localStorage
    discoverLocalPeers() {
        // CRITICAL FIX: Recognize self in localStorage to prevent "always syncing" issue
        console.log('🔍 Starting local peer discovery...');
        console.log('   My Peer ID:', this.peerId);
        
        // Check localStorage for other peers
        for (let i = 0; i < localStorage.length; i++) {
            const key = localStorage.key(i);
            if (key && key.startsWith('msscs_peer_')) {
                try {
                    const data = JSON.parse(localStorage.getItem(key));
                    const peerId = key.replace('msscs_peer_', '');
                    
                    // CRITICAL FIX: Skip self to avoid self-connection attempts
                    if (peerId === this.peerId) {
                        console.log('   ✓ Found self in localStorage (node recognized)');
                        continue;
                    }
                    
                    // Check if peer is recent (within last 30 seconds)
                    if (Date.now() - data.timestamp < 30000) {
                        console.log('🔍 Discovered local peer:', peerId);
                        // Auto-connect to local peer
                        setTimeout(() => this.connectToPeer(peerId), 1000);
                    }
                } catch (e) {
                    console.error('Error parsing peer data:', e);
                }
            }
        }
        
        // Re-discover every 10 seconds
        setInterval(() => {
            this.cleanupOldPeers();
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (key && key.startsWith('msscs_peer_')) {
                    try {
                        const data = JSON.parse(localStorage.getItem(key));
                        const peerId = key.replace('msscs_peer_', '');
                        
                        // CRITICAL FIX: Skip self
                        if (peerId === this.peerId) {
                            continue;
                        }
                        
                        if (Date.now() - data.timestamp < 30000 && !this.connections.has(peerId)) {
                            console.log('🔍 Discovered local peer:', peerId);
                            this.connectToPeer(peerId);
                        }
                    } catch (e) {
                        // Ignore errors
                    }
                }
            }
        }, 10000);
    }

    cleanupOldPeers() {
        const keysToRemove = [];
        for (let i = 0; i < localStorage.length; i++) {
            const key = localStorage.key(i);
            if (key && key.startsWith('msscs_peer_')) {
                try {
                    const data = JSON.parse(localStorage.getItem(key));
                    // Remove entries older than 30 seconds
                    if (Date.now() - data.timestamp > 30000) {
                        keysToRemove.push(key);
                    }
                } catch (e) {
                    keysToRemove.push(key);
                }
            }
        }
        keysToRemove.forEach(key => localStorage.removeItem(key));
    }

    // Connect to discovery server (local or public)
    connectToDiscoveryServer() {
        if (!this.discoveryEnabled || !this.peerId) return;

        try {
            // Try local discovery first, then public discovery
            const isLocalhost = window.location.hostname === 'localhost' || 
                               window.location.hostname === '127.0.0.1';
            
            // Check for custom discovery server in localStorage or environment
            let wsUrl = localStorage.getItem('msscs_discovery_server');
            
            if (!wsUrl) {
                // For localhost, try local discovery server
                // For production, use public discovery server if configured
                wsUrl = isLocalhost 
                    ? 'ws://localhost:9001'
                    : (window.MSSCS_DISCOVERY_SERVER || null);
            }
            
            // Skip if no server configured (rely on localStorage peer discovery)
            if (!wsUrl) {
                console.log('💡 No discovery server configured - using localStorage peer discovery only');
                console.log('   For better connectivity, set window.MSSCS_DISCOVERY_SERVER or localStorage.msscs_discovery_server');
                console.log('   Example: localStorage.setItem("msscs_discovery_server", "wss://discovery.example.com")');
                return;
            }
            
            console.log('🔍 Connecting to discovery server:', wsUrl);
            
            this.discoveryWs = new WebSocket(wsUrl);

            this.discoveryWs.onopen = () => {
                console.log('✅ Connected to local discovery server');
                
                // Register with discovery server
                this.discoveryWs.send(JSON.stringify({
                    type: 'register',
                    peerId: this.peerId,
                    peerType: 'web'
                }));

                // Send heartbeat every 20 seconds
                setInterval(() => {
                    if (this.discoveryWs && this.discoveryWs.readyState === WebSocket.OPEN) {
                        this.discoveryWs.send(JSON.stringify({
                            type: 'heartbeat'
                        }));
                    }
                }, 20000);
            };

            this.discoveryWs.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    
                    switch (message.type) {
                        case 'peer-list':
                            console.log('📋 Received peer list:', message.peers);
                            // Auto-connect to discovered peers
                            message.peers.forEach(peer => {
                                // CRITICAL FIX: Skip self in discovery server peer list
                                if (peer.peerId === this.peerId) {
                                    console.log('   ✓ Found self in peer list (skipping)');
                                    return;
                                }
                                if (!this.connections.has(peer.peerId)) {
                                    console.log(`🔗 Auto-connecting to ${peer.type} peer:`, peer.peerId);
                                    setTimeout(() => this.connectToPeer(peer.peerId), 500);
                                }
                            });
                            break;

                        case 'peer-joined':
                            console.log('👋 New peer joined:', message.peerId, `(${message.peerType})`);
                            // CRITICAL FIX: Skip self in peer-joined events
                            if (message.peerId === this.peerId) {
                                console.log('   ✓ Self-join event (skipping)');
                                return;
                            }
                            // Auto-connect to new peer
                            if (!this.connections.has(message.peerId)) {
                                setTimeout(() => this.connectToPeer(message.peerId), 1000);
                            }
                            break;

                        case 'peer-left':
                            console.log('👋 Peer left:', message.peerId);
                            break;

                        case 'signal':
                            // Handle WebRTC signaling if needed
                            console.log('📡 Received signal from:', message.fromPeerId);
                            break;
                    }
                } catch (error) {
                    console.error('Error handling discovery message:', error);
                }
            };

            this.discoveryWs.onerror = (error) => {
                console.warn('⚠️  Discovery server connection error:', error.message);
            };

            this.discoveryWs.onclose = () => {
                console.log('❌ Disconnected from discovery server');
                // Try to reconnect after 5 seconds
                setTimeout(() => {
                    if (this.discoveryEnabled && this.peerId) {
                        console.log('🔄 Reconnecting to discovery server...');
                        this.connectToDiscoveryServer();
                    }
                }, 5000);
            };
        } catch (error) {
            console.warn('⚠️  Could not connect to local discovery server:', error.message);
            console.log('💡 Tip: Start the discovery server with: node discovery-server.js');
        }
    }

    handleConnection(conn) {
        console.log('📥 Incoming connection from:', conn.peer);

        conn.on('open', () => {
            this.connections.set(conn.peer, conn);
            this.emit('peer-connected', conn.peer);
            console.log('✅ Connection established with:', conn.peer);
            
            // Monitor ICE connection state for diagnostics
            if (conn.peerConnection) {
                conn.peerConnection.oniceconnectionstatechange = () => {
                    const state = conn.peerConnection.iceConnectionState;
                    console.log(`🧊 ICE state for ${conn.peer}: ${state}`);
                    
                    if (state === 'failed') {
                        console.warn('⚠️  ICE connection failed, attempting restart...');
                        conn.peerConnection.restartIce();
                    } else if (state === 'disconnected') {
                        console.warn('⚠️  ICE disconnected, monitoring for reconnection...');
                    }
                };
                
                conn.peerConnection.onicegatheringstatechange = () => {
                    console.log(`🧊 ICE gathering for ${conn.peer}: ${conn.peerConnection.iceGatheringState}`);
                };
            }
            
            // Send handshake with capabilities
            conn.send({
                type: 'handshake',
                peerId: this.peerId,
                clientType: 'web',
                capabilities: {
                    storage: true,
                    relay: true,
                    version: '1.0.0'
                }
            });
            
            // Request peer list for peer exchange
            conn.send({
                type: 'get-peer-list'
            });
        });

        conn.on('data', (data) => {
            this.handleMessage(conn.peer, data);
        });

        conn.on('close', () => {
            console.log('❌ Connection closed with:', conn.peer);
            this.connections.delete(conn.peer);
            this.emit('peer-disconnected', conn.peer);
        });

        conn.on('error', (err) => {
            console.error('⚠️  Connection error with', conn.peer, ':', err);
            // Try to reconnect after a delay if it's a temporary error
            if (err.type === 'network' || err.type === 'peer-unavailable') {
                console.log('🔄 Will retry connection in 5 seconds...');
                setTimeout(() => {
                    if (!this.connections.has(conn.peer)) {
                        this.connectToPeer(conn.peer);
                    }
                }, 5000);
            }
        });
    }

    connectToPeer(peerId) {
        // CRITICAL FIX: Prevent self-connection attempts
        if (peerId === this.peerId) {
            console.warn('⚠️  Skipping self-connection attempt');
            return Promise.reject(new Error('Cannot connect to self'));
        }
        
        if (this.connections.has(peerId)) {
            console.log('ℹ️  Already connected to', peerId);
            return Promise.resolve();
        }

        if (!this.peer) {
            console.error('❌ Peer not initialized - cannot connect');
            return Promise.reject(new Error('Peer not initialized'));
        }

        console.log('');
        console.log('╔════════════════════════════════════════════════════════════════╗');
        console.log('║  🔗 Establishing P2P Connection                               ║');
        console.log('╚════════════════════════════════════════════════════════════════╝');
        console.log('');
        console.log('🎯 Target Peer:', peerId);
        console.log('   My Peer ID:', this.peerId);
        console.log('');
        console.log('📡 Connection Process:');
        console.log('   1. Signaling via PeerJS server...');
        console.log('   2. Exchanging ICE candidates...');
        console.log('   3. Testing STUN servers (NAT detection)...');
        console.log('   4. Attempting direct connection...');
        console.log('   5. Fallback to TURN relay if needed...');
        console.log('');
        console.log('⏳ This may take 5-30 seconds depending on network conditions...');
        console.log('');
        
        return new Promise((resolve, reject) => {
            try {
                const conn = this.peer.connect(peerId, {
                    reliable: true,
                    serialization: 'json',
                    metadata: {
                        clientType: 'web',
                        timestamp: Date.now(),
                        version: '1.0.0'
                    }
                });

                // Add timeout for connection attempt (30 seconds)
                const connectionTimeout = setTimeout(() => {
                    if (!this.connections.has(peerId)) {
                        console.warn('');
                        console.warn('⚠️  Connection timeout after 30 seconds');
                        console.warn('   Possible reasons:');
                        console.warn('   • Peer is offline');
                        console.warn('   • Peer ID is incorrect');
                        console.warn('   • Network connectivity issues');
                        console.warn('   • Both peers behind symmetric NAT (rare)');
                        console.warn('');
                        conn.close();
                        reject(new Error('Connection timeout'));
                    }
                }, 30000);

                conn.on('open', () => {
                    clearTimeout(connectionTimeout);
                    console.log('');
                    console.log('╔════════════════════════════════════════════════════════════════╗');
                    console.log('║  ✅ P2P Connection Established Successfully                   ║');
                    console.log('╚════════════════════════════════════════════════════════════════╝');
                    console.log('');
                    console.log('🎉 Connected to peer:', peerId);
                    console.log('');
                    console.log('🔒 Connection is encrypted and secure');
                    console.log('📦 Ready to exchange data');
                    console.log('');
                    resolve();
                });

                conn.on('error', (err) => {
                    clearTimeout(connectionTimeout);
                    console.error('');
                    console.error('❌ Connection failed:', err.type || err.message);
                    console.error('');
                    reject(err);
                });

                this.handleConnection(conn);
            } catch (error) {
                console.error('');
                console.error('❌ Failed to initiate connection:', error.message);
                console.error('');
                reject(error);
            }
        });
    }
    
    // Batch connect to multiple peers with staggered timing
    async connectToPeers(peerIds) {
        console.log(`🔗 Batch connecting to ${peerIds.length} peers...`);
        
        // Stagger connections to avoid overwhelming the network
        const results = [];
        for (let i = 0; i < peerIds.length; i++) {
            const peerId = peerIds[i];
            // Add small delay between connections
            if (i > 0) {
                await new Promise(resolve => setTimeout(resolve, 500));
            }
            results.push(this.connectToPeer(peerId).catch(err => ({ error: err })));
        }
        
        const settled = await Promise.allSettled(results);
        const successful = settled.filter(r => r.status === 'fulfilled' && !r.value?.error).length;
        const failed = settled.filter(r => r.status === 'rejected' || r.value?.error).length;
        
        console.log(`📊 Connection results: ${successful} successful, ${failed} failed`);
        return { successful, failed };
    }

    // Get shareable connection info with multiple formats
    getConnectionInfo() {
        const connectionUrl = `${window.location.origin}?peer=${this.peerId}`;
        const deepLink = `msscs://connect/${this.peerId}`;
        
        return {
            peerId: this.peerId,
            connectionUrl,
            deepLink,
            shareText: `Connect to my MSSCS node:\nPeer ID: ${this.peerId}\nOr visit: ${connectionUrl}`,
            qrCode: this.generateQRCodeUrl(),
            qrCodeData: this.peerId
        };
    }

    generateQRCodeUrl() {
        if (!this.peerId) return null;
        const url = `${window.location.origin}?peer=${this.peerId}`;
        return `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(url)}`;
    }

    // Copy peer ID to clipboard
    async copyPeerIdToClipboard() {
        if (!this.peerId) return false;
        try {
            await navigator.clipboard.writeText(this.peerId);
            return true;
        } catch (error) {
            console.error('Failed to copy to clipboard:', error);
            return false;
        }
    }

    // Get connection statistics with detailed info
    getConnectionStats() {
        const isConnected = this.peer !== null && !this.peer.destroyed;
        const stats = {
            peerId: this.peerId,
            connectedPeers: this.connections.size,
            peers: Array.from(this.connections.keys()),
            isConnected: isConnected,
            discoveryConnected: this.discoveryWs?.readyState === WebSocket.OPEN,
            connectionTypes: this.getConnectionTypes(),
            // Additional stats for UI
            totalConnections: this.connections.size,
            activeConnections: Array.from(this.connections.values()).filter(c => c.open).length,
            signalServerConnected: this.peer && !this.peer.disconnected,
            // Status summary for display
            statusText: isConnected 
                ? (this.connections.size > 0 
                    ? `Online - ${this.connections.size} peer${this.connections.size !== 1 ? 's' : ''}`
                    : 'Online - Discovering peers...')
                : 'Offline'
        };
        return stats;
    }
    
    // Get connection types (direct vs relayed)
    getConnectionTypes() {
        const types = { direct: 0, relayed: 0, unknown: 0 };
        for (const conn of this.connections.values()) {
            if (conn.peerConnection) {
                const pc = conn.peerConnection;
                if (pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') {
                    // Check if using relay
                    pc.getStats().then(stats => {
                        stats.forEach(report => {
                            if (report.type === 'candidate-pair' && report.state === 'succeeded') {
                                if (report.remoteCandidateId) {
                                    stats.forEach(candidate => {
                                        if (candidate.id === report.remoteCandidateId) {
                                            if (candidate.candidateType === 'relay') {
                                                types.relayed++;
                                            } else {
                                                types.direct++;
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    }).catch(() => types.unknown++);
                }
            }
        }
        return types;
    }

    // Share connection via Web Share API (mobile-friendly)
    async shareConnection() {
        if (!this.peerId) return false;
        
        const shareData = {
            title: 'Connect to my MSSCS Node',
            text: `Join my MSSCS storage network!\nPeer ID: ${this.peerId}`,
            url: `${window.location.origin}?peer=${this.peerId}`
        };

        try {
            if (navigator.share) {
                await navigator.share(shareData);
                return true;
            } else {
                // Fallback to clipboard
                return await this.copyPeerIdToClipboard();
            }
        } catch (error) {
            console.error('Failed to share:', error);
            return false;
        }
    }

    // Auto-connect from URL parameter
    autoConnectFromUrl() {
        const urlParams = new URLSearchParams(window.location.search);
        const peerIdToConnect = urlParams.get('peer');
        
        if (peerIdToConnect && peerIdToConnect !== this.peerId) {
            console.log('🔗 Auto-connecting to peer from URL:', peerIdToConnect);
            setTimeout(() => this.connectToPeer(peerIdToConnect), 2000);
        }
    }

    handleMessage(peerId, data) {
        switch (data.type) {
            case 'handshake':
                console.log('🤝 Handshake from:', data.peerId, data.clientType || 'unknown');
                if (data.capabilities) {
                    console.log('   Capabilities:', data.capabilities);
                }
                // Send peer list for peer exchange
                this.sendPeerList(peerId);
                break;
                
            case 'peer-list':
                console.log('📋 Received peer list from', peerId, ':', data.peers.length, 'peers');
                // Auto-connect to new peers (peer exchange)
                data.peers.forEach(remotePeerId => {
                    if (remotePeerId !== this.peerId && !this.connections.has(remotePeerId)) {
                        console.log('🔗 Discovered peer via exchange:', remotePeerId);
                        setTimeout(() => this.connectToPeer(remotePeerId), Math.random() * 2000);
                    }
                });
                break;
                
            case 'block-request':
                console.log('📤 Block request from', peerId, ':', data.blockId);
                this.emit('block-request', {
                    blockId: data.blockId,
                    peerId
                });
                break;
                
            case 'block-response':
                console.log('📦 Block response from', peerId);
                this.emit('block-received', {
                    block: data.block,
                    peerId
                });
                break;
                
            case 'block-broadcast':
                console.log('📡 Block broadcast from', peerId);
                this.emit('block-received', {
                    block: data.block,
                    peerId
                });
                break;
                
            case 'storage-stats':
                console.log('📊 Storage stats from', peerId, ':', data.stats);
                this.emit('peer-stats', {
                    peerId,
                    stats: data.stats
                });
                break;
                
            default:
                console.log('Unknown message type:', data.type);
        }
    }
    
    // Send peer list to a peer for peer exchange
    sendPeerList(peerId) {
        const conn = this.connections.get(peerId);
        if (conn) {
            const peerList = Array.from(this.connections.keys()).filter(id => id !== peerId);
            conn.send({
                type: 'peer-list',
                peers: peerList
            });
        }
    }

    sendBlock(peerId, block) {
        const conn = this.connections.get(peerId);
        if (conn) {
            conn.send({
                type: 'block-response',
                block
            });
        }
    }

    broadcastBlock(block) {
        for (const conn of this.connections.values()) {
            conn.send({
                type: 'block-broadcast',
                block
            });
        }
    }
    
    // Broadcast storage statistics to all peers
    broadcastStorageStats(stats) {
        for (const conn of this.connections.values()) {
            conn.send({
                type: 'storage-stats',
                stats: {
                    storage_used: stats.storage_used,
                    storage_limit: stats.storage_limit,
                    storage_available: stats.storage_available,
                    total_files: stats.total_files,
                    timestamp: Date.now()
                }
            });
        }
    }

    async requestBlock(blockId) {
        return new Promise((resolve) => {
            const timeout = setTimeout(() => resolve(null), 5000);
            
            const handler = (data) => {
                if (data.block && data.block.id === blockId) {
                    clearTimeout(timeout);
                    this.off('block-received', handler);
                    resolve(data.block);
                }
            };
            
            this.on('block-received', handler);
            
            // Request from all peers
            for (const conn of this.connections.values()) {
                conn.send({
                    type: 'block-request',
                    blockId
                });
            }
        });
    }

    getPeers() {
        return Array.from(this.connections.keys()).map(id => ({ id }));
    }

    on(event, handler) {
        if (!this.eventHandlers.has(event)) {
            this.eventHandlers.set(event, []);
        }
        this.eventHandlers.get(event).push(handler);
    }

    off(event, handler) {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            const index = handlers.indexOf(handler);
            if (index > -1) {
                handlers.splice(index, 1);
            }
        }
    }

    emit(event, data) {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            handlers.forEach(handler => handler(data));
        }
    }
}
