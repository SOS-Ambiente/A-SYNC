// Connection Quality Monitor for WebRTC P2P connections
// Tracks connection health, latency, and bandwidth

import type { PeerJSBridge } from './peerjs-bridge';

interface ConnectionMetrics {
    rtt: number;
    packetLoss: number;
    bytesReceived: number;
    bytesSent: number;
    connectionType: 'direct' | 'relayed' | 'unknown';
    candidateType: string;
    timestamp: number;
    peerId: string;
}

export class ConnectionMonitor {
    private p2p: PeerJSBridge;
    private metrics: Map<string, ConnectionMetrics> = new Map();
    private monitoringInterval: ReturnType<typeof setInterval> | null = null;

    constructor(p2pNetwork: PeerJSBridge) {
        this.p2p = p2pNetwork;
    }

    startMonitoring(intervalMs: number = 5000): void {
        console.log('📊 Starting connection quality monitoring...');
        
        this.monitoringInterval = setInterval(() => {
            this.updateMetrics();
        }, intervalMs);
    }

    stopMonitoring(): void {
        if (this.monitoringInterval) {
            clearInterval(this.monitoringInterval);
            this.monitoringInterval = null;
        }
    }

    private async updateMetrics(): Promise<void> {
        const peers = this.p2p.getPeers();
        
        for (const peerId of peers) {
            try {
                // Access internal connection object
                const conn = (this.p2p as any).connections?.get(peerId);
                if (!conn?.peerConnection) continue;

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

    private parseStats(stats: RTCStatsReport): Omit<ConnectionMetrics, 'timestamp' | 'peerId'> {
        const metrics: Omit<ConnectionMetrics, 'timestamp' | 'peerId'> = {
            rtt: 0,
            packetLoss: 0,
            bytesReceived: 0,
            bytesSent: 0,
            connectionType: 'unknown',
            candidateType: 'unknown'
        };

        stats.forEach((report: any) => {
            // RTT (Round Trip Time)
            if (report.type === 'candidate-pair' && report.state === 'succeeded') {
                metrics.rtt = (report.currentRoundTripTime || 0) * 1000;
                
                // Determine connection type
                if (report.remoteCandidateId) {
                    stats.forEach((candidate: any) => {
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

    getMetrics(peerId: string): ConnectionMetrics | undefined {
        return this.metrics.get(peerId);
    }

    getAllMetrics(): ConnectionMetrics[] {
        return Array.from(this.metrics.values());
    }

    getConnectionQuality(peerId: string): 'excellent' | 'good' | 'fair' | 'poor' | 'unknown' {
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
        summary.avgPacketLoss = parseFloat((totalPacketLoss / this.metrics.size).toFixed(2));

        return summary;
    }
}
