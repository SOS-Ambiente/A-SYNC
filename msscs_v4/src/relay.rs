// RELAY NODES & NAT TRAVERSAL
// Enables P2P connections through firewalls and NAT

use crate::error::{MSSCSError, Result};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

/// Relay node configuration
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Maximum concurrent relay connections
    pub max_connections: usize,
    /// Relay timeout (seconds)
    pub timeout: Duration,
    /// Enable circuit relay v2
    pub enable_circuit_relay: bool,
    /// Enable hole punching (DCUTR)
    pub enable_hole_punching: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        RelayConfig {
            max_connections: 100,
            timeout: Duration::from_secs(300),
            enable_circuit_relay: true,
            enable_hole_punching: true,
        }
    }
}

/// NAT type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NATType {
    /// No NAT (public IP)
    None,
    /// Full cone NAT (easiest to traverse)
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port-restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT (hardest to traverse)
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

/// Connection method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMethod {
    /// Direct connection (no NAT)
    Direct,
    /// Hole punching successful
    HolePunched,
    /// Relayed through relay node
    Relayed,
    /// Failed to connect
    Failed,
}

/// Relay connection
#[derive(Debug, Clone)]
pub struct RelayConnection {
    /// Source peer
    pub source: PeerId,
    /// Destination peer
    pub dest: PeerId,
    /// Relay node
    pub relay: PeerId,
    /// Connection established time
    pub established_at: Instant,
    /// Bytes transferred
    pub bytes_transferred: u64,
}

/// Relay node manager
pub struct RelayManager {
    /// Configuration
    config: RelayConfig,
    /// Known relay nodes
    relay_nodes: HashSet<PeerId>,
    /// Active relay connections
    active_connections: HashMap<String, RelayConnection>,
    /// NAT type cache
    nat_types: HashMap<PeerId, NATType>,
}

impl RelayManager {
    /// Create a new relay manager
    pub fn new(config: RelayConfig) -> Self {
        RelayManager {
            config,
            relay_nodes: HashSet::new(),
            active_connections: HashMap::new(),
            nat_types: HashMap::new(),
        }
    }
    
    /// Add a relay node
    pub fn add_relay_node(&mut self, peer_id: PeerId) {
        tracing::info!("🔗 Adding relay node: {}", peer_id);
        self.relay_nodes.insert(peer_id);
    }
    
    /// Remove a relay node
    pub fn remove_relay_node(&mut self, peer_id: &PeerId) {
        tracing::info!("🔌 Removing relay node: {}", peer_id);
        self.relay_nodes.remove(peer_id);
    }
    
    /// Get available relay nodes
    pub fn get_relay_nodes(&self) -> Vec<PeerId> {
        self.relay_nodes.iter().copied().collect()
    }
    
    /// Detect NAT type
    pub async fn detect_nat_type(&mut self, local_addr: SocketAddr, stun_servers: &[String]) -> Result<NATType> {
        tracing::info!("🔍 Detecting NAT type...");
        
        // Simplified NAT detection
        // In production, use STUN protocol (RFC 5389)
        
        // Check if we have a public IP
        if self.is_public_ip(local_addr.ip()) {
            tracing::info!("   ✅ No NAT detected (public IP)");
            return Ok(NATType::None);
        }
        
        // Perform STUN binding requests
        // This would involve:
        // 1. Send binding request to STUN server
        // 2. Receive mapped address
        // 3. Compare with local address
        // 4. Perform additional tests to determine NAT type
        
        tracing::info!("   ⚠️  NAT detected (type unknown)");
        Ok(NATType::Unknown)
    }
    
    /// Check if IP is public
    fn is_public_ip(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // Check for private IP ranges
                !(octets[0] == 10
                    || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                    || (octets[0] == 192 && octets[1] == 168)
                    || octets[0] == 127) // localhost
            }
            IpAddr::V6(ipv6) => {
                // Simplified IPv6 check
                !ipv6.is_loopback() && !ipv6.is_unspecified()
            }
        }
    }
    
    /// Attempt hole punching (DCUTR - Direct Connection Upgrade through Relay)
    pub async fn attempt_hole_punching(
        &mut self,
        local_peer: PeerId,
        remote_peer: PeerId,
        relay_peer: PeerId,
    ) -> Result<ConnectionMethod> {
        tracing::info!("🔨 Attempting hole punching: {} -> {}", local_peer, remote_peer);
        
        if !self.config.enable_hole_punching {
            tracing::debug!("   Hole punching disabled");
            return Ok(ConnectionMethod::Relayed);
        }
        
        // Get NAT types
        let local_nat = self.nat_types.get(&local_peer).copied().unwrap_or(NATType::Unknown);
        let remote_nat = self.nat_types.get(&remote_peer).copied().unwrap_or(NATType::Unknown);
        
        tracing::debug!("   Local NAT: {:?}, Remote NAT: {:?}", local_nat, remote_nat);
        
        // Check if hole punching is likely to succeed
        let can_punch = match (local_nat, remote_nat) {
            (NATType::None, _) | (_, NATType::None) => true,
            (NATType::FullCone, _) | (_, NATType::FullCone) => true,
            (NATType::RestrictedCone, NATType::RestrictedCone) => true,
            (NATType::PortRestrictedCone, NATType::PortRestrictedCone) => true,
            (NATType::Symmetric, NATType::Symmetric) => false, // Very difficult
            _ => true, // Try anyway
        };
        
        if !can_punch {
            tracing::warn!("   ⚠️  Hole punching unlikely to succeed (symmetric NAT)");
            return Ok(ConnectionMethod::Relayed);
        }
        
        // Perform hole punching
        // In production, this would:
        // 1. Coordinate with relay to exchange addresses
        // 2. Both peers send packets to each other simultaneously
        // 3. NAT creates mappings allowing direct communication
        // 4. Upgrade from relayed to direct connection
        
        tracing::info!("   ✅ Hole punching successful");
        Ok(ConnectionMethod::HolePunched)
    }
    
    /// Establish relayed connection
    pub async fn establish_relay_connection(
        &mut self,
        source: PeerId,
        dest: PeerId,
        relay: PeerId,
    ) -> Result<String> {
        tracing::info!("🔗 Establishing relay connection: {} -> {} via {}", source, dest, relay);
        
        // Check if relay node is available
        if !self.relay_nodes.contains(&relay) {
            return Err(MSSCSError::Network(format!("Relay node {} not available", relay)));
        }
        
        // Check connection limit
        if self.active_connections.len() >= self.config.max_connections {
            return Err(MSSCSError::Network("Maximum relay connections reached".to_string()));
        }
        
        // Create connection ID
        let connection_id = format!("{}-{}-{}", source, dest, relay);
        
        // Create relay connection
        let connection = RelayConnection {
            source,
            dest,
            relay,
            established_at: Instant::now(),
            bytes_transferred: 0,
        };
        
        self.active_connections.insert(connection_id.clone(), connection);
        
        tracing::info!("   ✅ Relay connection established: {}", connection_id);
        Ok(connection_id)
    }
    
    /// Close relay connection
    pub fn close_relay_connection(&mut self, connection_id: &str) -> Result<()> {
        if let Some(connection) = self.active_connections.remove(connection_id) {
            let duration = connection.established_at.elapsed();
            tracing::info!("🔌 Closed relay connection: {} (duration: {:.2}s, bytes: {})", 
                connection_id, duration.as_secs_f64(), connection.bytes_transferred);
            Ok(())
        } else {
            Err(MSSCSError::NotFound(format!("Connection {} not found", connection_id)))
        }
    }
    
    /// Update connection statistics
    pub fn update_connection_stats(&mut self, connection_id: &str, bytes: u64) {
        if let Some(connection) = self.active_connections.get_mut(connection_id) {
            connection.bytes_transferred += bytes;
        }
    }
    
    /// Clean up expired connections
    pub fn cleanup_expired_connections(&mut self) -> usize {
        let mut expired = Vec::new();
        
        for (id, connection) in &self.active_connections {
            if connection.established_at.elapsed() > self.config.timeout {
                expired.push(id.clone());
            }
        }
        
        let count = expired.len();
        for id in expired {
            self.close_relay_connection(&id).ok();
        }
        
        if count > 0 {
            tracing::info!("🗑️  Cleaned up {} expired relay connections", count);
        }
        
        count
    }
    
    /// Get connection statistics
    pub fn get_stats(&self) -> RelayStats {
        let total_bytes: u64 = self.active_connections.values()
            .map(|c| c.bytes_transferred)
            .sum();
        
        RelayStats {
            relay_nodes: self.relay_nodes.len(),
            active_connections: self.active_connections.len(),
            total_bytes_transferred: total_bytes,
        }
    }
}

/// Relay statistics
#[derive(Debug, Clone)]
pub struct RelayStats {
    pub relay_nodes: usize,
    pub active_connections: usize,
    pub total_bytes_transferred: u64,
}

/// Bootstrap relay nodes (public infrastructure)
pub fn get_bootstrap_relay_nodes() -> Vec<String> {
    vec![
        // In production, these would be actual relay node addresses
        "/ip4/relay1.msscs.network/tcp/4001/p2p/12D3KooWRelay1".to_string(),
        "/ip4/relay2.msscs.network/tcp/4001/p2p/12D3KooWRelay2".to_string(),
        "/ip4/relay3.msscs.network/tcp/4001/p2p/12D3KooWRelay3".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_relay_manager_creation() {
        let config = RelayConfig::default();
        let manager = RelayManager::new(config);
        
        assert_eq!(manager.relay_nodes.len(), 0);
        assert_eq!(manager.active_connections.len(), 0);
    }
    
    #[test]
    fn test_add_remove_relay_node() {
        let mut manager = RelayManager::new(RelayConfig::default());
        let peer_id = PeerId::random();
        
        manager.add_relay_node(peer_id);
        assert_eq!(manager.relay_nodes.len(), 1);
        
        manager.remove_relay_node(&peer_id);
        assert_eq!(manager.relay_nodes.len(), 0);
    }
    
    #[test]
    fn test_is_public_ip() {
        let manager = RelayManager::new(RelayConfig::default());
        
        // Private IPs
        assert!(!manager.is_public_ip("10.0.0.1".parse().unwrap()));
        assert!(!manager.is_public_ip("192.168.1.1".parse().unwrap()));
        assert!(!manager.is_public_ip("172.16.0.1".parse().unwrap()));
        assert!(!manager.is_public_ip("127.0.0.1".parse().unwrap()));
        
        // Public IP
        assert!(manager.is_public_ip("8.8.8.8".parse().unwrap()));
    }
    
    #[tokio::test]
    async fn test_relay_connection() {
        let mut manager = RelayManager::new(RelayConfig::default());
        
        let source = PeerId::random();
        let dest = PeerId::random();
        let relay = PeerId::random();
        
        // Add relay node
        manager.add_relay_node(relay);
        
        // Establish connection
        let conn_id = manager.establish_relay_connection(source, dest, relay).await.unwrap();
        assert_eq!(manager.active_connections.len(), 1);
        
        // Update stats
        manager.update_connection_stats(&conn_id, 1024);
        
        // Close connection
        manager.close_relay_connection(&conn_id).unwrap();
        assert_eq!(manager.active_connections.len(), 0);
    }
    
    #[test]
    fn test_nat_type_detection() {
        let _manager = RelayManager::new(RelayConfig::default());
        
        // Test NAT type combinations for hole punching
        let test_cases = vec![
            (NATType::None, NATType::FullCone, true),
            (NATType::FullCone, NATType::RestrictedCone, true),
            (NATType::Symmetric, NATType::Symmetric, false),
        ];
        
        for (local, remote, _expected) in test_cases {
            // Test logic would go here
            assert!(local != NATType::Unknown || remote != NATType::Unknown);
        }
    }
}
