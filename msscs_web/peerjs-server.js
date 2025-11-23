// Simple PeerJS signaling server for local network
const { PeerServer } = require('peer');

const server = PeerServer({
    port: 9000,
    path: '/peerjs',
    allow_discovery: true
});

server.on('connection', (client) => {
    console.log('✅ Client connected:', client.getId());
});

server.on('disconnect', (client) => {
    console.log('❌ Client disconnected:', client.getId());
});

console.log('🚀 PeerJS signaling server running on port 9000');
console.log('📡 Path: /peerjs');
console.log('🌐 Clients can connect to: http://localhost:9000/peerjs');
