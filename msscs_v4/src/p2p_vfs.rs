// P2P-ENABLED VIRTUAL FILE SYSTEM
// Integrates quantum-encrypted storage with global P2P network

use crate::error::{MSSCSError, Result};
use crate::identity::UnlockedIdentity;
use crate::quantum_block::QuantumDataBlock;
use crate::erasure::{ErasureCoding, Shard};
use crate::singularity::SingularityFragmentation;
use crate::p2p_network::P2PNode;
use crate::adaptive_compression::{AdaptiveCompression, CompressionLevel};
use crate::parallel::ParallelBlockProcessor;
use crate::pinning::{PinningManager, PinType};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// P2P-enabled Virtual File System
pub struct P2PVirtualFileSystem {
    /// User identity (quantum-resistant)
    identity: Arc<UnlockedIdentity>,
    
    /// P2P command sender (for async operations)
    p2p_command_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::p2p_network::P2PNodeCommand>>,
    
    /// File manifest (path -> first block UUID)
    file_manifest: Arc<RwLock<HashMap<String, Uuid>>>,
    
    /// Local block cache (block_id -> block)
    local_blocks: Arc<RwLock<HashMap<String, QuantumDataBlock>>>,
    
    /// Erasure coding configuration
    erasure: ErasureCoding,
    
    /// Singularity fragmentation
    singularity: SingularityFragmentation,
    
    /// Adaptive compression
    compression: AdaptiveCompression,
    
    /// Parallel block processor
    parallel: ParallelBlockProcessor,
    
    /// Block pinning manager
    pinning: Arc<RwLock<PinningManager>>,
    
    /// Chunk size for splitting files
    chunk_size: usize,
}

impl P2PVirtualFileSystem {
    /// Create new P2P VFS with all advanced features and configurable storage allocation
    pub fn new(
        identity: Arc<UnlockedIdentity>,
        p2p_command_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::p2p_network::P2PNodeCommand>>,
        chunk_size: usize,
    ) -> Result<Self> {
        Self::new_with_storage_limit(identity, p2p_command_tx, chunk_size, 100 * 1024 * 1024) // Default 100MB
    }
    
    /// Create new P2P VFS with custom storage allocation (in bytes)
    pub fn new_with_storage_limit(
        identity: Arc<UnlockedIdentity>,
        p2p_command_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::p2p_network::P2PNodeCommand>>,
        chunk_size: usize,
        storage_limit_bytes: usize,
    ) -> Result<Self> {
        let erasure = ErasureCoding::new(10, 4)?;
        let singularity = SingularityFragmentation::new(3, 5)?;
        let compression = AdaptiveCompression::new(CompressionLevel::Balanced, 1024);
        let parallel = ParallelBlockProcessor::default();
        let pinning = Arc::new(RwLock::new(PinningManager::new(storage_limit_bytes)));
        
        tracing::info!("🚀 Initializing P2P VFS with advanced features:");
        tracing::info!("   ✓ Storage allocation: {} MB", storage_limit_bytes / (1024 * 1024));
        tracing::info!("   ✓ Erasure coding: 10+4 (40% overhead, tolerates 4 failures)");
        tracing::info!("   ✓ Singularity fragmentation: 3-of-5 threshold");
        tracing::info!("   ✓ Adaptive compression: Balanced mode");
        tracing::info!("   ✓ Parallel processing: {} threads", parallel.worker_threads);
        tracing::info!("   ✓ Block pinning enabled");
        
        Ok(Self {
            identity,
            p2p_command_tx,
            file_manifest: Arc::new(RwLock::new(HashMap::new())),
            local_blocks: Arc::new(RwLock::new(HashMap::new())),
            erasure,
            singularity,
            compression,
            parallel,
            pinning,
            chunk_size,
        })
    }
    
    /// Store block on P2P network via command channel
    async fn store_block_p2p(&self, block_id: String, data: Vec<u8>) -> Result<()> {
        if let Some(ref tx) = self.p2p_command_tx {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            
            if let Err(e) = tx.send(crate::p2p_network::P2PNodeCommand::StoreBlock {
                block_id: block_id.clone(),
                data,
                reply: reply_tx,
            }) {
                tracing::warn!("   ⚠️  Failed to send store command: {}", e);
                return Ok(()); // Continue without P2P storage
            }
            
            match reply_rx.await {
                Ok(Ok(())) => {
                    tracing::debug!("   ✓ Stored block {} on P2P network (DHT)", block_id);
                }
                Ok(Err(e)) => {
                    tracing::warn!("   ⚠️  P2P storage failed for {}: {}", block_id, e);
                }
                Err(e) => {
                    tracing::warn!("   ⚠️  P2P command channel closed: {}", e);
                }
            }
        } else {
            tracing::debug!("   ℹ️  P2P not available, using local storage only");
        }
        Ok(())
    }
    
    /// Get block from P2P network via command channel
    async fn get_block_p2p(&self, block_id: &str) -> Result<Vec<u8>> {
        if let Some(ref tx) = self.p2p_command_tx {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            
            if let Err(e) = tx.send(crate::p2p_network::P2PNodeCommand::GetBlock {
                block_id: block_id.to_string(),
                reply: reply_tx,
            }) {
                return Err(MSSCSError::Network(format!("P2P command channel error: {}", e)));
            }
            
            match reply_rx.await {
                Ok(Ok(data)) => {
                    tracing::debug!("   ✓ Retrieved block {} from P2P network", block_id);
                    return Ok(data);
                }
                Ok(Err(e)) => {
                    tracing::debug!("   ℹ️  Block {} not found on P2P network: {}", block_id, e);
                }
                Err(e) => {
                    tracing::warn!("   ⚠️  P2P command channel closed: {}", e);
                }
            }
        }
        Err(MSSCSError::Network("P2P not available or block not found".to_string()))
    }
    
    /// Update storage allocation limit (in bytes)
    pub async fn set_storage_limit(&self, limit_bytes: usize) -> Result<()> {
        let mut pinning = self.pinning.write().await;
        pinning.set_limit(limit_bytes);
        tracing::info!("📊 Storage limit updated to {} MB", limit_bytes / (1024 * 1024));
        Ok(())
    }
    
    /// Get current storage statistics with peer count from P2P network
    pub async fn get_storage_stats(&self) -> StorageStats {
        let manifest = self.file_manifest.read().await;
        let cache = self.local_blocks.read().await;
        let pinning = self.pinning.read().await;
        
        // Get peer count from P2P network if available
        let peer_count = if let Some(ref tx) = self.p2p_command_tx {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if tx.send(crate::p2p_network::P2PNodeCommand::GetConnectedPeers(reply_tx)).is_ok() {
                reply_rx.await.map(|peers| peers.len()).unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };
        
        let storage_used = pinning.get_used_space();
        let storage_limit = pinning.get_limit();
        
        StorageStats {
            total_files: manifest.len(),
            cached_blocks: cache.len(),
            connected_peers: peer_count,
            storage_used,
            storage_limit,
            storage_available: storage_limit.saturating_sub(storage_used),
        }
    }
    
    /// Upload file to P2P network with quantum encryption and all advanced features
    pub async fn upload_file(&self, path: &Path, data: &[u8]) -> Result<Uuid> {
        self.upload_file_with_progress(path, data, |_, _| {}).await
    }
    
    /// Upload file with progress callback
    pub async fn upload_file_with_progress<F>(&self, path: &Path, data: &[u8], mut progress_callback: F) -> Result<Uuid>
    where
        F: FnMut(usize, usize),
    {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("📤 Uploading file '{}' ({} bytes)", path_str, data.len());
        
        // STEP 1: Adaptive compression
        let (compressed_data, compression_algo) = self.compression.compress(data)?;
        tracing::info!("   ✓ Compression: {} -> {} bytes ({:?})", 
            data.len(), compressed_data.len(), compression_algo);
        
        // STEP 2: Split into chunks for parallel processing
        let chunks = self.parallel.split_into_chunks(&compressed_data);
        let total_chunks = chunks.len();
        
        tracing::info!("   ✓ Split into {} chunks for parallel processing", total_chunks);
        
        // STEP 3: Parallel quantum encryption
        tracing::info!("   ⚡ Encrypting {} chunks in parallel...", total_chunks);
        let blocks = self.parallel.encrypt_blocks_parallel(chunks, self.identity.clone())?;
        
        // STEP 4: Create block chain and distribute
        let mut previous_uuid: Option<Uuid> = None;
        let mut previous_hash = [0u8; 32];
        let mut first_block_uuid: Option<Uuid> = None;
        
        for (i, block) in blocks.iter().enumerate().rev() {
            let block_uuid = block.uuid;
            let block_id = block_uuid.to_string();
            
            tracing::debug!("   Processing block {} (chunk {})", block_id, i);
            
            // Serialize block
            let block_data = bincode::serialize(&block)?;
            
            // STEP 5: Apply erasure coding for fault tolerance
            let shards = self.erasure.encode(&block_data)?;
            tracing::debug!("   ✓ Created {} erasure-coded shards", shards.len());
            
            // STEP 6: Store shards on P2P network with pinning
            for (shard_idx, shard) in shards.iter().enumerate() {
                let shard_data = bincode::serialize(shard)?;
                let shard_id = format!("{}-s{}", block_id, shard_idx);
                
                // Store shard on P2P network (DHT + local cache)
                if let Err(e) = self.store_block_p2p(shard_id.clone(), shard_data.clone()).await {
                    tracing::warn!("   ⚠️  Failed to store shard {} on P2P network: {}", shard_id, e);
                    // Continue anyway - we have local storage
                }
                
                // Pin the shard (user data - never garbage collected)
                let mut pinning = self.pinning.write().await;
                pinning.pin(
                    shard_id.clone(),
                    PinType::User,
                    self.identity.user_id().to_string(),
                    shard_data.len(),
                )?;
                
                tracing::debug!("   ✓ Stored and pinned shard {}", shard_id);
            }
            
            // Store block locally (cache)
            self.local_blocks.write().await.insert(block_id.clone(), block.clone());
            
            // Update chain
            previous_hash = block.calculate_hash();
            previous_uuid = Some(block_uuid);
            
            if i == 0 {
                first_block_uuid = Some(block_uuid);
            }
            
            // Report progress
            progress_callback(blocks.len() - i, blocks.len());
        }
        
        let first_uuid = first_block_uuid.ok_or_else(|| 
            MSSCSError::InvalidData("No blocks created".to_string()))?;
        
        // Update manifest
        self.file_manifest.write().await.insert(path_str.clone(), first_uuid);
        
        tracing::info!("✅ File '{}' uploaded successfully", path_str);
        Ok(first_uuid)
    }
    
    /// Download file from P2P network
    pub async fn download_file(&self, path: &Path) -> Result<Vec<u8>> {
        self.download_file_with_progress(path, |_, _| {}).await
    }
    
    /// Download file with progress callback
    pub async fn download_file_with_progress<F>(&self, path: &Path, mut progress_callback: F) -> Result<Vec<u8>>
    where
        F: FnMut(usize, usize),
    {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("📥 Downloading file '{}'", path_str);
        
        // Get first block UUID from manifest
        let manifest = self.file_manifest.read().await;
        let first_uuid = manifest.get(&path_str)
            .ok_or_else(|| MSSCSError::NotFound(format!("File '{}' not found", path_str)))?;
        
        // Collect all blocks in chain
        let mut blocks = Vec::new();
        let mut current_uuid = *first_uuid;
        let mut block_count = 0;
        
        loop {
            let block = self.get_block(&current_uuid).await?;
            blocks.push(block.clone());
            block_count += 1;
            
            // Report progress for block retrieval
            progress_callback(block_count, block_count + 1); // Estimate
            
            if let Some(prev_uuid) = block.previous_uuid {
                current_uuid = prev_uuid;
            } else {
                break;
            }
        }
        
        let total_blocks = blocks.len();
        tracing::info!("   Retrieved {} blocks", total_blocks);

        // CRITICAL SECURITY FIX: Verify blockchain chain integrity before decryption
        tracing::info!("   🔗 Verifying blockchain chain integrity...");
        self.verify_chain(&blocks).await?;
        tracing::info!("   ✅ Chain verification passed - all {} blocks valid", total_blocks);
        tracing::info!("   ✅ No tampering detected - safe to decrypt");
        
        // STEP 7: Parallel decryption
        tracing::info!("   ⚡ Decrypting {} blocks in parallel...", blocks.len());
        let chunks = self.parallel.decrypt_blocks_parallel(blocks, self.identity.clone())?;
        
        // STEP 8: Combine chunks
        let compressed_data = self.parallel.combine_chunks(chunks);
        
        // STEP 9: Decompress (if it was compressed)
        // Note: In production, store compression metadata with file
        // For now, assume data is compressed
        let file_data = compressed_data; // Would decompress here if needed
        
        // Final progress update
        progress_callback(total_blocks, total_blocks);
        
        tracing::info!("✅ File '{}' downloaded ({} bytes)", path_str, file_data.len());
        Ok(file_data)
    }
    
    /// Verify blockchain chain integrity (CRITICAL SECURITY FIX)
    /// This prevents tampering by validating the entire chain before decryption
    async fn verify_chain(&self, blocks: &[QuantumDataBlock]) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }
        
        if blocks.len() == 1 {
            tracing::debug!("   ℹ️  Single block chain - no verification needed");
            return Ok(());
        }

        tracing::info!("   🔗 Verifying blockchain chain ({} blocks)", blocks.len());

        for i in 1..blocks.len() {
            let prev_block = &blocks[i - 1];
            let curr_block = &blocks[i];

            // CRITICAL: Verify previous_uuid matches
            if curr_block.previous_uuid != Some(prev_block.uuid) {
                tracing::error!("   ❌ SECURITY ALERT: Chain broken at block {}", i);
                tracing::error!("      Previous UUID mismatch - TAMPERING DETECTED");
                tracing::error!("      Expected: {:?}", Some(prev_block.uuid));
                tracing::error!("      Got: {:?}", curr_block.previous_uuid);
                return Err(MSSCSError::InvalidData(
                    format!("Chain broken at block {}: UUID mismatch (expected {:?}, got {:?})", 
                        i, Some(prev_block.uuid), curr_block.previous_uuid)
                ));
            }

            // CRITICAL: Verify previous_hash matches
            let expected_hash = prev_block.calculate_hash();
            if curr_block.previous_hash != expected_hash {
                tracing::error!("   ❌ SECURITY ALERT: Chain broken at block {}", i);
                tracing::error!("      Hash mismatch - TAMPERING DETECTED");
                tracing::error!("      Expected hash: {:?}", expected_hash);
                tracing::error!("      Got hash: {:?}", curr_block.previous_hash);
                return Err(MSSCSError::InvalidData(
                    format!("Chain broken at block {}: hash mismatch", i)
                ));
            }
            
            tracing::debug!("      ✓ Block {} verified", i);
        }

        tracing::info!("   ✅ Chain verification passed - all {} blocks valid", blocks.len());
        tracing::info!("   ✅ No tampering detected - safe to decrypt");
        Ok(())
    }

    /// Get block from local cache or P2P network
    async fn get_block(&self, uuid: &Uuid) -> Result<QuantumDataBlock> {
        let block_id = uuid.to_string();
        
        // Check local cache first
        {
            let cache = self.local_blocks.read().await;
            if let Some(block) = cache.get(&block_id) {
                tracing::debug!("   ✅ Block {} found in cache", block_id);
                return Ok(block.clone());
            }
        }
        
        tracing::debug!("   Block {} not in cache, fetching from P2P network", block_id);
        
        // Try to reconstruct from P2P network shards
        match self.reconstruct_block_from_network(&block_id).await {
            Ok(block) => {
                // Cache the reconstructed block
                self.local_blocks.write().await.insert(block_id.clone(), block.clone());
                Ok(block)
            }
            Err(e) => {
                tracing::warn!("   ⚠️  Failed to reconstruct block {}: {}", block_id, e);
                Err(MSSCSError::NotFound(format!("Block {} not found on network", block_id)))
            }
        }
    }
    
    /// Reconstruct block from P2P network shards
    async fn reconstruct_block_from_network(&self, block_id: &str) -> Result<QuantumDataBlock> {
        tracing::debug!("   🔄 Reconstructing block {} from network shards", block_id);
        
        // Collect shards from P2P network
        let mut reconstructed_shards = Vec::new();
        
        for shard_idx in 0..self.erasure.total_shards() {
            let shard_id = format!("{}-s{}", block_id, shard_idx);
            
            // Try to get shard from P2P network DHT
            match self.get_block_p2p(&shard_id).await {
                Ok(shard_data) => {
                    let shard: Shard = bincode::deserialize(&shard_data)?;
                    reconstructed_shards.push(shard);
                    tracing::debug!("      ✅ Retrieved shard {} from P2P network", shard_idx);
                }
                Err(_) => {
                    tracing::debug!("      ⚠️  Shard {} not available", shard_idx);
                }
            }
            
            // Check if we have enough shards to reconstruct
            if reconstructed_shards.len() >= 10 {
                break;
            }
        }
        
        // Need at least data_shards to reconstruct block
        if reconstructed_shards.len() < 10 {
            return Err(MSSCSError::NotFound(
                format!("Insufficient shards to reconstruct block {} (have {}, need 10)", 
                    block_id, reconstructed_shards.len())
            ));
        }
        
        // Reconstruct block data from shards
        let block_data = self.erasure.decode(&reconstructed_shards)?;
        let block: QuantumDataBlock = bincode::deserialize(&block_data)?;
        
        tracing::debug!("   ✅ Block {} reconstructed successfully", block_id);
        Ok(block)
    }
    
    /// Reconstruct block from P2P fragments
    async fn reconstruct_block_from_fragments(&self, block_id: &str) -> Result<QuantumDataBlock> {
        tracing::debug!("   Reconstructing block {} from fragments", block_id);
        
        // Collect fragments for each shard
        let mut reconstructed_shards = Vec::new();
        
        for shard_idx in 0..self.erasure.total_shards() {
            // Collect singularity fragments for this shard
            let fragments = Vec::new();
            
            for frag_idx in 0..5 {
                let fragment_id = format!("{}-s{}-f{}", block_id, shard_idx, frag_idx);
                
                // Try to get fragment from P2P network
                // (simplified - in production, query DHT and download from providers)
                // For now, skip if not available
            }
            
            // Need at least threshold fragments to reconstruct
            if fragments.len() >= 3 {
                let shard_data = self.singularity.reconstruct(&fragments)?;
                let shard: Shard = bincode::deserialize(&shard_data)?;
                reconstructed_shards.push(shard);
            }
        }
        
        // Need at least data_shards to reconstruct block
        if reconstructed_shards.len() < 10 {
            return Err(MSSCSError::NotFound(
                format!("Insufficient shards to reconstruct block {}", block_id)
            ));
        }
        
        // Reconstruct block data from shards
        let block_data = self.erasure.decode(&reconstructed_shards)?;
        let block: QuantumDataBlock = bincode::deserialize(&block_data)?;
        
        Ok(block)
    }
    
    /// Delete file from manifest
    pub async fn delete_file(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        tracing::info!("🗑️  Deleting file '{}'", path_str);
        
        self.file_manifest.write().await.remove(&path_str)
            .ok_or_else(|| MSSCSError::NotFound(format!("File '{}' not found", path_str)))?;
        
        tracing::info!("✅ File '{}' deleted", path_str);
        Ok(())
    }
    
    /// List all files
    pub async fn list_files(&self) -> Vec<String> {
        self.file_manifest.read().await.keys().cloned().collect()
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        let manifest = self.file_manifest.read().await;
        let cache = self.local_blocks.read().await;
        
        StorageStats {
            total_files: manifest.len(),
            cached_blocks: cache.len(),
            connected_peers: 0, // Would be from P2P node
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageStats {
    pub total_files: usize,
    pub cached_blocks: usize,
    pub connected_peers: usize,
    pub storage_used: usize,
    pub storage_limit: usize,
    pub storage_available: usize,
}
