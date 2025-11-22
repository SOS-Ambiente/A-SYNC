// Virtual File System module
use crate::block::DataBlock;
use crate::config::Config;
use crate::error::{MSSCSError, Result};
use crate::network::Node;
use crate::persistence::PersistenceManager;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Virtual File System for distributed storage
pub struct VirtualFileSystem {
    local_blocks: HashMap<String, DataBlock>,
    file_manifest: HashMap<String, Uuid>,
    node: Option<Arc<Node>>,
    persistence: Arc<PersistenceManager>,
    config: Arc<Config>,
}

impl VirtualFileSystem {
    /// Create new VFS instance
    pub fn new(config: Arc<Config>, persistence: Arc<PersistenceManager>) -> Result<Self> {
        // Load existing blocks and manifest from disk
        let local_blocks = persistence.load_all_blocks()?;
        let file_manifest = persistence.load_manifest()?;
        
        tracing::info!("VFS initialized with {} blocks and {} files", 
            local_blocks.len(), file_manifest.len());
        
        Ok(VirtualFileSystem {
            local_blocks,
            file_manifest,
            node: None,
            persistence,
            config,
        })
    }
    
    /// Associate VFS with network node
    pub fn set_node(&mut self, node: Arc<Node>) {
        self.node = Some(node);
    }
    
    /// Write file to distributed storage
    pub async fn write_file(&mut self, path: &Path, data: &[u8]) -> Result<Uuid> {
        self.write_file_with_progress(path, data, |_, _| {}).await
    }

    /// Write file with progress callback
    pub async fn write_file_with_progress<F>(&mut self, path: &Path, data: &[u8], mut progress_callback: F) -> Result<Uuid>
    where
        F: FnMut(usize, usize),
    {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("Writing file '{}' ({} bytes)", path_str, data.len());
        
        // Split data into chunks
        let chunk_size = self.config.chunk_size;
        let chunks: Vec<&[u8]> = data.chunks(chunk_size).collect();
        let total_chunks = chunks.len();
        
        tracing::debug!("Split into {} chunks of {} bytes", total_chunks, chunk_size);
        
        // Create blocks in reverse order (last chunk first) to build chain
        let mut previous_uuid: Option<Uuid> = None;
        let mut previous_hash = [0u8; 32];
        let mut first_block_uuid: Option<Uuid> = None;
        
        for (i, chunk) in chunks.iter().enumerate().rev() {
            let node_index = i as u64;
            
            // Create block (quantum-resistant encryption)
            let block = DataBlock::new(chunk, node_index, previous_uuid, previous_hash)?;
            let block_uuid = block.uuid;
            
            // Calculate hash for next block in chain
            previous_hash = block.calculate_hash()?;
            previous_uuid = Some(block_uuid);
            
            // Store block locally
            self.local_blocks.insert(block_uuid.to_string(), block.clone());
            
            // Persist to disk
            self.persistence.save_block(&block)?;
            
            // Replicate to network (P2P distribution)
            if let Some(node) = &self.node {
                node.replicate_block(&block).await?;
            }
            
            // Remember first block UUID
            if i == 0 {
                first_block_uuid = Some(block_uuid);
            }
            
            // Report progress (from last to first, so invert)
            let completed = total_chunks - i;
            progress_callback(completed, total_chunks);
        }
        
        let first_uuid = first_block_uuid.ok_or_else(|| 
            MSSCSError::InvalidData("No blocks created".to_string()))?;
        
        // Update manifest
        self.file_manifest.insert(path_str.clone(), first_uuid);
        self.persistence.save_manifest(&self.file_manifest)?;
        
        tracing::info!("File '{}' written successfully with {} blocks", path_str, total_chunks);
        Ok(first_uuid)
    }
    
    /// Read file from distributed storage
    pub async fn read_file(&mut self, path: &Path) -> Result<Vec<u8>> {
        self.read_file_with_progress(path, |_, _| {}).await
    }

    /// Read file with progress callback
    pub async fn read_file_with_progress<F>(&mut self, path: &Path, mut progress_callback: F) -> Result<Vec<u8>>
    where
        F: FnMut(usize, usize),
    {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("Reading file '{}'", path_str);
        
        // Look up first block UUID
        let first_uuid = self.file_manifest.get(&path_str)
            .ok_or_else(|| MSSCSError::NotFound(format!("File '{}' not found", path_str)))?;
        
        // Collect all blocks in chain
        let mut blocks = Vec::new();
        let mut current_uuid = *first_uuid;
        
        loop {
            // Get block (local or network)
            let block = self.get_block(&current_uuid).await?;
            blocks.push(block.clone());
            
            // Follow chain
            if let Some(prev_uuid) = block.previous_uuid {
                current_uuid = prev_uuid;
            } else {
                break; // Genesis block reached
            }
        }
        
        let total_blocks = blocks.len();
        tracing::debug!("Retrieved {} blocks for file '{}'", total_blocks, path_str);
        
        // Decode and concatenate data with progress
        let mut file_data = Vec::new();
        for (i, block) in blocks.iter().enumerate() {
            let chunk_data = block.decode(block.node_index)?;
            file_data.extend_from_slice(&chunk_data);
            
            // Report progress
            progress_callback(i + 1, total_blocks);
        }
        
        tracing::info!("File '{}' read successfully ({} bytes)", path_str, file_data.len());
        Ok(file_data)
    }
    
    /// Get block from local storage or network
    async fn get_block(&mut self, uuid: &Uuid) -> Result<DataBlock> {
        // Check local storage first
        if let Some(block) = self.local_blocks.get(&uuid.to_string()) {
            return Ok(block.clone());
        }
        
        // Try to load from disk
        if let Ok(block) = self.persistence.load_block(uuid) {
            self.local_blocks.insert(uuid.to_string(), block.clone());
            return Ok(block);
        }
        
        // Query network peers
        if let Some(node) = &self.node {
            let peers = node.peers.read().await;
            
            for peer_addr in peers.iter() {
                match node.get_block_from_peer(peer_addr, uuid).await {
                    Ok(Some(block)) => {
                        tracing::info!("Retrieved block {} from peer {}", uuid, peer_addr);
                        
                        // Cache locally
                        self.local_blocks.insert(uuid.to_string(), block.clone());
                        self.persistence.save_block(&block)?;
                        
                        return Ok(block);
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        tracing::warn!("Failed to get block from {}: {}", peer_addr, e);
                        continue;
                    }
                }
            }
        }
        
        Err(MSSCSError::NotFound(format!("Block {} not found locally or on network", uuid)))
    }
    
    /// Delete file from manifest
    pub async fn delete_file(&mut self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("Deleting file '{}'", path_str);
        
        self.file_manifest.remove(&path_str)
            .ok_or_else(|| MSSCSError::NotFound(format!("File '{}' not found", path_str)))?;
        
        self.persistence.save_manifest(&self.file_manifest)?;
        
        tracing::info!("File '{}' deleted from manifest", path_str);
        Ok(())
    }
    
    /// List all files in manifest
    pub fn list_files(&self) -> Vec<String> {
        self.file_manifest.keys().cloned().collect()
    }
    
    /// Get block count
    pub fn block_count(&self) -> usize {
        self.local_blocks.len()
    }
    
    /// Get storage size in bytes
    pub fn storage_bytes(&self) -> u64 {
        self.local_blocks.values()
            .map(|b| bincode::serialize(b).map(|v| v.len() as u64).unwrap_or(0))
            .sum()
    }
}
