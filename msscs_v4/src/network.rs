// Network module - P2P implementation with libp2p Kademlia DHT
use crate::block::DataBlock;
use crate::config::Config;
use crate::error::{MSSCSError, Result};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    RequestBlock { uuid: Uuid },
    ResponseBlock { block: Option<DataBlock> },
    StoreBlock { block: DataBlock },
    Ping { node_id: String },
    Pong { node_id: String },
}

/// Network node with DHT support
pub struct Node {
    pub addr: String,
    pub node_id: String,
    pub local_blocks: Arc<RwLock<HashMap<String, DataBlock>>>,
    pub peers: Arc<RwLock<Vec<String>>>,
    pub peer_id: PeerId,
    config: Arc<Config>,
}

impl Node {
    /// Create new node instance
    pub fn new(config: Arc<Config>) -> Self {
        let node_id = Uuid::new_v4().to_string();
        let addr = format!("127.0.0.1:{}", config.port);
        let peer_id = PeerId::random();
        
        Node {
            addr,
            node_id,
            local_blocks: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(config.bootstrap_peers.clone())),
            peer_id,
            config,
        }
    }

    /// Start Kademlia DHT
    pub async fn start_dht(&self, bootstrap_peers: Vec<String>) -> Result<()> {
        tracing::info!("Starting Kademlia DHT with {} bootstrap peers", bootstrap_peers.len());
        
        // DHT will be managed in a background task
        // For now, just add bootstrap peers to our peer list
        let mut peers = self.peers.write().await;
        for peer in bootstrap_peers {
            if !peers.contains(&peer) {
                peers.push(peer);
            }
        }
        
        tracing::info!("DHT initialized with {} peers", peers.len());
        Ok(())
    }

    /// Start P2P listener
    pub async fn run_p2p_listener(self: Arc<Self>) -> Result<()> {
        let listener = TcpListener::bind(&self.addr)
            .await
            .map_err(|e| MSSCSError::Network(format!("Failed to bind to {}: {}", self.addr, e)))?;
        
        tracing::info!("P2P listener started on {}", self.addr);
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tracing::debug!("Accepted connection from {}", addr);
                        let node = self.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, node).await {
                                tracing::error!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Handle incoming P2P connection
    async fn handle_connection(mut stream: TcpStream, node: Arc<Node>) -> Result<()> {
        // Read message length (4 bytes)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let msg_len = u32::from_be_bytes(len_bytes) as usize;
        
        // Read message
        let mut msg_bytes = vec![0u8; msg_len];
        stream.read_exact(&mut msg_bytes).await?;
        
        let message: Message = bincode::deserialize(&msg_bytes)?;
        
        // Handle message
        match message {
            Message::RequestBlock { uuid } => {
                let blocks = node.local_blocks.read().await;
                let block = blocks.get(&uuid.to_string()).cloned();
                
                let response = Message::ResponseBlock { block };
                let response_bytes = bincode::serialize(&response)?;
                
                stream.write_all(&(response_bytes.len() as u32).to_be_bytes()).await?;
                stream.write_all(&response_bytes).await?;
            }
            Message::StoreBlock { block } => {
                tracing::info!("Received block {} to store", block.uuid);
                let mut blocks = node.local_blocks.write().await;
                blocks.insert(block.uuid.to_string(), block);
            }
            Message::Ping { node_id: _ } => {
                let response = Message::Pong { node_id: node.node_id.clone() };
                let response_bytes = bincode::serialize(&response)?;
                
                stream.write_all(&(response_bytes.len() as u32).to_be_bytes()).await?;
                stream.write_all(&response_bytes).await?;
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Replicate block to peers
    pub async fn replicate_block(&self, block: &DataBlock) -> Result<()> {
        let peers = self.peers.read().await;
        let replication_factor = self.config.replication_factor.min(peers.len());
        
        for peer_addr in peers.iter().take(replication_factor) {
            let block_clone = block.clone();
            let peer_addr_clone = peer_addr.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::send_store_block(&peer_addr_clone, &block_clone).await {
                    tracing::warn!("Failed to replicate block to {}: {}", peer_addr_clone, e);
                } else {
                    tracing::info!("Replicated block {} to {}", block_clone.uuid, peer_addr_clone);
                }
            });
        }
        
        Ok(())
    }

    /// Send StoreBlock message to peer
    async fn send_store_block(peer_addr: &str, block: &DataBlock) -> Result<()> {
        Self::retry_with_backoff(3, || async {
            let mut stream = TcpStream::connect(peer_addr).await
                .map_err(|e| MSSCSError::Network(format!("Failed to connect to {}: {}", peer_addr, e)))?;
            
            let message = Message::StoreBlock { block: block.clone() };
            let msg_bytes = bincode::serialize(&message)?;
            
            stream.write_all(&(msg_bytes.len() as u32).to_be_bytes()).await?;
            stream.write_all(&msg_bytes).await?;
            
            Ok(())
        }).await
    }

    /// Get block from peer
    pub async fn get_block_from_peer(&self, peer_addr: &str, uuid: &Uuid) -> Result<Option<DataBlock>> {
        Self::retry_with_backoff(3, || async {
            let mut stream = TcpStream::connect(peer_addr).await
                .map_err(|e| MSSCSError::Network(format!("Failed to connect to {}: {}", peer_addr, e)))?;
            
            let message = Message::RequestBlock { uuid: *uuid };
            let msg_bytes = bincode::serialize(&message)?;
            
            stream.write_all(&(msg_bytes.len() as u32).to_be_bytes()).await?;
            stream.write_all(&msg_bytes).await?;
            
            // Read response
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes).await?;
            let msg_len = u32::from_be_bytes(len_bytes) as usize;
            
            let mut msg_bytes = vec![0u8; msg_len];
            stream.read_exact(&mut msg_bytes).await?;
            
            let response: Message = bincode::deserialize(&msg_bytes)?;
            
            match response {
                Message::ResponseBlock { block } => Ok(block),
                _ => Err(MSSCSError::Network("Unexpected response".to_string())),
            }
        }).await
    }

    /// Retry operation with exponential backoff
    async fn retry_with_backoff<F, Fut, T>(max_attempts: u32, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut delay_ms = 100;
        
        loop {
            attempt += 1;
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= max_attempts {
                        return Err(e);
                    }
                    tracing::warn!("Attempt {} failed: {}. Retrying in {}ms...", attempt, e, delay_ms);
                    sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2; // Exponential backoff
                }
            }
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            addr: self.addr.clone(),
            node_id: self.node_id.clone(),
            local_blocks: self.local_blocks.clone(),
            peers: self.peers.clone(),
            peer_id: self.peer_id,
            config: self.config.clone(),
        }
    }
}
