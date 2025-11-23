// P2P Bridge module - Connects mobile discovery to real P2P node
use crate::network_discovery::{NetworkDiscovery, DiscoveredNode, NetworkStatus};
use msscs_v4::{p2p_network::{P2PNode, P2PConfig, P2PEvent}, block::DataBlock};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// P2P Bridge that connects network discovery to the real P2P node
pub struct P2PBridge {
    p2p_node: Option<Arc<RwLock<P2PNode>>>,
    network_discovery: NetworkDiscovery,
    event_sender: Option<mpsc::UnboundedSender<P2PBridgeEvent>>,
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredNode>>>,
}

/// P2P Bridge events for mobile client
#[derive(Debug, Clone)]
pub enum P2PBridgeEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    BlockReceived(String, Vec<u8>),
    NetworkStatus(NetworkStatus),
    Error(String),
}

impl P2PBridge {
    /// Create new P2P bridge
    pub fn new() -> Self {
        Self {
            p2p_node: None,
            network_discovery: NetworkDiscovery::new(),
            event_sender: None,
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize P2P bridge with event sender
    pub fn initialize(&mut self, event_sender: mpsc::UnboundedSender<P2PBridgeEvent>) {
        self.event_sender = Some(event_sender);
    }

    /// Start P2P node with configuration
    pub async fn start_p2p_node(&mut self, listen_port: u16) -> Result<(), String> {
        info!("Starting P2P node on port {}", listen_port);

        let mut p2p_config = P2PConfig::default();
        p2p_config.listen_port = listen_port;
        p2p_config.enable_mdns = true;
        p2p_config.max_peers = 20;
        p2p_config.replication_factor = 3;

        let (mut p2p_node, mut event_receiver, local_blocks) = match P2PNode::new(p2p_config).await {
            Ok(result) => result,
            Err(e) => return Err(format!("Failed to create P2P node: {}", e)),
        };

        // Start P2P node
        let (event_rx, blocks) = match p2p_node.start().await {
            Ok(result) => result,
            Err(e) => return Err(format!("Failed to start P2P node: {}", e)),
        };

        // Store P2P node
        let p2p_node_arc = Arc::new(RwLock::new(p2p_node));
        self.p2p_node = Some(p2p_node_arc.clone());

        // Handle P2P events in background
        if let Some(ref sender) = self.event_sender {
            let event_sender = sender.clone();
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    match event {
                        P2PEvent::PeerConnected(peer_id) => {
                            info!("P2P peer connected: {}", peer_id);
                            let _ = event_sender.send(P2PBridgeEvent::PeerConnected(peer_id.to_string()));
                        }
                        P2PEvent::PeerDisconnected(peer_id) => {
                            info!("P2P peer disconnected: {}", peer_id);
                            let _ = event_sender.send(P2PBridgeEvent::PeerDisconnected(peer_id.to_string()));
                        }
                        P2PEvent::BlockReceived { peer, block } => {
                            info!("Received block {} from peer {}", block.uuid, peer);
                            let _ = event_sender.send(P2PBridgeEvent::BlockReceived(block.uuid.to_string(), block.data.clone()));
                        }
                        P2PEvent::BootstrapComplete => {
                            info!("P2P bootstrap completed");
                        }
                        P2PEvent::Error(error) => {
                            warn!("P2P error: {}", error);
                            let _ = event_sender.send(P2PBridgeEvent::Error(error));
                        }
                    }
                }
            });
        }

        // Start network discovery
        let discovery = self.network_discovery.clone();
        let p2p_node_clone = p2p_node_arc.clone();
        let discovered_peers = self.discovered_peers.clone();
        let event_sender_clone = self.event_sender.clone();

        tokio::spawn(async move {
            let mut discovery_clone = discovery;

            // Start discovery in a loop
            loop {
                if let Some(ref p2p) = p2p_node_clone.try_read() {
                    let temp_node = Arc::new(crate::network::Node::new(crate::config::Config::default()));
                    discovery_clone.start_discovery(temp_node).await;
                }

                // Check connectivity status
                if let Some(ref sender) = event_sender_clone {
                    match discovery_clone.check_network_connectivity().await {
                        Ok(status) => {
                            let _ = sender.send(P2PBridgeEvent::NetworkStatus(status));
                        }
                        Err(e) => {
                            let _ = sender.send(P2PBridgeEvent::Error(format!("Network check failed: {}", e)));
                        }
                    }
                }

                sleep(Duration::from_secs(30)).await; // Check every 30 seconds
            }
        });

        info!("P2P bridge started successfully");
        Ok(())
    }

    /// Add discovered peer to P2P network
    pub async fn add_discovered_peer(&mut self, address: String, port: u16) -> Result<(), String> {
        let peer_addr = format!("{}:{}", address, port);

        // Add to discovered peers list
        let discovered_node = DiscoveredNode {
            name: format!("Mobile: {}", address),
            address: address.clone(),
            port,
            node_id: peer_addr.clone(),
        };

        {
            let mut peers = self.discovered_peers.write().await;
            peers.insert(address, discovered_node);
        }

        // Add to P2P node if available
        if let Some(ref p2p_node_arc) = self.p2p_node {
            let mut p2p_node = p2p_node_arc.write().await;

            // Parse multiaddr
            let multiaddr = format!("/ip4/{}/tcp/{}", address, port);
            match multiaddr.parse::<libp2p::Multiaddr>() {
                Ok(addr) => {
                    if let Err(e) = p2p_node.add_peer(addr).await {
                        warn!("Failed to add discovered peer to P2P: {}", e);
                        return Err(format!("Failed to add peer to P2P: {}", e));
                    } else {
                        info!("Added discovered peer {} to P2P network", peer_addr);
                        Ok(())
                    }
                }
                Err(e) => {
                    return Err(format!("Invalid peer address: {}", e));
                }
            }
        } else {
            warn!("P2P node not available, peer added to discovery list only");
            Ok(())
        }
    }

    /// Get connected P2P peers
    pub async fn get_connected_peers(&self) -> Vec<String> {
        if let Some(ref p2p_node_arc) = self.p2p_node {
            let p2p_node = p2p_node_arc.read().await;
            p2p_node.get_connected_peers().await
                .into_iter()
                .map(|peer_id| peer_id.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get discovered peers from network discovery
    pub async fn get_discovered_peers(&self) -> Vec<DiscoveredNode> {
        let peers = self.discovered_peers.read().await;
        peers.values().cloned().collect()
    }

    /// Store block in P2P network
    pub async fn store_block(&mut self, data: Vec<u8>) -> Result<String, String> {
        if let Some(ref p2p_node_arc) = self.p2p_node {
            let mut p2p_node = p2p_node_arc.write().await;

            // Create a simple DataBlock for mobile client
            let block_id = Uuid::new_v4();

            // In a real implementation, this would use the DataBlock from msscs_v4
            // For now, just store the raw data
            info!("Storing block {} in P2P network", block_id);

            // Note: This is a simplified implementation
            // The full implementation would create a proper DataBlock and store it
            Ok(block_id.to_string())
        } else {
            Err("P2P node not available".to_string())
        }
    }

    /// Request block from P2P network
    pub async fn get_block(&mut self, block_id: String) -> Result<Option<Vec<u8>>, String> {
        if let Some(ref p2p_node_arc) = self.p2p_node {
            let mut p2p_node = p2p_node_arc.write().await;

            let uuid = match Uuid::parse_str(&block_id) {
                Ok(id) => id,
                Err(e) => return Err(format!("Invalid block ID: {}", e)),
            };

            info!("Requesting block {} from P2P network", uuid);

            // Note: This is a simplified implementation
            // The full implementation would use the P2PNode's get_block method
            Ok(None) // Placeholder
        } else {
            Err("P2P node not available".to_string())
        }
    }

    /// Check P2P network status
    pub async fn get_network_status(&self) -> NetworkStatus {
        let mut status = self.network_discovery.check_network_connectivity().await
            .unwrap_or_else(|e| {
                NetworkStatus {
                    mdns_available: false,
                    internet_available: false,
                    firewall_blocked: true,
                    mdns_error: Some(e.to_string()),
                    firewall_help: None,
                }
            });

        // Check P2P node status
        if let Some(ref p2p_node_arc) = self.p2p_node {
            let connected_peers = {
                let p2p_node = p2p_node_arc.read().await;
                p2p_node.get_connected_peers().await.len()
            };

            // Update status based on P2P connectivity
            if connected_peers > 0 {
                status.internet_available = true;
                status.firewall_blocked = false;
            }
        }

        status
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let discovered_peers = self.get_discovered_peers().await.len();
        let connected_peers = self.get_connected_peers().await.len();

        NetworkStats {
            discovered_peers,
            connected_peers,
            total_blocks: 0, // Would be implemented with proper block tracking
            network_health: if connected_peers > 0 { NetworkHealth::Healthy } else { NetworkHealth::Offline },
        }
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub discovered_peers: usize,
    pub connected_peers: usize,
    pub total_blocks: usize,
    pub network_health: NetworkHealth,
}

#[derive(Debug, Clone)]
pub enum NetworkHealth {
    Healthy,
    Warning,
    Offline,
}