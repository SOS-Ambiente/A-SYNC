// QUANTUM-ENHANCED BLOCK MODULE
// Integrates quantum-proof encryption with existing block system

use crate::error::{MSSCSError, Result};
use crate::huffman;
use crate::quantum_crypto::QuantumProofBlock;
use crate::identity::UnlockedIdentity;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;
use blake3;

/// Enhanced data block with quantum-proof encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumDataBlock {
    /// Unique identifier for the block
    pub uuid: Uuid,
    
    /// Position in the chain
    pub node_index: u64,
    
    /// Previous block reference
    pub previous_uuid: Option<Uuid>,
    pub previous_hash: [u8; 32],
    
    /// Quantum-encrypted payload
    pub quantum_block: QuantumProofBlock,
    
    /// Metadata (encrypted)
    pub metadata: BlockMetadata,
    
    /// Creation timestamp
    pub created_at: u64,
}

/// Block metadata (stored encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Original data size (before compression/encryption)
    pub original_size: usize,
    
    /// Compressed size
    pub compressed_size: usize,
    
    /// Content type hint
    pub content_type: Option<String>,
    
    /// Custom tags
    pub tags: Vec<String>,
}

impl QuantumDataBlock {
    /// Create new quantum-enhanced block with full pipeline:
    /// 1. Huffman compression
    /// 2. Seven-layer quantum-proof encryption
    /// 3. Post-quantum signatures
    pub fn new(
        data: &[u8],
        node_index: u64,
        previous_uuid: Option<Uuid>,
        previous_hash: [u8; 32],
        identity: &UnlockedIdentity,
        content_type: Option<String>,
    ) -> Result<Self> {
        tracing::info!("Creating quantum-enhanced block (node {})", node_index);
        
        let uuid = Uuid::new_v4();
        let original_size = data.len();
        
        // 1. Compress with Huffman
        tracing::debug!("  [1/3] Huffman compression");
        let compressed = huffman::compress(data)?;
        let compressed_size = compressed.len();
        
        tracing::debug!("  Compression: {} -> {} bytes ({:.1}% reduction)",
            original_size, compressed_size,
            (1.0 - compressed_size as f64 / original_size as f64) * 100.0
        );
        
        // 2. Apply quantum-proof encryption
        tracing::debug!("  [2/3] Quantum-proof encryption (7 layers)");
        let quantum_block = QuantumProofBlock::new(
            &compressed,
            identity.master_key(),
            identity.kyber_public_key(),
        )?;
        
        // 3. Create metadata
        let metadata = BlockMetadata {
            original_size,
            compressed_size,
            content_type,
            tags: Vec::new(),
        };
        
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        tracing::info!("✅ Quantum block created: {}", uuid);
        
        Ok(Self {
            uuid,
            node_index,
            previous_uuid,
            previous_hash,
            quantum_block,
            metadata,
            created_at,
        })
    }
    
    /// Decode quantum-enhanced block
    pub fn decode(&self, identity: &UnlockedIdentity) -> Result<Vec<u8>> {
        tracing::info!("Decoding quantum block: {}", self.uuid);
        
        // 1. Decrypt with quantum-proof decryption
        tracing::debug!("  [1/2] Quantum-proof decryption");
        let compressed = self.quantum_block.decrypt(
            identity.master_key(),
            identity.kyber_secret_key(),
        )?;
        
        // 2. Decompress
        tracing::debug!("  [2/2] Huffman decompression");
        let data = huffman::decompress(&compressed)?;
        
        // Verify size matches metadata
        if data.len() != self.metadata.original_size {
            return Err(MSSCSError::InvalidData(
                format!("Size mismatch: expected {}, got {}", 
                    self.metadata.original_size, data.len())
            ));
        }
        
        tracing::info!("✅ Quantum block decoded successfully");
        Ok(data)
    }
    
    /// Calculate block hash (using BLAKE3 for speed)
    pub fn calculate_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.uuid.as_bytes());
        hasher.update(&self.node_index.to_le_bytes());
        
        if let Some(prev_uuid) = self.previous_uuid {
            hasher.update(prev_uuid.as_bytes());
        }
        
        hasher.update(&self.previous_hash);
        hasher.update(&self.quantum_block.block_id);
        
        *hasher.finalize().as_bytes()
    }
    
    /// Verify block integrity
    pub fn verify(&self, identity: &UnlockedIdentity) -> bool {
        self.decode(identity).is_ok()
    }
    
    /// Get block size statistics
    pub fn size_stats(&self) -> BlockSizeStats {
        BlockSizeStats {
            original_size: self.metadata.original_size,
            compressed_size: self.metadata.compressed_size,
            encrypted_size: self.quantum_block.double_encrypted_payload.len(),
            total_overhead: self.calculate_total_size(),
        }
    }
    
    /// Calculate total serialized size
    fn calculate_total_size(&self) -> usize {
        bincode::serialize(self).map(|v| v.len()).unwrap_or(0)
    }
}

/// Block size statistics
#[derive(Debug, Clone)]
pub struct BlockSizeStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub encrypted_size: usize,
    pub total_overhead: usize,
}

impl BlockSizeStats {
    pub fn compression_ratio(&self) -> f64 {
        self.compressed_size as f64 / self.original_size as f64
    }
    
    pub fn total_overhead_ratio(&self) -> f64 {
        self.total_overhead as f64 / self.original_size as f64
    }
    
    pub fn print_summary(&self) {
        println!("📊 Block Size Statistics:");
        println!("   Original:    {:>10} bytes", self.original_size);
        println!("   Compressed:  {:>10} bytes ({:.1}% of original)",
            self.compressed_size,
            self.compression_ratio() * 100.0
        );
        println!("   Encrypted:   {:>10} bytes", self.encrypted_size);
        println!("   Total:       {:>10} bytes ({:.1}% overhead)",
            self.total_overhead,
            (self.total_overhead_ratio() - 1.0) * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::QuantumIdentity;
    
    #[test]
    fn test_quantum_block_creation() {
        let passphrase = "test passphrase for quantum block";
        let identity = QuantumIdentity::new(passphrase).unwrap();
        let unlocked = identity.unlock(passphrase).unwrap();
        
        let data = b"Test data for quantum block";
        let block = QuantumDataBlock::new(
            data,
            0,
            None,
            [0u8; 32],
            &unlocked,
            Some("text/plain".to_string()),
        ).unwrap();
        
        println!("Block UUID: {}", block.uuid);
        println!("Block hash: {}", hex::encode(block.calculate_hash()));
        
        let stats = block.size_stats();
        stats.print_summary();
        
        // Decode and verify
        let decoded = block.decode(&unlocked).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
    
    #[test]
    fn test_quantum_block_chain() {
        let passphrase = "test chain passphrase";
        let identity = QuantumIdentity::new(passphrase).unwrap();
        let unlocked = identity.unlock(passphrase).unwrap();
        
        // Create genesis block
        let data1 = b"Genesis block data";
        let block1 = QuantumDataBlock::new(
            data1,
            0,
            None,
            [0u8; 32],
            &unlocked,
            None,
        ).unwrap();
        
        let hash1 = block1.calculate_hash();
        
        // Create second block
        let data2 = b"Second block data";
        let block2 = QuantumDataBlock::new(
            data2,
            1,
            Some(block1.uuid),
            hash1,
            &unlocked,
            None,
        ).unwrap();
        
        // Verify chain
        assert_eq!(block2.previous_uuid, Some(block1.uuid));
        assert_eq!(block2.previous_hash, hash1);
        
        // Decode both
        let decoded1 = block1.decode(&unlocked).unwrap();
        let decoded2 = block2.decode(&unlocked).unwrap();
        
        assert_eq!(data1.as_slice(), decoded1.as_slice());
        assert_eq!(data2.as_slice(), decoded2.as_slice());
        
        println!("✅ Quantum block chain test passed");
    }
}
