// PeerJS bridge for Tauri app to connect with web peers
import Peer, { DataConnection } from 'peerjs';

export class PeerJSBridge {
    private peer: Peer | null = null;
    private connections: Map<string, DataConnection> = new Map();
    private peerId: string | null = null;
    private discoveryWs: WebSocket | null = null;
    private eventHandlers: Map<string, Function[]> = new Map();

    async init(): Promise<string> {
        console.log('🌉 Initializing PeerJS bridge for Tauri...');
        return new Promise((resolve, reject) => {
            try {
                // Enhanced ICE servers for global internet connectivity
                const iceServers = [
                    // Google STUN servers (multiple for redundancy)
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:stun1.l.google.com:19302' },
                    { urls: 'stun:stun2.l.google.com:19302' },
                    { urls: 'stun:stun3.l.google.com:19302' },
                    { urls: 'stun:stun4.l.google.com:19302' },
                    
                    // Mozilla STUN
                    { urls: 'stun:stun.services.mozilla.com' },
                    
                    // Additional public STUN servers
                    { urls: 'stun:stun.stunprotocol.org:3478' },
                    { urls: 'stun:stun.voip.blackberry.com:3478' },
                    
                    // Free TURN servers for NAT traversal (helps with strict firewalls and symmetric NATs)
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
                    {
                        urls: 'turn:openrelay.metered.ca:443?transport=tcp',
                        username: 'openrelayproject',
                        credential: 'openrelayproject'
                    },
                    {
                        urls: 'turn:openrelay.metered.ca:80?transport=tcp',
                        username: 'openrelayproject',
                        credential: 'openrelayproject'
                    }
                ];

                // WebRTC configuration for optimal connectivity (W3C WebRTC spec compliant)
                const rtcConfig = {
                    iceServers,
                    iceTransportPolicy: 'all', // Use both STUN and TURN
                    iceCandidatePoolSize: 10,  // Pre-gather ICE candidates
                    bundlePolicy: 'max-bundle',
                    rtcpMuxPolicy: 'require'
                };

                // Try local PeerServer first (for same-network connectivity)
                console.log('🏠 Attempting local PeerServer connection for LAN peers...');
                this.peer = new Peer({
                    host: 'localhost',
                    port: 9000,
                    path: '/peerjs',
                    config: rtcConfig,
                    debug: 2
                });

                let resolved = false;

                // Fallback to cloud server after 3 seconds (for internet-wide connectivity)
                setTimeout(() => {
                    if (!this.peerId && !resolved) {
                        console.warn('⚠️  Local PeerServer not available');
                        console.log('🌍 Switching to cloud PeerJS server for internet-wide connectivity');
                        console.log('   ✓ STUN servers: 8 configured');
                        console.log('   ✓ TURN servers: 4 configured (NAT traversal)');
                        console.log('   ✓ ICE candidate pool: 10 (faster connections)');
                        this.peer?.destroy();
                        
                        // Use cloud PeerJS server for global connectivity
                        this.peer = new Peer({
                            config: rtcConfig,
                            debug: 2
                        });
                        
                        this.setupPeerHandlers(resolve, reject);
                    }
                }, 3000);

                this.setupPeerHandlers(resolve, reject);
            } catch (error) {
                reject(error);
            }
        });
    }

    private setupPeerHandlers(resolve: (value: string) => void, reject: (reason?: any) => void) {
        if (!this.peer) return;

        let resolved = false;

        this.peer.on('open', (id) => {
            this.peerId = id;
            console.log('✅ PeerJS connected - Peer ID:', id);
            this.emit('ready', id);
            
            if (!resolved) {
                resolved = true;
                resolve(id);
            }

            // Connect to discovery server
            this.connectToDiscoveryServer();
        });

        this.peer.on('connection', (conn) => {
            this.handleConnection(conn);
        });

        this.peer.on('error', (err: any) => {
            console.error('❌ PeerJS error:', err.type || 'unknown', '-', err.message);
            
            // Log specific error types for debugging
            if (err.type === 'network') {
                console.error('   Network error - check internet connection');
            } else if (err.type === 'peer-unavailable') {
                console.error('   Peer unavailable - they may be offline');
            } else if (err.type === 'server-error') {
                console.error('   PeerJS server error');
            }
            
            this.emit('error', err.message);
            
            if (!resolved) {
                resolved = true;
                // Don't reject immediately - allow offline mode
                console.warn('⚠️  PeerJS connection failed, continuing without WebRTC bridge');
                reject(err);
            }
        });

        // Timeout after 10 seconds
        setTimeout(() => {
            if (!resolved) {
                resolved = true;
                reject(new Error('PeerJS connection timeout'));
            }
        }, 10000);
    }

    private handleConnection(conn: DataConnection) {
        console.log('📥 Incoming connection from:', conn.peer);

        conn.on('open', () => {
            this.connections.set(conn.peer, conn);
            this.emit('peer-connected', conn.peer);
            console.log('✅ Connection established with:', conn.peer);

            // Monitor ICE connection state for diagnostics (W3C WebRTC spec)
            if ((conn as any).peerConnection) {
                const pc = (conn as any).peerConnection;
                pc.oniceconnectionstatechange = () => {
                    const state = pc.iceConnectionState;
                    console.log(`🧊 ICE state for ${conn.peer}: ${state}`);
                    
                    if (state === 'failed') {
                        console.warn('⚠️  ICE connection failed, attempting restart...');
                        pc.restartIce();
                    } else if (state === 'disconnected') {
                        console.warn('⚠️  ICE disconnected, monitoring for reconnection...');
                    }
                };
                
                pc.onicegatheringstatechange = () => {
                    console.log(`🧊 ICE gathering for ${conn.peer}: ${pc.iceGatheringState}`);
                };
            }

            // Send handshake with capabilities
            conn.send({
                type: 'handshake',
                peerId: this.peerId,
                clientType: 'desktop',
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

        conn.on('data', (data: any) => {
            this.handleMessage(conn.peer, data);
        });

        conn.on('close', () => {
            console.log('❌ Connection closed with:', conn.peer);
            this.connections.delete(conn.peer);
            this.emit('peer-disconnected', conn.peer);
        });

        conn.on('error', (err: any) => {
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

    connectToPeer(peerId: string): Promise<void> {
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

        console.log('🔗 Connecting to peer:', peerId);
        console.log('   My Peer ID:', this.peerId);
        console.log('   Using WebRTC with STUN/TURN for NAT traversal...');
        
        return new Promise((resolve, reject) => {
            try {
                const conn = this.peer!.connect(peerId, { 
                    reliable: true,
                    serialization: 'json',
                    metadata: {
                        clientType: 'desktop',
                        timestamp: Date.now(),
                        version: '1.0.0'
                    }
                });

                // Add timeout for connection attempt
                const connectionTimeout = setTimeout(() => {
                    if (!this.connections.has(peerId)) {
                        console.warn('⚠️  Connection timeout for peer:', peerId);
                        conn.close();
                        reject(new Error('Connection timeout'));
                    }
                }, 30000); // 30 second timeout

                conn.on('open', () => {
                    clearTimeout(connectionTimeout);
                    console.log('✅ Successfully connected to peer:', peerId);
                    resolve();
                });

                conn.on('error', (err: any) => {
                    clearTimeout(connectionTimeout);
                    reject(err);
                });

                this.handleConnection(conn);
            } catch (error) {
                console.error('❌ Failed to initiate connection to', peerId, ':', error);
                reject(error);
            }
        });
    }
    
    // Batch connect to multiple peers with staggered timing
    async connectToPeers(peerIds: string[]): Promise<{ successful: number; failed: number }> {
        console.log(`🔗 Batch connecting to ${peerIds.length} peers...`);
        
        // Stagger connections to avoid overwhelming the network
        const results: Promise<void>[] = [];
        for (let i = 0; i < peerIds.length; i++) {
            const peerId = peerIds[i];
            // Add small delay between connections
            if (i > 0) {
                await new Promise(resolve => setTimeout(resolve, 500));
            }
            results.push(this.connectToPeer(peerId));
        }
        
        const settled = await Promise.allSettled(results);
        const successful = settled.filter(r => r.status === 'fulfilled').length;
        const failed = settled.filter(r => r.status === 'rejected').length;
        
        console.log(`📊 Connection results: ${successful} successful, ${failed} failed`);
        return { successful, failed };
    }

    private handleMessage(peerId: string, data: any) {
        console.log('📨 Message from', peerId, ':', data.type);

        switch (data.type) {
            case 'handshake':
                console.log('🤝 Handshake from:', data.peerId, `(${data.clientType || 'unknown'})`);
                // Send peer list for peer exchange
                this.sendPeerList(peerId);
                break;

            case 'get-peer-list':
                // Peer is requesting our peer list
                this.sendPeerList(peerId);
                break;

            case 'peer-list':
                console.log('📋 Received peer list from', peerId, ':', data.peers.length, 'peers');
                // Auto-connect to new peers (peer exchange)
                data.peers.forEach((remotePeerId: string) => {
                    if (remotePeerId !== this.peerId && !this.connections.has(remotePeerId)) {
                        console.log('🔗 Discovered peer via exchange:', remotePeerId);
                        setTimeout(() => this.connectToPeer(remotePeerId), Math.random() * 2000);
                    }
                });
                break;

            case 'block-request':
                this.emit('block-request', {
                    blockId: data.blockId,
                    peerId
                });
                break;

            case 'block-response':
                this.emit('block-received', {
                    block: data.block,
                    peerId
                });
                break;

            case 'block-broadcast':
                this.emit('block-received', {
                    block: data.block,
                    peerId
                });
                break;

            default:
                console.log('Unknown message type:', data.type);
        }
    }

    // Send peer list to a peer for peer exchange
    private sendPeerList(peerId: string) {
        const conn = this.connections.get(peerId);
        if (conn) {
            const peerList = Array.from(this.connections.keys()).filter(id => id !== peerId);
            conn.send({
                type: 'peer-list',
                peers: peerList
            });
        }
    }

    sendBlock(peerId: string, block: any) {
        const conn = this.connections.get(peerId);
        if (conn) {
            conn.send({
                type: 'block-response',
                block
            });
        }
    }

    broadcastBlock(block: any) {
        for (const conn of this.connections.values()) {
            conn.send({
                type: 'block-broadcast',
                block
            });
        }
    }

    getPeers(): string[] {
        return Array.from(this.connections.keys());
    }

    getPeerId(): string | null {
        return this.peerId;
    }

    // Get shareable connection info with multiple formats
    getConnectionInfo() {
        const connectionUrl = `msscs://connect/${this.peerId}`;
        const webUrl = `https://msscs.app?peer=${this.peerId}`; // Replace with your actual web app URL
        
        return {
            peerId: this.peerId,
            connectionUrl,
            webUrl,
            shareText: `Connect to my MSSCS node:\nPeer ID: ${this.peerId}\nOr visit: ${webUrl}`,
            qrCodeUrl: this.generateQRCodeUrl(),
            qrCodeData: this.peerId
        };
    }

    generateQRCodeUrl(): string | null {
        if (!this.peerId) return null;
        // Generate QR code with connection URL for easy mobile sharing
        const connectionUrl = `msscs://connect/${this.peerId}`;
        return `https://api.qrserver.com/v1/create-qr-code/?size=300x300&data=${encodeURIComponent(connectionUrl)}`;
    }

    // Export peer ID to clipboard
    async copyPeerIdToClipboard(): Promise<boolean> {
        if (!this.peerId) return false;
        try {
            await navigator.clipboard.writeText(this.peerId);
            return true;
        } catch (error) {
            console.error('Failed to copy to clipboard:', error);
            return false;
        }
    }

    // Get connection statistics
    getConnectionStats() {
        const isConnected = this.peer !== null && !this.peer.destroyed;
        const stats = {
            peerId: this.peerId,
            connectedPeers: this.connections.size,
            peers: Array.from(this.connections.keys()),
            isConnected: isConnected,
            discoveryConnected: this.discoveryWs?.readyState === WebSocket.OPEN,
            // Status summary for display
            statusText: isConnected 
                ? (this.connections.size > 0 
                    ? `Online - ${this.connections.size} peer${this.connections.size !== 1 ? 's' : ''}`
                    : 'Online - Discovering peers...')
                : 'Offline'
        };
        return stats;
    }

    private connectToDiscoveryServer() {
        if (!this.peerId) return;

        try {
            // Try to get discovery server URL from environment or use default
            // In production, this should be configured to point to a public server
            const wsUrl = (window as any).MSSCS_DISCOVERY_SERVER || 'ws://localhost:9001';
            
            console.log('🔍 Connecting to discovery server:', wsUrl);
            this.discoveryWs = new WebSocket(wsUrl);

            this.discoveryWs.onopen = () => {
                console.log('✅ Connected to local discovery server');

                // Register with discovery server
                this.discoveryWs?.send(JSON.stringify({
                    type: 'register',
                    peerId: this.peerId,
                    peerType: 'desktop'
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
                            message.peers.forEach((peer: any) => {
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
                    }
                } catch (error) {
                    console.error('Error handling discovery message:', error);
                }
            };

            this.discoveryWs.onerror = () => {
                console.warn('⚠️  Discovery server connection error');
            };

            this.discoveryWs.onclose = () => {
                console.log('❌ Disconnected from discovery server');
                // Try to reconnect after 5 seconds
                setTimeout(() => {
                    if (this.peerId) {
                        console.log('🔄 Reconnecting to discovery server...');
                        this.connectToDiscoveryServer();
                    }
                }, 5000);
            };
        } catch (error) {
            console.warn('⚠️  Could not connect to local discovery server');
        }
    }

    on(event: string, handler: Function) {
        if (!this.eventHandlers.has(event)) {
            this.eventHandlers.set(event, []);
        }
        this.eventHandlers.get(event)!.push(handler);
    }

    private emit(event: string, data?: any) {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            handlers.forEach(handler => handler(data));
        }
    }

    destroy() {
        this.discoveryWs?.close();
        this.peer?.destroy();
        this.connections.clear();
    }
}
