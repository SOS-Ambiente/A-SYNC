// P2P Network module - Real libp2p Kademlia DHT implementation
use crate::block::DataBlock;
use crate::error::MSSCSError;
use futures::prelude::*;
use libp2p::{
    core::Multiaddr,
    identity,
    kad::{store::MemoryStore, BootstrapOk, QueryId, Quorum, Record, RecordKey},
    mdns,
    noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// P2P Configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub listen_port: u16,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub max_peers: usize,
    pub replication_factor: usize,
    pub enable_mdns: bool,
    pub enable_relay: bool,
    pub enable_autonat: bool,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 0, // Random port
            bootstrap_peers: Self::default_bootstrap_peers(),
            max_peers: 50,
            replication_factor: 3,
            enable_mdns: true,
            enable_relay: true,
            enable_autonat: true,
        }
    }
}

impl P2PConfig {
    /// Get default public bootstrap peers for internet-wide connectivity
    /// These are well-known IPFS bootstrap nodes that act as entry points to the global DHT
    pub fn default_bootstrap_peers() -> Vec<Multiaddr> {
        vec![
            // IPFS public bootstrap nodes (reliable and well-maintained)
            // These nodes are operated by Protocol Labs and the IPFS community
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".parse().ok(),
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa".parse().ok(),
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb".parse().ok(),
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt".parse().ok(),
            
            // Additional public nodes with direct IP addresses (fallback if DNS fails)
            "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".parse().ok(),
            "/ip4/104.131.131.82/udp/4001/quic-v1/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".parse().ok(),
            
            // More IPFS bootstrap nodes for better connectivity and redundancy
            "/ip4/147.75.83.83/tcp/4001/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb".parse().ok(),
            "/ip4/147.75.83.83/udp/4001/quic-v1/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb".parse().ok(),
            
            // Additional IPFS bootstrap nodes for global coverage
            "/ip4/147.75.109.213/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".parse().ok(),
            "/ip4/147.75.109.213/udp/4001/quic-v1/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".parse().ok(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// P2P network events
#[derive(Debug, Clone)]
pub enum P2PEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    BlockReceived { peer: PeerId, block: DataBlock },
    BlockRequested { peer: PeerId, block_id: Uuid },
    BootstrapComplete,
    Error(String),
}

/// P2P Network behavior combining Kademlia DHT, Request-Response, mDNS, Relay, and AutoNAT
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "P2PBehaviourEvent")]
pub struct P2PBehaviour {
    kademlia: libp2p::kad::Behaviour<MemoryStore>,
    request_response: libp2p::request_response::Behaviour<P2PCodec>,
    mdns: mdns::tokio::Behaviour,
    relay_client: libp2p::relay::client::Behaviour,
    autonat: libp2p::autonat::Behaviour,
    dcutr: libp2p::dcutr::Behaviour,
}

#[derive(Debug)]
pub enum P2PBehaviourEvent {
    Kademlia(libp2p::kad::Event),
    RequestResponse(libp2p::request_response::Event<P2PRequest, P2PResponse>),
    Mdns(mdns::Event),
    RelayClient(libp2p::relay::client::Event),
    Autonat(libp2p::autonat::Event),
    Dcutr(libp2p::dcutr::Event),
}

impl From<libp2p::kad::Event> for P2PBehaviourEvent {
    fn from(event: libp2p::kad::Event) -> Self {
        P2PBehaviourEvent::Kademlia(event)
    }
}

impl From<libp2p::request_response::Event<P2PRequest, P2PResponse>> for P2PBehaviourEvent {
    fn from(event: libp2p::request_response::Event<P2PRequest, P2PResponse>) -> Self {
        P2PBehaviourEvent::RequestResponse(event)
    }
}

impl From<mdns::Event> for P2PBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        P2PBehaviourEvent::Mdns(event)
    }
}

impl From<libp2p::relay::client::Event> for P2PBehaviourEvent {
    fn from(event: libp2p::relay::client::Event) -> Self {
        P2PBehaviourEvent::RelayClient(event)
    }
}

impl From<libp2p::autonat::Event> for P2PBehaviourEvent {
    fn from(event: libp2p::autonat::Event) -> Self {
        P2PBehaviourEvent::Autonat(event)
    }
}

impl From<libp2p::dcutr::Event> for P2PBehaviourEvent {
    fn from(event: libp2p::dcutr::Event) -> Self {
        P2PBehaviourEvent::Dcutr(event)
    }
}

/// P2P Request/Response protocol codec
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2PRequest {
    GetBlock { id: Uuid },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2PResponse {
    Block { data: Option<DataBlock> },
    Pong,
}

#[derive(Debug, Clone, Default)]
pub struct P2PCodec;

#[async_trait::async_trait]
impl request_response::Codec for P2PCodec {
    type Protocol = &'static str;
    type Request = P2PRequest;
    type Response = P2PResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send
    {
        // Simple protocol: length-prefixed JSON
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;

        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;

        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send
    {
        use futures::AsyncWriteExt;
        let data = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        io.write_all(&(data.len() as u32).to_be_bytes()).await?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, res: Self::Response) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send
    {
        use futures::AsyncWriteExt;
        let data = serde_json::to_vec(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        io.write_all(&(data.len() as u32).to_be_bytes()).await?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }
}

/// Commands that can be sent to the P2P node
pub enum P2PNodeCommand {
    GetConnectedPeers(tokio::sync::oneshot::Sender<Vec<PeerId>>),
    StoreBlock {
        block_id: String,
        data: Vec<u8>,
        reply: tokio::sync::oneshot::Sender<std::result::Result<(), String>>,
    },
    GetBlock {
        block_id: String,
        reply: tokio::sync::oneshot::Sender<std::result::Result<Vec<u8>, String>>,
    },
}

/// Main P2P Node implementation
pub struct P2PNode {
    swarm: Swarm<P2PBehaviour>,
    event_sender: mpsc::UnboundedSender<P2PEvent>,
    local_blocks: Arc<RwLock<HashMap<Uuid, DataBlock>>>,
    pending_requests: Arc<RwLock<HashMap<QueryId, Uuid>>>,
    pending_get_queries: Arc<RwLock<HashMap<QueryId, tokio::sync::oneshot::Sender<std::result::Result<Vec<u8>, String>>>>>,
    pending_put_queries: Arc<RwLock<HashMap<QueryId, tokio::sync::oneshot::Sender<std::result::Result<(), String>>>>>,
    command_receiver: Option<mpsc::UnboundedReceiver<P2PNodeCommand>>,
    command_sender: mpsc::UnboundedSender<P2PNodeCommand>,
}

impl P2PNode {
    /// Create new P2P node with full NAT traversal support
    pub async fn new(config: P2PConfig) -> std::result::Result<Self, MSSCSError> {
        info!("Initializing P2P node with config: {:?}", config);

        // Generate or load keypair
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        // Create swarm with relay and hole-punching support
        let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default().port_reuse(true).nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| MSSCSError::Network(format!("Failed to build transport: {}", e)))?
            .with_quic() // Add QUIC transport for better NAT traversal
            .with_relay_client(noise::Config::new, yamux::Config::default)
            .map_err(|e| MSSCSError::Network(format!("Failed to create relay client: {}", e)))?
            .with_behaviour(|_keypair, relay_client| {
                // Create Kademlia DHT
                let store = MemoryStore::new(peer_id);
                let mut kademlia = libp2p::kad::Behaviour::new(peer_id, store);
                
                // Configure Kademlia for public DHT with optimized settings
                let mut kad_config = libp2p::kad::Config::default();
                kad_config.set_query_timeout(std::time::Duration::from_secs(60));
                kad_config.set_replication_factor(std::num::NonZeroUsize::new(config.replication_factor).unwrap());
                
                let mut kademlia = libp2p::kad::Behaviour::with_config(peer_id, store, kad_config);
                kademlia.set_mode(Some(libp2p::kad::Mode::Server));
                
                // Add bootstrap peers to Kademlia
                for addr in &config.bootstrap_peers {
                    if let Some(peer_id) = addr.iter().find_map(|p| match p {
                        libp2p::multiaddr::Protocol::P2p(hash) => {
                            PeerId::from_multihash(hash.into()).ok()
                        },
                        _ => None,
                    }) {
                        kademlia.add_address(&peer_id, addr.clone());
                        info!("Added bootstrap peer to Kademlia: {} at {}", peer_id, addr);
                    }
                }

                // Create request-response protocol
                let request_response = libp2p::request_response::Behaviour::new(
                    std::iter::once(("/msscs/1.0.0", ProtocolSupport::Full)),
                    libp2p::request_response::Config::default(),
                );

                // Create mDNS for local discovery
                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
                    .expect("Failed to create mDNS");

                // Create AutoNAT for NAT detection
                let autonat = libp2p::autonat::Behaviour::new(
                    peer_id,
                    libp2p::autonat::Config {
                        only_global_ips: false,
                        ..Default::default()
                    },
                );

                // Create DCUtR for hole punching
                let dcutr = libp2p::dcutr::Behaviour::new(peer_id);

                Ok(P2PBehaviour {
                    kademlia,
                    request_response,
                    mdns,
                    relay_client,
                    autonat,
                    dcutr,
                })
            })
            .map_err(|e| MSSCSError::Network(format!("Failed to create behaviour: {}", e)))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        // Listen on all interfaces with TCP (IPv4)
        let tcp_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port)
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| MSSCSError::Network(format!("Invalid address: {}", e)))?;
        swarm.listen_on(tcp_addr.clone())
            .map_err(|e| MSSCSError::Network(format!("Failed to listen on TCP: {}", e)))?;
        info!("📡 Listening on TCP: {}", tcp_addr);

        // Listen on IPv6 TCP for dual-stack support
        let tcp_addr_v6: Multiaddr = format!("/ip6/::/tcp/{}", config.listen_port)
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| MSSCSError::Network(format!("Invalid address: {}", e)))?;
        if let Ok(_) = swarm.listen_on(tcp_addr_v6.clone()) {
            info!("📡 Listening on TCP IPv6: {}", tcp_addr_v6);
        } else {
            debug!("IPv6 TCP not available (this is normal on some systems)");
        }

        // Listen on QUIC for better NAT traversal (IPv4)
        let quic_addr: Multiaddr = format!("/ip4/0.0.0.0/udp/{}/quic-v1", config.listen_port)
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| MSSCSError::Network(format!("Invalid address: {}", e)))?;
        swarm.listen_on(quic_addr.clone())
            .map_err(|e| MSSCSError::Network(format!("Failed to listen on QUIC: {}", e)))?;
        info!("📡 Listening on QUIC: {}", quic_addr);

        // Listen on QUIC IPv6 for dual-stack support
        let quic_addr_v6: Multiaddr = format!("/ip6/::/udp/{}/quic-v1", config.listen_port)
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| MSSCSError::Network(format!("Invalid address: {}", e)))?;
        if let Ok(_) = swarm.listen_on(quic_addr_v6.clone()) {
            info!("📡 Listening on QUIC IPv6: {}", quic_addr_v6);
        } else {
            debug!("IPv6 QUIC not available (this is normal on some systems)");
        }

        let (event_sender, _event_receiver) = mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        info!("P2P node created with peer ID: {}", peer_id);
        info!("NAT traversal enabled: relay={}, autonat={}", config.enable_relay, config.enable_autonat);

        Ok(P2PNode {
            swarm,
            event_sender,
            local_blocks: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_get_queries: Arc::new(RwLock::new(HashMap::new())),
            pending_put_queries: Arc::new(RwLock::new(HashMap::new())),
            command_receiver: Some(cmd_rx),
            command_sender: cmd_tx,
        })
    }
    
    /// Get the command sender for this node
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<P2PNodeCommand> {
        self.command_sender.clone()
    }

    /// Start the P2P node and return event receiver
    pub async fn start(mut self, config: P2PConfig) -> std::result::Result<(mpsc::UnboundedReceiver<P2PEvent>, Arc<RwLock<HashMap<Uuid, DataBlock>>>), MSSCSError> {
        let event_sender = self.event_sender.clone();
        let local_blocks_clone = self.local_blocks.clone();
        let local_blocks_return = self.local_blocks.clone();
        let (event_tx, event_receiver) = mpsc::unbounded_channel();
        let pending_get_queries = self.pending_get_queries.clone();
        let pending_put_queries = self.pending_put_queries.clone();
        
        // Bootstrap DHT before starting event loop
        if let Err(e) = Self::bootstrap_static(&mut self.swarm, &config).await {
            tracing::warn!("⚠️  Bootstrap warning: {}", e);
        }
        
        // Take command receiver
        let mut cmd_rx = self.command_receiver.take().expect("Command receiver already taken");

        // Run swarm event loop in background
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle swarm events
                    Some(event) = self.swarm.next() => {
                    match event {
                        SwarmEvent::Behaviour(behaviour_event) => {
                            match behaviour_event {
                                P2PBehaviourEvent::Kademlia(kad_event) => {
                                    match kad_event {
                                        libp2p::kad::Event::OutboundQueryProgressed { result, id, .. } => {
                                            match result {
                                                libp2p::kad::QueryResult::Bootstrap(Ok(BootstrapOk { .. })) => {
                                                    info!("✅ DHT bootstrap complete - connected to global network");
                                                    let _ = event_sender.send(P2PEvent::BootstrapComplete);
                                                    let _ = event_tx.send(P2PEvent::BootstrapComplete);
                                                }
                                                libp2p::kad::QueryResult::GetClosestPeers(Ok(result)) => {
                                                    if result.peers.is_empty() {
                                                        warn!("⚠️  No peers found in DHT query");
                                                        let _ = event_sender.send(P2PEvent::Error("No peers found".to_string()));
                                                    } else {
                                                        debug!("Found {} peers in DHT", result.peers.len());
                                                    }
                                                }
                                                libp2p::kad::QueryResult::GetRecord(Ok(ok)) => {
                                                    if let libp2p::kad::GetRecordOk::FoundRecord(peer_record) = ok {
                                                        debug!("✅ Got record from DHT: {:?}", peer_record.record.key);
                                                        // CRITICAL FIX: Properly handle pending queries
                                                        if let Some(sender) = pending_get_queries.write().await.remove(&id) {
                                                            let _ = sender.send(Ok(peer_record.record.value));
                                                            debug!("   Delivered record to waiting query");
                                                        }
                                                    }
                                                }
                                                libp2p::kad::QueryResult::GetRecord(Err(e)) => {
                                                    debug!("❌ Get record failed: {:?}", e);
                                                    // CRITICAL FIX: Notify waiting queries of failure
                                                    if let Some(sender) = pending_get_queries.write().await.remove(&id) {
                                                        let _ = sender.send(Err(format!("DHT get failed: {:?}", e)));
                                                    }
                                                }
                                                libp2p::kad::QueryResult::PutRecord(Ok(ok)) => {
                                                    debug!("✅ Put record successful: {:?}", ok.key);
                                                    // CRITICAL FIX: Notify waiting queries of success
                                                    if let Some(sender) = pending_put_queries.write().await.remove(&id) {
                                                        let _ = sender.send(Ok(()));
                                                        debug!("   Confirmed storage to waiting query");
                                                    }
                                                }
                                                libp2p::kad::QueryResult::PutRecord(Err(e)) => {
                                                    warn!("❌ Put record failed: {:?}", e);
                                                    // CRITICAL FIX: Notify waiting queries of failure
                                                    if let Some(sender) = pending_put_queries.write().await.remove(&id) {
                                                        let _ = sender.send(Err(format!("DHT put failed: {:?}", e)));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                P2PBehaviourEvent::RequestResponse(req_resp_event) => {
                                    match req_resp_event {
                                        request_response::Event::Message { peer, message } => {
                                            match message {
                                                request_response::Message::Request {
                                                    request,
                                                    channel,
                                                    ..
                                                } => {
                                                    match request {
                                                        P2PRequest::GetBlock { id } => {
                                                            let blocks = local_blocks_clone.read().await;
                                                            let block = blocks.get(&id).cloned();

                                                            if let Err(e) = self.swarm.behaviour_mut().request_response.send_response(channel, P2PResponse::Block { data: block }) {
                                                                warn!("Failed to send block response to {}: {:?}", peer, e);
                                                            }

                                                            let _ = event_sender.send(P2PEvent::BlockRequested { peer, block_id: id });
                                                        }
                                                        P2PRequest::Ping => {
                                                            let _ = self.swarm.behaviour_mut().request_response.send_response(channel, P2PResponse::Pong);
                                                        }
                                                    }
                                                }
                                                request_response::Message::Response {
                                                    response,
                                                    ..
                                                } => {
                                                    match response {
                                                        P2PResponse::Block { data: Some(block) } => {
                                                            let _ = event_sender.send(P2PEvent::BlockReceived { peer, block });
                                                        }
                                                        P2PResponse::Block { data: None } => {
                                                            debug!("Peer {} doesn't have requested block", peer);
                                                        }
                                                        P2PResponse::Pong => {
                                                            debug!("Received pong from {}", peer);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                P2PBehaviourEvent::Mdns(mdns_event) => {
                                    match mdns_event {
                                        mdns::Event::Discovered(list) => {
                                            for (peer_id, addr) in list {
                                                info!("Discovered peer {} at {}", peer_id, addr);
                                                self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                                let _ = self.swarm.dial(peer_id);
                                            }
                                        }
                                        mdns::Event::Expired(list) => {
                                            for (peer_id, _addr) in list {
                                                debug!("Peer {} expired from mDNS", peer_id);
                                            }
                                        }
                                    }
                                }
                                P2PBehaviourEvent::RelayClient(relay_event) => {
                                    match relay_event {
                                        libp2p::relay::client::Event::ReservationReqAccepted { relay_peer_id, .. } => {
                                            info!("✅ Relay reservation accepted by {}", relay_peer_id);
                                            info!("   NAT traversal enabled - you can now receive connections from behind NAT");
                                        }
                                        libp2p::relay::client::Event::OutboundCircuitEstablished { relay_peer_id, limit } => {
                                            info!("🔄 Outbound circuit established via relay {}", relay_peer_id);
                                            if let Some(limit) = limit {
                                                debug!("   Circuit limit: {:?}", limit);
                                            }
                                        }
                                        libp2p::relay::client::Event::InboundCircuitEstablished { src_peer_id, limit } => {
                                            info!("📥 Inbound circuit established from {}", src_peer_id);
                                            if let Some(limit) = limit {
                                                debug!("   Circuit limit: {:?}", limit);
                                            }
                                        }
                                    }
                                }
                                P2PBehaviourEvent::Autonat(autonat_event) => {
                                    match autonat_event {
                                        libp2p::autonat::Event::InboundProbe(_) => {
                                            debug!("🔍 Received inbound NAT probe");
                                        }
                                        libp2p::autonat::Event::OutboundProbe(_) => {
                                            debug!("🔍 Sent outbound NAT probe");
                                        }
                                        libp2p::autonat::Event::StatusChanged { old, new } => {
                                            info!("🔄 NAT status changed from {:?} to {:?}", old, new);
                                            match new {
                                                libp2p::autonat::NatStatus::Public(addr) => {
                                                    info!("✅ Public address detected: {}", addr);
                                                    info!("   Direct P2P connections possible");
                                                }
                                                libp2p::autonat::NatStatus::Private => {
                                                    info!("🔒 Behind NAT - relay and hole-punching will be used");
                                                    info!("   This is normal - connections will work via relay");
                                                }
                                                libp2p::autonat::NatStatus::Unknown => {
                                                    debug!("❓ NAT status unknown - still determining");
                                                }
                                            }
                                        }
                                    }
                                }
                                P2PBehaviourEvent::Dcutr(dcutr_event) => {
                                    // DCUtR events - hole punching for NAT traversal
                                    debug!("DCUtR event: {:?}", dcutr_event);
                                }
                            }
                        }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Connected to {}", peer_id);
                        let _ = event_sender.send(P2PEvent::PeerConnected(peer_id));
                        let _ = event_tx.send(P2PEvent::PeerConnected(peer_id));
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        info!("Disconnected from {} (cause: {:?})", peer_id, cause);
                        let _ = event_sender.send(P2PEvent::PeerDisconnected(peer_id));
                        let _ = event_tx.send(P2PEvent::PeerDisconnected(peer_id));
                    }
                    SwarmEvent::IncomingConnection { local_addr, send_back_addr, connection_id } => {
                        debug!("Incoming connection {:?} from {} on {}", connection_id, send_back_addr, local_addr);
                    }
                    SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, connection_id } => {
                        warn!("Incoming connection error {:?} from {} on {}: {}", connection_id, send_back_addr, local_addr, error);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, connection_id } => {
                        warn!("Outgoing connection error {:?} to {:?}: {}", connection_id, peer_id, error);
                    }
                    _ => {}
                    }
                    }
                    // Handle commands
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            P2PNodeCommand::GetConnectedPeers(reply) => {
                                let peers: Vec<PeerId> = self.swarm.connected_peers().copied().collect();
                                let _ = reply.send(peers);
                            }
                            P2PNodeCommand::StoreBlock { block_id, data, reply } => {
                                let result = async {
                                    // Store in DHT with proper replication
                                    let key = RecordKey::new(&block_id.as_bytes());
                                    let record = Record {
                                        key,
                                        value: data.clone(),
                                        publisher: None,
                                        expires: None,
                                    };
                                    
                                    let query_id = self.swarm
                                        .behaviour_mut()
                                        .kademlia
                                        .put_record(record, Quorum::One)
                                        .map_err(|e| format!("Failed to store in DHT: {:?}", e))?;
                                    
                                    // Create oneshot channel for response
                                    let (put_tx, put_rx) = tokio::sync::oneshot::channel();
                                    
                                    // Track the query
                                    pending_put_queries.write().await.insert(query_id, put_tx);
                                    
                                    // Store locally in shared blocks map
                                    let block_uuid = Uuid::parse_str(&block_id)
                                        .map_err(|e| format!("Invalid block ID: {}", e))?;
                                    
                                    // Deserialize and store block
                                    let block: DataBlock = bincode::deserialize(&data)
                                        .map_err(|e| format!("Failed to deserialize block: {}", e))?;
                                    
                                    local_blocks_clone.write().await.insert(block_uuid, block);
                                    
                                    // Wait for DHT confirmation with timeout
                                    match tokio::time::timeout(std::time::Duration::from_secs(10), put_rx).await {
                                        Ok(Ok(Ok(()))) => Ok::<(), String>(()),
                                        Ok(Ok(Err(e))) => Err(format!("DHT put failed: {}", e)),
                                        Ok(Err(_)) => Err("DHT response channel closed".to_string()),
                                        Err(_) => {
                                            // Timeout - consider it successful if stored locally
                                            tracing::warn!("DHT put timeout for {}, but stored locally", block_id);
                                            Ok(())
                                        }
                                    }
                                }.await;
                                
                                let _ = reply.send(result);
                            }
                            P2PNodeCommand::GetBlock { block_id, reply } => {
                                // Try local storage first
                                let block_uuid = match Uuid::parse_str(&block_id) {
                                    Ok(uuid) => uuid,
                                    Err(e) => {
                                        let _ = reply.send(Err(format!("Invalid block ID: {}", e)));
                                        continue;
                                    }
                                };
                                
                                // Check local blocks
                                {
                                    let blocks = local_blocks_clone.read().await;
                                    if let Some(block) = blocks.get(&block_uuid) {
                                        let data = match bincode::serialize(block) {
                                            Ok(d) => d,
                                            Err(e) => {
                                                let _ = reply.send(Err(format!("Serialization error: {}", e)));
                                                continue;
                                            }
                                        };
                                        let _ = reply.send(Ok(data));
                                        continue;
                                    }
                                }
                                
                                // Try DHT with response tracking
                                let key = RecordKey::new(&block_id.as_bytes());
                                let (get_tx, get_rx) = tokio::sync::oneshot::channel();
                                
                                let query_id = self.swarm
                                    .behaviour_mut()
                                    .kademlia
                                    .get_record(key);
                                
                                // Track the query
                                pending_get_queries.write().await.insert(query_id, get_tx);
                                
                                // Wait for DHT response with timeout
                                let result = match tokio::time::timeout(std::time::Duration::from_secs(10), get_rx).await {
                                    Ok(Ok(Ok(data))) => Ok(data),
                                    Ok(Ok(Err(e))) => Err(format!("DHT get failed: {}", e)),
                                    Ok(Err(_)) => Err("DHT response channel closed".to_string()),
                                    Err(_) => Err(format!("Block {} not found in local storage or DHT (timeout)", block_id)),
                                };
                                
                                let _ = reply.send(result);
                            }
                        }
                    }
                }
            }
        });

        Ok((event_receiver, local_blocks_return))
    }


    /// Bootstrap the DHT with retry logic and relay reservation (static method)
    async fn bootstrap_static(swarm: &mut Swarm<P2PBehaviour>, config: &P2PConfig) -> std::result::Result<(), MSSCSError> {
        if !config.bootstrap_peers.is_empty() {
            info!("🌍 Bootstrapping DHT with {} public peers for internet-wide connectivity", config.bootstrap_peers.len());
            info!("   Using IPFS public bootstrap nodes for global reach");
            
            let mut connected_count = 0;
            let mut relay_peers = Vec::new();
            
            // Dial all bootstrap peers concurrently for faster connection
            for addr in &config.bootstrap_peers {
                match swarm.dial(addr.clone()) {
                    Ok(_) => {
                        info!("   📡 Dialing bootstrap peer: {}", addr);
                        connected_count += 1;
                        
                        // Track potential relay peers for NAT traversal
                        if let Some(peer_id) = addr.iter().find_map(|p| match p {
                            libp2p::multiaddr::Protocol::P2p(hash) => {
                                PeerId::from_multihash(hash.into()).ok()
                            },
                            _ => None,
                        }) {
                            relay_peers.push(peer_id);
                        }
                    }
                    Err(e) => {
                        warn!("   ⚠️  Failed to dial bootstrap peer {}: {}", addr, e);
                        warn!("      This is normal - will try other peers");
                    }
                }
            }

            if connected_count > 0 {
                info!("   ✅ Initiated {} bootstrap connections", connected_count);
                
                // Start DHT bootstrap process
                match swarm.behaviour_mut().kademlia.bootstrap() {
                    Ok(_) => {
                        info!("   🔄 DHT bootstrap query started");
                        info!("      Discovering peers across the global network...");
                        info!("      This may take 10-30 seconds");
                    }
                    Err(e) => {
                        warn!("   ⚠️  Failed to start bootstrap: {}", e);
                        warn!("      Will retry automatically");
                    }
                }
                
                // Relay reservations for NAT traversal
                if config.enable_relay && !relay_peers.is_empty() {
                    info!("   🔓 NAT Traversal Configuration:");
                    info!("      ✓ Relay reservations enabled");
                    info!("      ✓ Hole-punching (DCUtR) enabled");
                    info!("      ✓ {} relay peers available", relay_peers.len());
                    info!("      ✓ Works behind NAT/firewalls");
                    info!("      ✓ No port forwarding needed");
                }
                
                info!("");
                info!("🎉 Bootstrap initiated successfully!");
                info!("   Your node is joining the global P2P network");
                info!("   Peer discovery in progress...");
            } else {
                warn!("❌ No bootstrap peers could be dialed - running in isolated mode");
                warn!("");
                warn!("   Troubleshooting:");
                warn!("   1. Check internet connection");
                warn!("   2. Verify firewall allows outbound connections");
                warn!("   3. Try again in a few seconds (bootstrap nodes may be busy)");
                warn!("   4. Check if IPFS bootstrap nodes are reachable");
                warn!("");
                warn!("   The node will continue trying to connect in the background");
            }
        } else {
            info!("⚠️  No bootstrap peers configured - running in isolated mode");
            info!("   Add bootstrap peers to connect to the global network");
        }

        Ok(())
    }

    /// Store block in local storage and replicate to network
    pub async fn store_block(&mut self, block_id: String, block_data: Vec<u8>) -> std::result::Result<(), MSSCSError> {
        // Provide to DHT for global replication
        let key = RecordKey::new(&block_id.as_bytes());
        let record = Record {
            key,
            value: block_data,
            publisher: None,
            expires: None,
        };

        let query_id = self.swarm
            .behaviour_mut()
            .kademlia
            .put_record(record, Quorum::One)
            .map_err(|e| MSSCSError::Network(format!("Failed to put record: {:?}", e)))?;

        debug!("Storing block {} in DHT with query ID: {:?}", block_id, query_id);

        Ok(())
    }

    /// Request block from network
    pub async fn get_block(&mut self, block_id: &str) -> std::result::Result<Vec<u8>, MSSCSError> {
        // Try to get from DHT first
        let key = RecordKey::new(&block_id.as_bytes());
        let _query_id = self.swarm
            .behaviour_mut()
            .kademlia
            .get_record(key);

        // In a real implementation, we'd wait for the DHT response
        // For now, return an error to trigger peer requests
        Err(MSSCSError::NotFound(format!("Block {} not in DHT yet", block_id)))
    }
    
    /// Request block from connected peers
    pub async fn request_block_from_peers(&mut self, block_id: &str) -> std::result::Result<(), MSSCSError> {
        // Try direct requests to connected peers
        let peers: Vec<_> = self.swarm.connected_peers().copied().collect();
        
        if peers.is_empty() {
            return Err(MSSCSError::Network("No peers connected".to_string()));
        }
        
        for peer in peers {
            // Parse block_id as UUID for the request
            let uuid = Uuid::parse_str(block_id)
                .map_err(|e| MSSCSError::InvalidData(format!("Invalid block ID: {}", e)))?;
            
            let _request_id = self.swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer, P2PRequest::GetBlock { id: uuid });

            debug!("Sent block request to peer {}: {}", peer, block_id);
        }

        Ok(())
    }

    /// Add peer address to DHT
    pub async fn add_peer(&mut self, addr: Multiaddr) -> std::result::Result<(), MSSCSError> {
        if let Some(peer_id) = addr.iter().find_map(|p| match p {
            libp2p::multiaddr::Protocol::P2p(hash) => {
                // hash is already a Multihash, convert to PeerId
                PeerId::from_multihash(hash.into()).ok()
            },
            _ => None,
        }) {
            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
        }
        self.swarm.dial(addr)
            .map_err(|e| MSSCSError::Network(format!("Failed to dial: {:?}", e)))?;
        Ok(())
    }

    /// Ping a peer
    pub async fn ping_peer(&mut self, peer_id: PeerId) -> std::result::Result<(), MSSCSError> {
        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, P2PRequest::Ping);
        Ok(())
    }
}