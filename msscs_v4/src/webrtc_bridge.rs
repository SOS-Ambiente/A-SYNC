// WebRTC Bridge for Web-to-Desktop P2P connectivity
// Enables seamless communication between browser clients and desktop/mobile apps

use crate::error::Result;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// WebRTC signaling message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    /// Offer from initiator
    Offer {
        sdp: String,
        peer_id: String,
    },
    /// Answer from responder
    Answer {
        sdp: String,
        peer_id: String,
    },
    /// ICE candidate
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
        peer_id: String,
    },
}

/// WebRTC bridge for cross-platform P2P
pub struct WebRTCBridge {
    /// Peer ID mapping (WebRTC peer ID -> libp2p PeerId)
    peer_mapping: Arc<RwLock<HashMap<String, PeerId>>>,
    
    /// Pending signaling messages
    pending_signals: Arc<RwLock<HashMap<String, Vec<SignalingMessage>>>>,
}

impl WebRTCBridge {
    /// Create new WebRTC bridge
    pub fn new() -> Self {
        info!("🌉 Initializing WebRTC bridge for web-to-desktop connectivity");
        Self {
            peer_mapping: Arc::new(RwLock::new(HashMap::new())),
            pending_signals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a peer mapping
    pub async fn register_peer(&self, webrtc_id: String, libp2p_id: PeerId) {
        let mut mapping = self.peer_mapping.write().await;
        mapping.insert(webrtc_id.clone(), libp2p_id);
        info!("✅ Registered peer mapping: {} -> {}", webrtc_id, libp2p_id);
    }
    
    /// Get libp2p peer ID from WebRTC ID
    pub async fn get_libp2p_peer(&self, webrtc_id: &str) -> Option<PeerId> {
        let mapping = self.peer_mapping.read().await;
        mapping.get(webrtc_id).copied()
    }
    
    /// Store signaling message for peer
    pub async fn store_signal(&self, target_peer: String, message: SignalingMessage) {
        let mut signals = self.pending_signals.write().await;
        signals.entry(target_peer.clone())
            .or_insert_with(Vec::new)
            .push(message);
        debug!("📨 Stored signaling message for peer: {}", target_peer);
    }
    
    /// Retrieve pending signals for peer
    pub async fn get_signals(&self, peer_id: &str) -> Vec<SignalingMessage> {
        let mut signals = self.pending_signals.write().await;
        signals.remove(peer_id).unwrap_or_default()
    }
    
    /// Handle incoming signaling message
    pub async fn handle_signal(&self, message: SignalingMessage) -> Result<()> {
        match &message {
            SignalingMessage::Offer { peer_id, .. } => {
                info!("📞 Received WebRTC offer from: {}", peer_id);
            }
            SignalingMessage::Answer { peer_id, .. } => {
                info!("📞 Received WebRTC answer from: {}", peer_id);
            }
            SignalingMessage::IceCandidate { peer_id, .. } => {
                debug!("🧊 Received ICE candidate from: {}", peer_id);
            }
        }
        
        // Store for target peer to retrieve
        let target = match &message {
            SignalingMessage::Offer { peer_id, .. } |
            SignalingMessage::Answer { peer_id, .. } |
            SignalingMessage::IceCandidate { peer_id, .. } => peer_id.clone(),
        };
        
        self.store_signal(target, message).await;
        Ok(())
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> BridgeStats {
        let mapping = self.peer_mapping.read().await;
        let signals = self.pending_signals.read().await;
        
        BridgeStats {
            registered_peers: mapping.len(),
            pending_signals: signals.values().map(|v| v.len()).sum(),
        }
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub registered_peers: usize,
    pub pending_signals: usize,
}

impl Default for WebRTCBridge {
    fn default() -> Self {
        Self::new()
    }
}
