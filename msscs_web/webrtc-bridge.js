// WebRTC Bridge for connecting PeerJS (web) with libp2p (Rust/Tauri)
// This bridge enables cross-platform P2P communication

export class WebRTCBridge {
    constructor(p2pNetwork) {
        this.p2p = p2pNetwork;
        this.bridgeConnections = new Map();
        this.signalServer = null;
    }

    // Initialize bridge with signaling server
    async init(signalServerUrl = 'ws://localhost:9002') {
        try {
            this.signalServer = new WebSocket(signalServerUrl);
            
            this.signalServer.onopen = () => {
                console.log('✅ WebRTC Bridge connected to signaling server');
                this.registerWithBridge();
            };
            
            this.signalServer.onmessage = (event) => {
                this.handleSignal(JSON.parse(event.data));
            };
            
            this.signalServer.onerror = () => {
                console.warn('⚠️  Bridge signaling server not available');
                console.log('   Web peers can still connect via PeerJS');
            };
            
            this.signalServer.onclose = () => {
                console.log('❌ Bridge signaling server disconnected');
                // Retry connection after 5 seconds
                setTimeout(() => this.init(signalServerUrl), 5000);
            };
        } catch (error) {
            console.warn('⚠️  Could not connect to bridge server:', error.message);
        }
    }

    registerWithBridge() {
        if (this.signalServer && this.signalServer.readyState === WebSocket.OPEN) {
            this.signalServer.send(JSON.stringify({
                type: 'register',
                peerId: this.p2p.peerId,
                clientType: 'web-peerjs',
                capabilities: {
                    webrtc: true,
                    storage: true
                }
            }));
        }
    }

    handleSignal(message) {
        switch (message.type) {
            case 'bridge-peer-list':
                console.log('📋 Bridge peer list:', message.peers);
                // These are libp2p peers that can be bridged
                message.peers.forEach(peer => {
                    if (peer.clientType === 'rust-libp2p' || peer.clientType === 'desktop') {
                        console.log(`🌉 Bridgeable peer found: ${peer.peerId} (${peer.clientType})`);
                    }
                });
                break;

            case 'webrtc-offer':
                this.handleOffer(message);
                break;

            case 'webrtc-answer':
                this.handleAnswer(message);
                break;

            case 'ice-candidate':
                this.handleIceCandidate(message);
                break;
        }
    }

    async handleOffer(message) {
        // Create peer connection for bridged connection
        const pc = new RTCPeerConnection({
            iceServers: [
                { urls: 'stun:stun.l.google.com:19302' },
                {
                    urls: 'turn:openrelay.metered.ca:80',
                    username: 'openrelayproject',
                    credential: 'openrelayproject'
                }
            ]
        });

        // Set up data channel
        pc.ondatachannel = (event) => {
            const channel = event.channel;
            this.bridgeConnections.set(message.fromPeerId, { pc, channel });
            
            channel.onmessage = (e) => {
                // Forward messages to P2P network
                this.p2p.emit('bridge-message', {
                    peerId: message.fromPeerId,
                    data: JSON.parse(e.data)
                });
            };
        };

        // Handle ICE candidates
        pc.onicecandidate = (event) => {
            if (event.candidate) {
                this.sendSignal({
                    type: 'ice-candidate',
                    toPeerId: message.fromPeerId,
                    candidate: event.candidate
                });
            }
        };

        // Set remote description and create answer
        await pc.setRemoteDescription(new RTCSessionDescription(message.offer));
        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);

        this.sendSignal({
            type: 'webrtc-answer',
            toPeerId: message.fromPeerId,
            answer: pc.localDescription
        });
    }

    async handleAnswer(message) {
        const connection = this.bridgeConnections.get(message.fromPeerId);
        if (connection) {
            await connection.pc.setRemoteDescription(new RTCSessionDescription(message.answer));
        }
    }

    async handleIceCandidate(message) {
        const connection = this.bridgeConnections.get(message.fromPeerId);
        if (connection && message.candidate) {
            await connection.pc.addIceCandidate(new RTCIceCandidate(message.candidate));
        }
    }

    sendSignal(message) {
        if (this.signalServer && this.signalServer.readyState === WebSocket.OPEN) {
            this.signalServer.send(JSON.stringify(message));
        }
    }

    // Send data to bridged peer
    sendToBridgedPeer(peerId, data) {
        const connection = this.bridgeConnections.get(peerId);
        if (connection && connection.channel.readyState === 'open') {
            connection.channel.send(JSON.stringify(data));
        }
    }

    destroy() {
        this.bridgeConnections.forEach(conn => {
            conn.channel?.close();
            conn.pc?.close();
        });
        this.bridgeConnections.clear();
        this.signalServer?.close();
    }
}
