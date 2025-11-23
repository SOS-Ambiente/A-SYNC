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
#[behaviour(to_swarm = "P2PBehaviourEvent")]
pub struct P2PBehaviour {
    kademlia: libp2p::kad::Behaviour<MemoryStore>,
    request_response: libp2p::request_response::Behaviour<P2PCodec>,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum P2PBehaviourEvent {
    Kademlia(libp2p::kad::Event),
    RequestResponse(libp2p::request_response::Event<P2PRequest, P2PResponse>),
    Mdns(mdns::Event),
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
}

/// Main P2P Node implementation
pub struct P2PNode {
    swarm: Swarm<P2PBehaviour>,
    event_sender: mpsc::UnboundedSender<P2PEvent>,
    local_blocks: Arc<RwLock<HashMap<Uuid, DataBlock>>>,
    pending_requests: Arc<RwLock<HashMap<QueryId, Uuid>>>,
    command_receiver: Option<mpsc::UnboundedReceiver<P2PNodeCommand>>,
    command_sender: mpsc::UnboundedSender<P2PNodeCommand>,
}

impl P2PNode {
    /// Create new P2P node
    pub async fn new(config: P2PConfig) -> std::result::Result<Self, MSSCSError> {
        info!("Initializing P2P node with config: {:?}", config);

        // Generate or load keypair
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        // Create network behavior
        let store = MemoryStore::new(peer_id);
        let kademlia = libp2p::kad::Behaviour::new(peer_id, store);

        let request_response = libp2p::request_response::Behaviour::new(
            std::iter::once(("/msscs/1.0.0", ProtocolSupport::Full)),
            libp2p::request_response::Config::default(),
        );

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
            .map_err(|e| MSSCSError::Network(format!("Failed to create mDNS: {}", e)))?;

        let behaviour = P2PBehaviour {
            kademlia,
            request_response,
            mdns,
        };

        // Create swarm with new API
        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| MSSCSError::Network(format!("Failed to build transport: {}", e)))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| MSSCSError::Network(format!("Failed to create behaviour: {}", e)))?
            .build();

        // Listen on specified port
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port);
        let addr: Multiaddr = listen_addr.parse()
            .map_err(|e: libp2p::multiaddr::Error| MSSCSError::Network(format!("Invalid address: {}", e)))?;
        swarm.listen_on(addr)
            .map_err(|e| MSSCSError::Network(format!("Failed to listen on port: {}", e)))?;

        let (event_sender, _event_receiver) = mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        info!("P2P node created with peer ID: {}", peer_id);

        Ok(P2PNode {
            swarm,
            event_sender,
            local_blocks: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            command_receiver: Some(cmd_rx),
            command_sender: cmd_tx,
        })
    }
    
    /// Get the command sender for this node
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<P2PNodeCommand> {
        self.command_sender.clone()
    }

    /// Start the P2P node and return event receiver
    pub async fn start(mut self) -> std::result::Result<(mpsc::UnboundedReceiver<P2PEvent>, Arc<RwLock<HashMap<Uuid, DataBlock>>>), MSSCSError> {
        let event_sender = self.event_sender.clone();
        let local_blocks_clone = self.local_blocks.clone();
        let local_blocks_return = self.local_blocks.clone();
        let (_, event_receiver) = mpsc::unbounded_channel();
        
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
                                        libp2p::kad::Event::OutboundQueryProgressed { result, .. } => {
                                            match result {
                                                libp2p::kad::QueryResult::Bootstrap(Ok(BootstrapOk { .. })) => {
                                                    let _ = event_sender.send(P2PEvent::BootstrapComplete);
                                                }
                                                libp2p::kad::QueryResult::GetClosestPeers(Ok(result)) => {
                                                    if result.peers.is_empty() {
                                                        let _ = event_sender.send(P2PEvent::Error("No peers found".to_string()));
                                                    }
                                                }
                                                libp2p::kad::QueryResult::GetRecord(Ok(result)) => {
                                                    debug!("Got record: {:?}", result);
                                                }
                                                libp2p::kad::QueryResult::PutRecord(Ok(_)) => {
                                                    debug!("Put record successful");
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
                        }
                    }
                }
            }
        });

        Ok((event_receiver, local_blocks_return))
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerId> {
        self.swarm.connected_peers().copied().collect()
    }

    /// Bootstrap the DHT
    pub async fn bootstrap(&mut self, config: &P2PConfig) -> std::result::Result<(), MSSCSError> {
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
    pub async fn store_block(&mut self, block: DataBlock) -> std::result::Result<(), MSSCSError> {
        let block_id = block.uuid;

        // Store locally
        self.local_blocks.write().await.insert(block_id, block.clone());

        // Provide to DHT
        let key = RecordKey::new(&block_id.to_string());
        let record = Record {
            key,
            value: serde_json::to_vec(&block)?,
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
    pub async fn get_block(&mut self, block_id: Uuid) -> std::result::Result<(), MSSCSError> {
        // Try to get from DHT first
        let key = RecordKey::new(&block_id.to_string());
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

            debug!("Sent block request to peer {}: {:?}", peer, request_id);
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