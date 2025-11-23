export class P2PNetwork {
    constructor() {
        this.peer = null;
        this.connections = new Map();
        this.eventHandlers = new Map();
        this.peerId = null;
    }

    async init() {
        // Use PeerJS for WebRTC signaling
        return new Promise((resolve) => {
            // Import PeerJS from CDN
            if (!window.Peer) {
                const script = document.createElement('script');
                script.src = 'https://unpkg.com/peerjs@1.5.2/dist/peerjs.min.js';
                script.onload = () => this.initPeer(resolve);
                document.head.appendChild(script);
            } else {
                this.initPeer(resolve);
            }
        });
    }

    initPeer(resolve) {
        // Create peer with random ID
        this.peer = new Peer({
            config: {
                iceServers: [
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:stun1.l.google.com:19302' }
                ]
            }
        });

        this.peer.on('open', (id) => {
            this.peerId = id;
            console.log('My peer ID:', id);
            this.emit('ready', id);
            resolve();
        });

        this.peer.on('connection', (conn) => {
            this.handleConnection(conn);
        });

        this.peer.on('error', (err) => {
            console.error('Peer error:', err);
        });
    }

    handleConnection(conn) {
        console.log('Incoming connection from:', conn.peer);

        conn.on('open', () => {
            this.connections.set(conn.peer, conn);
            this.emit('peer-connected', conn.peer);
            
            // Send handshake
            conn.send({
                type: 'handshake',
                peerId: this.peerId
            });
        });

        conn.on('data', (data) => {
            this.handleMessage(conn.peer, data);
        });

        conn.on('close', () => {
            this.connections.delete(conn.peer);
            this.emit('peer-disconnected', conn.peer);
        });

        conn.on('error', (err) => {
            console.error('Connection error:', err);
        });
    }

    connectToPeer(peerId) {
        if (this.connections.has(peerId)) {
            console.log('Already connected to', peerId);
            return;
        }

        const conn = this.peer.connect(peerId, {
            reliable: true
        });

        this.handleConnection(conn);
    }

    handleMessage(peerId, data) {
        switch (data.type) {
            case 'handshake':
                console.log('Handshake from:', data.peerId);
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
