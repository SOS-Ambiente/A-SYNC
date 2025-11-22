// Virtual File System module
use crate::block::{DataBlock, FileMetadata};
use crate::config::Config;
use crate::error::{MSSCSError, Result};
use crate::network::Node;
use crate::persistence::PersistenceManager;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// File writing options
#[derive(Debug, Clone)]
pub struct FileWriteOptions {
    pub chunk_size: Option<u64>,
    pub compress: bool,
    pub encrypt: bool,
    pub replication_factor: u32,
}

impl Default for FileWriteOptions {
    fn default() -> Self {
        Self {
            chunk_size: None, // Use default from config
            compress: true,
            encrypt: true,
            replication_factor: 3,
        }
    }
}

/// File reading options
#[derive(Debug, Clone)]
pub struct FileReadOptions {
    pub decompress: bool,
    pub verify_integrity: bool,
    pub prefer_local: bool,
}

impl Default for FileReadOptions {
    fn default() -> Self {
        Self {
            decompress: true,
            verify_integrity: true,
            prefer_local: true,
        }
    }
}

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

        self.write_file_with_options(path, data, Some(Box::new(move |bytes, total| {
            let progress = (bytes as f64 / total as f64 * 100.0) as usize;
            progress_callback(progress, 100);
        }))), FileWriteOptions::default()).await
    }

    /// Enhanced write file with options and progress callback (for new API)
    pub async fn write_file_with_options<F>(
        &mut self,
        path: &Path,
        data: &[u8],
        progress_callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
        options: FileWriteOptions,
    ) -> Result<Uuid> {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("Writing file '{}' ({} bytes) with options", path_str, data.len());

        let chunk_size = options.chunk_size.unwrap_or(self.config.chunk_size);

        // Compress data first using Huffman if enabled
        let processed_data = if options.compress {
            tracing::debug!("Compressing data with Huffman algorithm");
            let mut compressor = crate::huffman::HuffmanCompressor::new();
            compressor.compress(data)?
        } else {
            data.to_vec()
        };

        let total_chunks = (processed_data.len() as f64 / chunk_size as f64).ceil() as usize;
        let mut chunk_ids = Vec::new();
        let mut previous_hash: Option<String> = None;
        let mut bytes_processed = 0u64;

        // Process chunks in reverse order for cryptographic chaining
        for chunk_index in (0..total_chunks).rev() {
            let start = chunk_index * chunk_size;
            let end = std::cmp::min(start + chunk_size, processed_data.len());
            let chunk_data = &processed_data[start..end];

            // Create data block with enhanced features
            let block = DataBlock::new(
                chunk_data.to_vec(),
                previous_hash,
            )?;

            // Add to local storage
            self.local_blocks.insert(block.uuid.to_string(), block.clone());

            // Persist to disk
            self.persistence.save_block(&block)?;

            // Replicate to peers if enabled
            if let Some(node) = &self.node {
                node.replicate_block(&block).await?;
            }

            chunk_ids.push(block.uuid.to_string());
            previous_hash = Some(block.get_hash());

            bytes_processed += chunk_data.len() as u64;

            if let Some(ref callback) = progress_callback {
                callback(bytes_processed, processed_data.len() as u64);
            }
        }

        // Store file metadata using new FileMetadata struct
        let metadata = FileMetadata::new(
            path.to_path_buf(),
            data.len() as u64,
            processed_data.len() as u64,
            chunk_ids.iter().rev().cloned().collect(), // Reverse to correct order
            crate::block::calculate_checksum(data),
        );

        self.persistence.save_file_metadata(&metadata)?;

        tracing::info!("File '{}' written successfully with {} chunks", path_str, total_chunks);
        Ok(Uuid::parse_str(&chunk_ids[0])?) // Return first chunk ID as file ID
    }

    /// Enhanced read file with options and progress callback (for new API)
    pub async fn read_file_with_options<F>(
        &mut self,
        file_id: &str,
        progress_callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
        options: FileReadOptions,
    ) -> Result<Vec<u8>> {
        tracing::info!("Reading file '{}' with options", file_id);

        // Load file metadata
        let metadata = self.persistence.load_file_metadata(file_id)?
            .ok_or_else(|| MSSCSError::FileNotFound(format!("File metadata not found: {}", file_id)))?;

        // Load all chunks
        let mut chunks = Vec::with_capacity(metadata.chunk_count);
        let mut bytes_processed = 0u64;

        for chunk_id in metadata.chunk_ids.iter().rev() {
            let chunk_data = match self.get_chunk_from_storage(chunk_id).await {
                Ok(data) => data,
                Err(_) => {
                    // Try to fetch from P2P network
                    let uuid = Uuid::parse_str(chunk_id)
                        .map_err(|_| MSSCSError::InvalidData(format!("Invalid chunk ID: {}", chunk_id)))?;

                    if let Some(node) = &self.node {
                        let mut block_found = false;

                        let peers = node.peers.read().await;
                        for peer_addr in peers.iter() {
                            if let Ok(Some(block)) = node.get_block_from_peer(peer_addr, &uuid).await {
                                chunks.push(block.data.clone());
                                block_found = true;
                                break;
                            }
                        }

                        if !block_found {
                            return Err(MSSCSError::Network(format!("Chunk {} not found locally or on network", chunk_id)));
                        }

                        continue;
                    } else {
                        return Err(MSSCSError::Network(format!("Chunk {} not found locally and no network available", chunk_id)));
                    }
                }
            };

            chunks.push(chunk_data);
            bytes_processed += chunk_data.len() as u64;

            if let Some(ref callback) = progress_callback {
                callback(bytes_processed, metadata.compressed_size);
            }
        }

        // Reconstruct file from chunks
        let compressed_data = chunks.concat();

        // Decompress if enabled and needed
        let final_data = if options.decompress && metadata.compressed_size != metadata.original_size {
            tracing::debug!("Decompressing data with Huffman algorithm");
            let mut decompressor = crate::huffman::HuffmanDecompressor::new();
            decompressor.decompress(&compressed_data)?
        } else {
            compressed_data
        };

        // Verify integrity if enabled
        if options.verify_integrity {
            let checksum = crate::block::calculate_checksum(&final_data);
            if checksum != metadata.checksum {
                return Err(MSSCSError::CorruptedData("File checksum mismatch".to_string()));
            }
        }

        tracing::info!("File '{}' read successfully ({} bytes)", file_id, final_data.len());
        Ok(final_data)
    }

    /// Get chunk data from local storage
    async fn get_chunk_from_storage(&mut self, chunk_id: &str) -> Result<Vec<u8>> {
        // Check local memory first
        if let Some(block) = self.local_blocks.get(chunk_id) {
            return Ok(block.get_compressed_data().to_vec());
        }

        // Try to load from disk
        let uuid = Uuid::parse_str(chunk_id)
            .map_err(|_| MSSCSError::InvalidData(format!("Invalid chunk ID: {}", chunk_id)))?;

        if let Ok(block) = self.persistence.load_block(&uuid) {
            self.local_blocks.insert(chunk_id.to_string(), block.clone());
            return Ok(block.get_compressed_data().to_vec());
        }

        Err(MSSCSError::NotFound(format!("Chunk {} not found in local storage", chunk_id)))
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
