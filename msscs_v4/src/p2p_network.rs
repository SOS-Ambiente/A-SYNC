// P2P Network module - Real libp2p Kademlia DHT implementation
use crate::block::DataBlock;
use crate::error::{MSSCSError, Result};
use futures::prelude::*;
use libp2p::{
    core::{self, upgrade, Multiaddr, PeerId, Transport},
    dns,
    identity::{self, Keypair},
    kad::{self, store::MemoryStore, Kademlia, KademliaEvent, QueryId, BootstrapOk},
    mplex,
    noise::{Keypair as NoiseKeypair, X25519Spec, X25519},
    request_response::{self, ProtocolSupport, RequestResponse, RequestResponseConfig},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp,
    yamux,
    Swarm,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// P2P Configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub listen_port: u16,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub max_peers: usize,
    pub replication_factor: usize,
    pub enable_mdns: bool,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 0, // Random port
            bootstrap_peers: Vec::new(),
            max_peers: 50,
            replication_factor: 3,
            enable_mdns: true,
        }
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

/// P2P Network behavior combining Kademlia DHT and Request-Response
#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub request_response: RequestResponse<P2PCodec>,
    pub mdns: libp2p::mdns::Mdns,
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

#[derive(Debug, Clone)]
pub struct P2PCodec;

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

/// Main P2P Node implementation
pub struct P2PNode {
    swarm: Swarm<P2PBehaviour>,
    event_sender: mpsc::UnboundedSender<P2PEvent>,
    local_blocks: Arc<RwLock<HashMap<Uuid, DataBlock>>>,
    pending_requests: Arc<RwLock<HashMap<QueryId, Uuid>>>,
}

impl P2PNode {
    /// Create new P2P node
    pub async fn new(config: P2PConfig) -> Result<Self> {
        info!("Initializing P2P node with config: {:?}", config);

        // Generate or load keypair
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        // Build transport
        let transport = tcp::TokioTcpTransport::default()
            .upgrade(core::upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(
                &NoiseKeypair::new().into_authentic(&keypair)?,
                X25519Spec::new(),
            ))
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        // Create network behavior
        let store = MemoryStore::new(peer_id);
        let kademlia = Kademlia::new(peer_id, store);

        let request_response = RequestResponse::new(
            [(b"/msscs/1.0.0", ProtocolSupport::Full)],
            RequestResponseConfig::default(),
        );

        let mdns = libp2p::mdns::Mdns::new(Default::default()).await
            .map_err(|e| MSSCSError::Network(format!("Failed to create mDNS: {}", e)))?;

        let behaviour = P2PBehaviour {
            kademlia,
            request_response,
            mdns,
        };

        // Create swarm
        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, peer_id);

        // Listen on specified port
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port);
        swarm.listen_on(listen_addr.parse()?)
            .map_err(|e| MSSCSError::Network(format!("Failed to listen on port: {}", e)))?;

        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

        info!("P2P node created with peer ID: {}", peer_id);

        Ok(P2PNode {
            swarm,
            event_sender,
            local_blocks: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the P2P node and return event receiver
    pub async fn start(mut self) -> Result<(mpsc::UnboundedReceiver<P2PEvent>, Arc<RwLock<HashMap<Uuid, DataBlock>>>)> {
        let event_sender = self.event_sender.clone();
        let local_blocks = self.local_blocks.clone();

        // Run swarm event loop in background
        tokio::spawn(async move {
            loop {
                match self.swarm.select_next_some().await {
                    SwarmEvent::Behaviour(event) => {
                        match event {
                            P2PBehaviourEvent::Kademlia(kad_event) => {
                                match kad_event {
                                    KademliaEvent::BootstrapOk(BootstrapOk { .. }) => {
                                        let _ = event_sender.send(P2PEvent::BootstrapComplete);
                                    }
                                    KademliaEvent::GetClosestPeersResult(result) => {
                                        if result.peers.is_empty() {
                                            let _ = event_sender.send(P2PEvent::Error("No peers found".to_string()));
                                        }
                                    }
                                    KademliaEvent::GetProvidersResult(result) => {
                                        debug!("Got providers: {:?}", result);
                                    }
                                    KademliaEvent::GetRecordResult(result) => {
                                        debug!("Got record: {:?}", result);
                                    }
                                    KademliaEvent::PutRecordResult(result) => {
                                        debug!("Put record: {:?}", result);
                                    }
                                    _ => {}
                                }
                            }
                            P2PBehaviourEvent::RequestResponse(event) => {
                                match event {
                                    request_response::Event::Message { message, .. } => {
                                        match message {
                                            request_response::Message::Request {
                                                request: P2PRequest::GetBlock { id },
                                                channel,
                                                peer
                                            } => {
                                                let blocks = local_blocks.read().await;
                                                let block = blocks.get(&id).cloned();

                                                if let Err(e) = self.swarm.send_response(channel, P2PResponse::Block { data: block }) {
                                                    warn!("Failed to send block response to {}: {}", peer, e);
                                                }

                                                let _ = event_sender.send(P2PEvent::BlockRequested { peer, block_id: id });
                                            }
                                            request_response::Event::Message {
                                                request: P2PRequest::Ping,
                                                channel,
                                                peer
                                            } => {
                                                let _ = self.swarm.send_response(channel, P2PResponse::Pong);
                                            }
                                            _ => {}
                                        }
                                    }
                                    request_response::Event::Response {
                                        response: P2PResponse::Block { data: Some(block) },
                                        peer
                                    } => {
                                        let _ = event_sender.send(P2PEvent::BlockReceived { peer, block });
                                    }
                                    request_response::Event::Response {
                                        response: P2PResponse::Block { data: None },
                                        peer
                                    } => {
                                        debug!("Peer {} doesn't have requested block", peer);
                                    }
                                    _ => {}
                                }
                            }
                            P2PBehaviourEvent::Mdns(event) => {
                                match event {
                                    libp2p::mdns::Event::Discovered(list) => {
                                        for (peer_id, addr) in list {
                                            info!("Discovered peer {} at {}", peer_id, addr);
                                            let _ = self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                            let _ = self.swarm.dial(peer_id);
                                        }
                                    }
                                    libp2p::mdns::Event::Expired(list) => {
                                        for (peer_id, _addr) in list {
                                            debug!("Peer {} expired from mDNS", peer_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Connected to {}", peer_id);
                        let _ = event_sender.send(P2PEvent::PeerConnected(peer_id));
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        info!("Disconnected from {} (cause: {:?})", peer_id, cause);
                        let _ = event_sender.send(P2PEvent::PeerDisconnected(peer_id));
                    }
                    SwarmEvent::IncomingConnection { local_addr, send_back_addr } => {
                        debug!("Incoming connection from {} on {}", send_back_addr, local_addr);
                    }
                    SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error } => {
                        warn!("Incoming connection error from {} on {}: {}", send_back_addr, local_addr, error);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        warn!("Outgoing connection error to {}: {}", peer_id, error);
                    }
                    _ => {}
                }
            }
        });

        Ok((event_receiver, local_blocks))
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerId> {
        self.swarm.connected_peers().copied().collect()
    }

    /// Bootstrap the DHT
    pub async fn bootstrap(&mut self, config: &P2PConfig) -> Result<()> {
        if !config.bootstrap_peers.is_empty() {
            for addr in &config.bootstrap_peers {
                if let Err(e) = self.swarm.dial(addr.clone()) {
                    warn!("Failed to dial bootstrap peer {}: {}", addr, e);
                }
            }

            // Start bootstrap process
            if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
                warn!("Failed to start bootstrap: {}", e);
            }
        }

        Ok(())
    }

    /// Store block in local storage and replicate to network
    pub async fn store_block(&mut self, block: DataBlock) -> Result<()> {
        let block_id = block.uuid;

        // Store locally
        self.local_blocks.write().await.insert(block_id, block.clone());

        // Provide to DHT
        let key = kad::record::Key::new(&block_id.to_string());
        let record = kad::record::Record {
            key,
            value: serde_json::to_vec(&block)?,
            publisher: None,
            expires: None,
        };

        let query_id = self.swarm
            .behaviour_mut()
            .kademlia
            .put_record(record, kad::record::Quorum::One);

        debug!("Storing block {} in DHT with query ID: {:?}", block_id, query_id);

        Ok(())
    }

    /// Request block from network
    pub async fn get_block(&mut self, block_id: Uuid) -> Result<()> {
        // Try to get from DHT first
        let key = kad::record::Key::new(&block_id.to_string());
        let query_id = self.swarm
            .behaviour_mut()
            .kademlia
            .get_record(key);

        self.pending_requests.write().await.insert(query_id, block_id);

        // Also try direct requests to connected peers
        let peers: Vec<_> = self.swarm.connected_peers().copied().collect();
        for peer in peers {
            let request_id = self.swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer, P2PRequest::GetBlock { id: block_id });

            self.pending_requests.write().await.insert(request_id.into(), block_id);
        }

        Ok(())
    }

    /// Add peer address to DHT
    pub async fn add_peer(&mut self, addr: Multiaddr) -> Result<()> {
        if let Some(peer_id) = addr.iter().last().and_then(|p| match p {
            libp2p::multiaddr::Protocol::P2p(peer) => Some(PeerId::from_multihash(peer).unwrap()),
            _ => None,
        }) {
            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
        }
        self.swarm.dial(addr)?;
        Ok(())
    }

    /// Ping a peer
    pub async fn ping_peer(&mut self, peer_id: PeerId) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, P2PRequest::Ping);
        Ok(())
    }
}