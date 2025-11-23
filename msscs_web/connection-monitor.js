// Connection Quality Monitor for WebRTC P2P connections
// Tracks connection health, latency, and bandwidth

export class ConnectionMonitor {
    constructor(p2pNetwork) {
        this.p2p = p2pNetwork;
        this.metrics = new Map();
        this.monitoringInterval = null;
    }

    startMonitoring(intervalMs = 5000) {
        console.log('📊 Starting connection quality monitoring...');
        
        this.monitoringInterval = setInterval(() => {
            this.updateMetrics();
        }, intervalMs);
    }

    stopMonitoring() {
        if (this.monitoringInterval) {
            clearInterval(this.monitoringInterval);
            this.monitoringInterval = null;
        }
    }

    async updateMetrics() {
        for (const [peerId, conn] of this.p2p.connections.entries()) {
            if (!conn.peerConnection) continue;

            try {
                const stats = await conn.peerConnection.getStats();
                const metrics = this.parseStats(stats);
                
                this.metrics.set(peerId, {
                    ...metrics,
                    timestamp: Date.now(),
                    peerId
                });

                // Log significant changes
                if (metrics.rtt > 500) {
                    console.warn(`⚠️  High latency to ${peerId}: ${metrics.rtt}ms`);
                }
                if (metrics.packetLoss > 5) {
                    console.warn(`⚠️  Packet loss to ${peerId}: ${metrics.packetLoss}%`);
                }
            } catch (error) {
                console.error(`Failed to get stats for ${peerId}:`, error);
            }
        }
    }

    parseStats(stats) {
        const metrics = {
            rtt: 0,
            packetLoss: 0,
            bytesReceived: 0,
            bytesSent: 0,
            connectionType: 'unknown',
            candidateType: 'unknown'
        };

        stats.forEach(report => {
            // RTT (Round Trip Time)
            if (report.type === 'candidate-pair' && report.state === 'succeeded') {
                metrics.rtt = report.currentRoundTripTime * 1000 || 0;
                
                // Determine connection type
                if (report.remoteCandidateId) {
                    stats.forEach(candidate => {
                        if (candidate.id === report.remoteCandidateId) {
                            metrics.candidateType = candidate.candidateType || 'unknown';
                            metrics.connectionType = candidate.candidateType === 'relay' ? 'relayed' : 'direct';
                        }
                    });
                }
            }

            // Packet loss
            if (report.type === 'inbound-rtp') {
                const packetsLost = report.packetsLost || 0;
                const packetsReceived = report.packetsReceived || 0;
                if (packetsReceived > 0) {
                    metrics.packetLoss = (packetsLost / (packetsLost + packetsReceived)) * 100;
                }
            }

            // Bandwidth
            if (report.type === 'transport') {
                metrics.bytesReceived = report.bytesReceived || 0;
                metrics.bytesSent = report.bytesSent || 0;
            }
        });

        return metrics;
    }

    getMetrics(peerId) {
        return this.metrics.get(peerId);
    }

    getAllMetrics() {
        return Array.from(this.metrics.values());
    }

    getConnectionQuality(peerId) {
        const metrics = this.metrics.get(peerId);
        if (!metrics) return 'unknown';

        // Simple quality scoring
        if (metrics.rtt < 100 && metrics.packetLoss < 1) return 'excellent';
        if (metrics.rtt < 250 && metrics.packetLoss < 3) return 'good';
        if (metrics.rtt < 500 && metrics.packetLoss < 5) return 'fair';
        return 'poor';
    }

    getSummary() {
        const summary = {
            totalConnections: this.metrics.size,
            avgRtt: 0,
            avgPacketLoss: 0,
            directConnections: 0,
            relayedConnections: 0,
            qualityDistribution: {
                excellent: 0,
                good: 0,
                fair: 0,
                poor: 0,
                unknown: 0
            }
        };

        if (this.metrics.size === 0) return summary;

        let totalRtt = 0;
        let totalPacketLoss = 0;

        for (const [peerId, metrics] of this.metrics.entries()) {
            totalRtt += metrics.rtt;
            totalPacketLoss += metrics.packetLoss;
            
            if (metrics.connectionType === 'direct') summary.directConnections++;
            if (metrics.connectionType === 'relayed') summary.relayedConnections++;
            
            const quality = this.getConnectionQuality(peerId);
            summary.qualityDistribution[quality]++;
        }

        summary.avgRtt = Math.round(totalRtt / this.metrics.size);
        summary.avgPacketLoss = (totalPacketLoss / this.metrics.size).toFixed(2);

        return summary;
    }
}
