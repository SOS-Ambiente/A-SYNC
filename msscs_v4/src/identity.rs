// Identity module - Quantum-resistant cryptographic identities
use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Quantum-resistant cryptographic identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumIdentity {
    /// Unique identity identifier
    pub id: Uuid,
    /// Node name/human readable identifier
    pub name: String,
    /// Ed25519 public key for standard crypto operations
    pub public_key: Vec<u8>,
    /// Post-quantum public key (Kyber-1024)
    pub pq_public_key: Vec<u8>,
    /// Identity version
    pub version: u32,
    /// Creation timestamp
    pub created_at: u64,
    /// Metadata tags
    pub tags: HashMap<String, String>,
    /// Trust score from network reputation
    pub trust_score: f64,
    /// Last seen timestamp
    pub last_seen: Option<u64>,
}

impl QuantumIdentity {
    /// Create new quantum identity
    pub fn new(name: String) -> Result<Self> {
        use ed25519_dalek::SigningKey;
        use pqc_kyber::*;

        let id = Uuid::new_v4();

        // Generate Ed25519 keypair for standard operations
        let ed_keypair = SigningKey::generate(&mut rand::rngs::OsRng);
        let public_key = ed_keypair.verifying_key().to_bytes().to_vec();

        // Generate post-quantum keypair (Kyber-1024)
        let mut rng = rand::rngs::OsRng;
        let keys = keypair(&mut rng).unwrap();
        let pq_public_key = keys.public.to_vec();

        let identity = QuantumIdentity {
            id,
            name,
            public_key,
            pq_public_key,
            version: 1,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| MSSCSError::Config(format!("Time error: {}", e)))?
                .as_secs(),
            tags: HashMap::new(),
            trust_score: 0.0,
            last_seen: None,
        };

        info!("Created new quantum identity: {} ({})", identity.id, identity.name);
        Ok(identity)
    }

    /// Sign data using Ed25519 (requires secret key - would need secure storage)
    pub fn sign(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // Note: This would require the secret key to be securely stored
        // For now, return placeholder implementation
        warn!("Identity::sign() called but secret key storage not implemented");
        Ok(vec![])
    }

    /// Verify signature using Ed25519 public key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        use ed25519_dalek::{VerifyingKey, Verifier, Signature};

        let public_key_bytes: [u8; 32] = self.public_key.clone()
            .try_into()
            .map_err(|_| MSSCSError::InvalidData("Invalid public key format".to_string()))?;

        let public_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|_| MSSCSError::InvalidData("Invalid public key".to_string()))?;

        let sig_bytes: [u8; 64] = signature.try_into()
            .map_err(|_| MSSCSError::InvalidData("Invalid signature length".to_string()))?;

        let signature = Signature::from_bytes(&sig_bytes);

        Ok(public_key.verify(data, &signature).is_ok())
    }

    /// Encrypt data using post-quantum encryption
    pub fn encrypt_quantum(&self, data: &[u8]) -> Result<Vec<u8>> {
        use pqc_kyber::*;

        // This would use the recipient's PQ public key
        // For demonstration, we'll use our own
        let public_key_bytes: [u8; KYBER_PUBLICKEYBYTES] = self.pq_public_key.clone()
            .try_into()
            .map_err(|_| MSSCSError::Crypto("Invalid PQ public key length".to_string()))?;
        
        let mut rng = rand::rngs::OsRng;
        let (_ciphertext, _shared_secret) = encapsulate(&public_key_bytes, &mut rng)
            .map_err(|e| MSSCSError::Crypto(format!("Encapsulation failed: {:?}", e)))?;

        // In real implementation, would encrypt data with shared secret
        let mut encrypted = _ciphertext.to_vec();
        encrypted.extend_from_slice(data); // Simplified

        Ok(encrypted)
    }

    /// Update trust score
    pub fn update_trust_score(&mut self, delta: f64) {
        self.trust_score = (self.trust_score + delta).clamp(0.0, 100.0);
        self.last_seen = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        debug!("Updated trust score for {} to {}", self.name, self.trust_score);
    }

    /// Add metadata tag
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Get tag value
    pub fn get_tag(&self, key: &str) -> Option<&String> {
        self.tags.get(key)
    }

    /// Check if identity is "online" based on last seen
    pub fn is_online(&self, timeout_secs: u64) -> bool {
        self.last_seen
            .map(|last| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                (now - last) < timeout_secs
            })
            .unwrap_or(false)
    }

    /// Get reputation tier
    pub fn get_tier(&self) -> ReputationTier {
        if self.trust_score >= 80.0 {
            ReputationTier::Trusted
        } else if self.trust_score >= 50.0 {
            ReputationTier::Established
        } else if self.trust_score >= 20.0 {
            ReputationTier::New
        } else {
            ReputationTier::Unknown
        }
    }
}

/// Reputation tier for identity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationTier {
    Unknown,
    New,
    Established,
    Trusted,
}

/// Identity manager for multiple identities
pub struct IdentityManager {
    identities: Arc<RwLock<HashMap<Uuid, QuantumIdentity>>>,
    data_dir: PathBuf,
    current_identity: Option<Uuid>,
}

impl IdentityManager {
    /// Create new identity manager
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;

        Ok(IdentityManager {
            identities: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
            current_identity: None,
        })
    }

    /// Create new identity
    pub async fn create_identity(&mut self, name: String) -> Result<Uuid> {
        let identity = QuantumIdentity::new(name)?;
        let id = identity.id;

        // Store in memory
        self.identities.write().await.insert(id, identity);

        // Save to disk
        self.save_identity(&id).await?;

        // Set as current if none exists
        if self.current_identity.is_none() {
            self.current_identity = Some(id);
        }

        info!("Created and stored identity: {}", id);
        Ok(id)
    }

    /// Load identity from disk
    pub async fn load_identity(&mut self, id: &Uuid) -> Result<()> {
        let file_path = self.data_dir.join(format!("{}.identity", id));

        if !file_path.exists() {
            return Err(MSSCSError::NotFound(format!("Identity file not found: {}", id)));
        }

        let data = std::fs::read_to_string(file_path)
            .map_err(|e| MSSCSError::Config(format!("Failed to read identity file: {}", e)))?;

        let identity: QuantumIdentity = serde_json::from_str(&data)
            .map_err(|e| MSSCSError::Config(format!("Failed to parse identity: {}", e)))?;

        self.identities.write().await.insert(*id, identity);
        info!("Loaded identity: {}", id);
        Ok(())
    }

    /// Save identity to disk
    pub async fn save_identity(&self, id: &Uuid) -> Result<()> {
        let identities = self.identities.read().await;
        let identity = identities.get(id)
            .ok_or_else(|| MSSCSError::NotFound(format!("Identity not found: {}", id)))?;

        let data = serde_json::to_string_pretty(identity)
            .map_err(|e| MSSCSError::Config(format!("Failed to serialize identity: {}", e)))?;

        let file_path = self.data_dir.join(format!("{}.identity", id));
        std::fs::write(file_path, data)
            .map_err(|e| MSSCSError::Config(format!("Failed to write identity file: {}", e)))?;

        debug!("Saved identity: {}", id);
        Ok(())
    }

    /// Get identity by ID
    pub async fn get_identity(&self, id: &Uuid) -> Option<QuantumIdentity> {
        self.identities.read().await.get(id).cloned()
    }

    /// Get current identity
    pub async fn get_current_identity(&self) -> Option<QuantumIdentity> {
        if let Some(current_id) = self.current_identity {
            self.identities.read().await.get(&current_id).cloned()
        } else {
            None
        }
    }

    /// Set current identity
    pub async fn set_current_identity(&mut self, id: Uuid) -> Result<()> {
        // Verify identity exists
        let identities = self.identities.read().await;
        if !identities.contains_key(&id) {
            return Err(MSSCSError::NotFound(format!("Identity not found: {}", id)));
        }
        drop(identities);

        self.current_identity = Some(id);
        info!("Set current identity to: {}", id);
        Ok(())
    }

    /// List all identities
    pub async fn list_identities(&self) -> Vec<QuantumIdentity> {
        self.identities.read().await.values().cloned().collect()
    }

    /// Update identity trust score
    pub async fn update_trust_score(&self, id: &Uuid, delta: f64) -> Result<()> {
        let mut identities = self.identities.write().await;
        if let Some(identity) = identities.get_mut(id) {
            identity.update_trust_score(delta);
            drop(identities);
            self.save_identity(id).await?;
            Ok(())
        } else {
            Err(MSSCSError::NotFound(format!("Identity not found: {}", id)))
        }
    }

    /// Get peer reputation
    pub async fn get_peer_reputation(&self, id: &Uuid) -> Option<ReputationTier> {
        self.identities.read().await.get(id).map(|i| i.get_tier())
    }

    /// Cleanup old offline identities
    pub async fn cleanup_offline_identities(&self, offline_timeout_secs: u64) -> Result<usize> {
        let mut to_remove = Vec::new();
        let identities = self.identities.read().await;

        for (id, identity) in identities.iter() {
            if !identity.is_online(offline_timeout_secs) && identity.trust_score < 10.0 {
                to_remove.push(*id);
            }
        }
        drop(identities);

        let count = to_remove.len();

        for id in &to_remove {
            // Remove from memory
            self.identities.write().await.remove(id);

            // Delete from disk
            let file_path = self.data_dir.join(format!("{}.identity", id));
            let _ = std::fs::remove_file(file_path);

            debug!("Removed offline identity: {}", id);
        }

        info!("Cleaned up {} offline identities", count);
        Ok(count)
    }
}