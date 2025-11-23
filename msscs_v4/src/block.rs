use crate::error::{MSSCSError, Result};
use crate::huffman;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const NONCE_SIZE: usize = 12; // AES-GCM standard nonce size
const KEY_SIZE: usize = 32; // 256 bits for AES-256

/// Represents a data block in the MSSCS system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataBlock {
    /// Unique identifier for the block
    pub uuid: Uuid,
    /// Position of the block in the chain (essential for key derivation)
    pub node_index: u64,
    /// UUID of the previous block (None for genesis block)
    pub previous_uuid: Option<Uuid>,
    /// Hash SHA-256 of the previous block (used for Merkle tree)
    pub previous_hash: [u8; 32],
    /// Nonce used for encryption (must be stored for decryption)
    pub nonce: [u8; NONCE_SIZE],
    /// Final payload: encoded, compressed, and encrypted data
    encrypted_payload: Vec<u8>,
}

impl DataBlock {
    /// Creates a new data block with full processing pipeline:
    /// 1. Base-16 encoding (multi-state)
    /// 2. Huffman compression
    /// 3. AES-256-GCM encryption with derived key
    pub fn new(
        data: &[u8],
        node_index: u64,
        previous_uuid: Option<Uuid>,
        previous_hash: [u8; 32],
    ) -> Result<Self> {
        // 1. Generate UUID and Nonce
        let uuid = Uuid::new_v4();
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 2. Encode to Base-16 (multi-state logic)
        let mut encoded_payload = Vec::with_capacity(data.len() * 2);
        for &byte in data {
            encoded_payload.push(byte >> 4); // High nibble
            encoded_payload.push(byte & 0x0F); // Low nibble
        }

        // 3. Compress with Huffman
        let compressed_payload = huffman::compress(&encoded_payload)?;

        // 4. Derive encryption key from UUID + node_index
        let key = Self::derive_key(&uuid, node_index);
        let cipher = Aes256Gcm::new(&key.into());

        // 5. Encrypt with AES-256-GCM
        let encrypted_payload = cipher
            .encrypt(nonce, compressed_payload.as_ref())
            .map_err(|e| MSSCSError::Crypto(format!("Encryption failed: {}", e)))?;

        Ok(Self {
            uuid,
            node_index,
            previous_uuid,
            previous_hash,
            nonce: nonce_bytes,
            encrypted_payload,
        })
    }

    /// Calculates the SHA-256 hash of this block
    pub fn calculate_hash(&self) -> Result<[u8; 32]> {
        // Serialize relevant data for hashing
        let hashable_data = (
            self.uuid,
            self.node_index,
            self.previous_uuid,
            &self.nonce,
            &self.encrypted_payload,
        );
        let serialized_data = bincode::serialize(&hashable_data)?;
        
        let mut hasher = Sha256::new();
        hasher.update(serialized_data);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Decodes (decrypts, decompresses, and decodes) the block data
    /// Requires the correct node_index to derive the decryption key
    pub fn decode(&self, node_index: u64) -> Result<Vec<u8>> {
        // 1. Verify node_index matches
        if node_index != self.node_index {
            return Err(MSSCSError::Crypto(
                "Invalid node_index for decryption".to_string(),
            ));
        }

        // 2. Derive decryption key
        let key = Self::derive_key(&self.uuid, node_index);
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&self.nonce);

        // 3. Decrypt
        let compressed_payload = cipher
            .decrypt(nonce, self.encrypted_payload.as_ref())
            .map_err(|e| MSSCSError::Crypto(format!("Decryption failed: {}", e)))?;

        // 4. Decompress
        let encoded_payload = huffman::decompress(&compressed_payload)?;

        // 5. Decode from Base-16
        if encoded_payload.len() % 2 != 0 {
            return Err(MSSCSError::InvalidData(
                "Invalid Base-16 encoded data length".to_string(),
            ));
        }

        let mut decoded_data = Vec::with_capacity(encoded_payload.len() / 2);
        for chunk in encoded_payload.chunks(2) {
            let high = chunk[0];
            let low = chunk[1];
            decoded_data.push((high << 4) | low);
        }

        Ok(decoded_data)
    }

    /// Derives a 256-bit encryption key from UUID and node_index
    fn derive_key(uuid: &Uuid, node_index: u64) -> [u8; KEY_SIZE] {
        let mut hasher = Sha256::new();
        hasher.update(uuid.as_bytes());
        hasher.update(node_index.to_le_bytes());
        
        let result = hasher.finalize();
        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&result);
        key
    }

    /// Verifies the integrity of the block
    pub fn verify(&self) -> bool {
        // Try to decode - if successful, block is valid
        self.decode(self.node_index).is_ok()
    }

    /// Get the encrypted payload size
    pub fn get_encrypted_size(&self) -> usize {
        self.encrypted_payload.len()
    }

    /// Get the encrypted payload (for network transmission)
    pub fn get_encrypted_payload(&self) -> &[u8] {
        &self.encrypted_payload
    }
}

/// Calculate checksum for data
pub fn calculate_checksum(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
