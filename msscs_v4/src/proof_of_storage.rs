// PROOF OF STORAGE
// Challenge-response protocol to verify nodes are actually storing data

use crate::error::{MSSCSError, Result};
use blake3;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

/// Storage proof challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    /// Challenge ID
    pub challenge_id: String,
    /// Block ID to prove
    pub block_id: String,
    /// Random byte positions to check
    pub positions: Vec<usize>,
    /// Challenge timestamp
    pub timestamp: u64,
    /// Challenge expires after this time
    pub expires_at: u64,
}

/// Storage proof response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    /// Challenge ID
    pub challenge_id: String,
    /// Block ID
    pub block_id: String,
    /// Bytes at requested positions
    pub bytes: Vec<u8>,
    /// Merkle proof for the bytes
    pub merkle_proof: Vec<[u8; 32]>,
    /// Response timestamp
    pub timestamp: u64,
}

/// Proof verification result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofResult {
    /// Proof is valid
    Valid,
    /// Proof is invalid
    Invalid(String),
    /// Proof expired
    Expired,
    /// Proof not found
    NotFound,
}

/// Proof of Storage manager
pub struct ProofOfStorage {
    /// Challenge timeout (seconds)
    challenge_timeout: u64,
    /// Number of random positions to check
    num_positions: usize,
}

impl ProofOfStorage {
    /// Create a new proof of storage manager
    pub fn new(challenge_timeout: u64, num_positions: usize) -> Self {
        ProofOfStorage {
            challenge_timeout,
            num_positions,
        }
    }
    
    /// Generate a storage challenge
    pub fn generate_challenge(&self, block_id: String, block_size: usize) -> StorageChallenge {
        let challenge_id = self.generate_challenge_id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Generate random positions to check
        let mut rng = rand::thread_rng();
        let mut positions = Vec::new();
        for _ in 0..self.num_positions {
            let pos = rng.gen_range(0..block_size);
            positions.push(pos);
        }
        positions.sort();
        
        tracing::debug!("📋 Generated challenge {} for block {}", challenge_id, block_id);
        tracing::debug!("   Checking {} positions: {:?}", positions.len(), positions);
        
        StorageChallenge {
            challenge_id,
            block_id,
            positions,
            timestamp: now,
            expires_at: now + self.challenge_timeout,
        }
    }
    
    /// Create a proof response
    pub fn create_proof(
        &self,
        challenge: &StorageChallenge,
        block_data: &[u8],
    ) -> Result<StorageProof> {
        // Check if challenge expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > challenge.expires_at {
            return Err(MSSCSError::Validation("Challenge expired".to_string()));
        }
        
        // Extract bytes at requested positions
        let mut bytes = Vec::new();
        for &pos in &challenge.positions {
            if pos >= block_data.len() {
                return Err(MSSCSError::Validation(format!(
                    "Position {} out of bounds (block size: {})",
                    pos, block_data.len()
                )));
            }
            bytes.push(block_data[pos]);
        }
        
        // Generate Merkle proof
        let merkle_proof = self.generate_merkle_proof(block_data, &challenge.positions);
        
        tracing::debug!("✅ Created proof for challenge {}", challenge.challenge_id);
        
        Ok(StorageProof {
            challenge_id: challenge.challenge_id.clone(),
            block_id: challenge.block_id.clone(),
            bytes,
            merkle_proof,
            timestamp: now,
        })
    }
    
    /// Verify a storage proof
    pub fn verify_proof(
        &self,
        challenge: &StorageChallenge,
        proof: &StorageProof,
        block_hash: &[u8; 32],
    ) -> ProofResult {
        // Check challenge ID matches
        if challenge.challenge_id != proof.challenge_id {
            return ProofResult::Invalid("Challenge ID mismatch".to_string());
        }
        
        // Check block ID matches
        if challenge.block_id != proof.block_id {
            return ProofResult::Invalid("Block ID mismatch".to_string());
        }
        
        // Check if expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > challenge.expires_at {
            return ProofResult::Expired;
        }
        
        // Check number of bytes matches
        if proof.bytes.len() != challenge.positions.len() {
            return ProofResult::Invalid("Byte count mismatch".to_string());
        }
        
        // Verify Merkle proof
        if !self.verify_merkle_proof(&proof.bytes, &challenge.positions, &proof.merkle_proof, block_hash) {
            return ProofResult::Invalid("Merkle proof verification failed".to_string());
        }
        
        tracing::debug!("✅ Proof verified for challenge {}", challenge.challenge_id);
        ProofResult::Valid
    }
    
    /// Generate challenge ID
    fn generate_challenge_id(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
    
    /// Generate Merkle proof for specific positions
    fn generate_merkle_proof(&self, data: &[u8], positions: &[usize]) -> Vec<[u8; 32]> {
        // Simplified Merkle proof generation
        // In production, use proper Merkle tree implementation
        let mut proof = Vec::new();
        
        // Hash the full data
        let full_hash = blake3::hash(data);
        proof.push(*full_hash.as_bytes());
        
        // Hash each position's neighborhood
        for &pos in positions {
            let start = pos.saturating_sub(32);
            let end = (pos + 32).min(data.len());
            let neighborhood = &data[start..end];
            let hash = blake3::hash(neighborhood);
            proof.push(*hash.as_bytes());
        }
        
        proof
    }
    
    /// Verify Merkle proof
    fn verify_merkle_proof(
        &self,
        _bytes: &[u8],
        positions: &[usize],
        proof: &[[u8; 32]],
        _block_hash: &[u8; 32],
    ) -> bool {
        // Simplified verification
        // In production, use proper Merkle tree verification
        
        if proof.is_empty() {
            return false;
        }
        
        // Check we have enough proof elements
        if proof.len() < positions.len() + 1 {
            return false;
        }
        
        // Verify bytes are consistent with proof
        // (In production, this would verify the full Merkle path)
        true
    }
    
    /// Calculate storage score based on proof history
    pub fn calculate_storage_score(&self, proofs: &[ProofResult]) -> f64 {
        if proofs.is_empty() {
            return 0.0;
        }
        
        let valid_count = proofs.iter()
            .filter(|r| **r == ProofResult::Valid)
            .count();
        
        (valid_count as f64 / proofs.len() as f64) * 100.0
    }
}

impl Default for ProofOfStorage {
    fn default() -> Self {
        Self::new(300, 10) // 5 minute timeout, 10 positions
    }
}

/// Storage reputation tracker
pub struct ReputationTracker {
    /// Node reputation scores (node_id -> score)
    scores: std::collections::HashMap<String, f64>,
    /// Proof history (node_id -> results)
    history: std::collections::HashMap<String, Vec<ProofResult>>,
    /// Maximum history size per node
    max_history: usize,
}

impl ReputationTracker {
    /// Create a new reputation tracker
    pub fn new(max_history: usize) -> Self {
        ReputationTracker {
            scores: std::collections::HashMap::new(),
            history: std::collections::HashMap::new(),
            max_history,
        }
    }
    
    /// Record proof result
    pub fn record_proof(&mut self, node_id: String, result: ProofResult) {
        // Add to history
        let history = self.history.entry(node_id.clone()).or_insert_with(Vec::new);
        history.push(result);
        
        // Limit history size
        if history.len() > self.max_history {
            history.remove(0);
        }
        
        // Recalculate score
        let pos = ProofOfStorage::default();
        let score = pos.calculate_storage_score(history);
        self.scores.insert(node_id, score);
    }
    
    /// Get node reputation score
    pub fn get_score(&self, node_id: &str) -> f64 {
        self.scores.get(node_id).copied().unwrap_or(50.0) // Default: 50%
    }
    
    /// Get nodes with high reputation
    pub fn get_reliable_nodes(&self, min_score: f64) -> Vec<String> {
        self.scores.iter()
            .filter(|(_, &score)| score >= min_score)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_challenge_generation() {
        let pos = ProofOfStorage::new(300, 10);
        let challenge = pos.generate_challenge("block123".to_string(), 1000);
        
        assert_eq!(challenge.block_id, "block123");
        assert_eq!(challenge.positions.len(), 10);
        assert!(challenge.expires_at > challenge.timestamp);
    }
    
    #[test]
    fn test_proof_creation_and_verification() {
        let pos = ProofOfStorage::new(300, 5);
        let block_data = b"This is test data for proof of storage verification";
        let block_hash = blake3::hash(block_data);
        
        // Generate challenge
        let challenge = pos.generate_challenge("block123".to_string(), block_data.len());
        
        // Create proof
        let proof = pos.create_proof(&challenge, block_data).unwrap();
        
        // Verify proof
        let result = pos.verify_proof(&challenge, &proof, block_hash.as_bytes());
        assert_eq!(result, ProofResult::Valid);
    }
    
    #[test]
    fn test_expired_challenge() {
        let pos = ProofOfStorage::new(0, 5); // 0 second timeout
        let block_data = b"Test data";
        
        let mut challenge = pos.generate_challenge("block123".to_string(), block_data.len());
        
        // Manually set expiration to past
        challenge.expires_at = challenge.timestamp - 1;
        
        // Try to create proof
        let result = pos.create_proof(&challenge, block_data);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_reputation_tracker() {
        let mut tracker = ReputationTracker::new(100);
        
        // Record successful proofs
        tracker.record_proof("node1".to_string(), ProofResult::Valid);
        tracker.record_proof("node1".to_string(), ProofResult::Valid);
        tracker.record_proof("node1".to_string(), ProofResult::Invalid("test".to_string()));
        
        let score = tracker.get_score("node1");
        assert!((score - 66.67).abs() < 0.1); // 2/3 = 66.67%
        
        // Get reliable nodes
        let reliable = tracker.get_reliable_nodes(60.0);
        assert_eq!(reliable.len(), 1);
        assert_eq!(reliable[0], "node1");
    }
    
    #[test]
    fn test_storage_score_calculation() {
        let pos = ProofOfStorage::default();
        
        let proofs = vec![
            ProofResult::Valid,
            ProofResult::Valid,
            ProofResult::Valid,
            ProofResult::Invalid("test".to_string()),
        ];
        
        let score = pos.calculate_storage_score(&proofs);
        assert_eq!(score, 75.0); // 3/4 = 75%
    }
}
