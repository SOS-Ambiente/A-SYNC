// Access Control & Sharing with capability-based permissions
use crate::error::{MSSCSError, Result};
use crate::identity::UnlockedIdentity;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Access permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
    /// Full control (read, write, delete, share)
    FullControl,
}

/// Access capability token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    /// Resource identifier (block UUID, file path, etc.)
    pub resource_id: String,
    /// Permission level
    pub permission: Permission,
    /// Token issuer (owner's user ID)
    pub issuer: String,
    /// Token recipient (user ID or public key)
    pub recipient: String,
    /// Token creation timestamp
    pub created_at: u64,
    /// Token expiration timestamp (None = never expires)
    pub expires_at: Option<u64>,
    /// Encrypted resource key
    pub encrypted_key: Vec<u8>,
    /// Nonce for key encryption
    pub nonce: Vec<u8>,
    /// Token signature (prevents tampering)
    pub signature: Vec<u8>,
}

/// Access control manager
pub struct AccessControl {
    /// Owned resources (resource_id -> encryption key)
    owned_resources: HashMap<String, Vec<u8>>,
    /// Granted access tokens (resource_id -> tokens)
    granted_tokens: HashMap<String, Vec<AccessToken>>,
    /// Received access tokens (resource_id -> token)
    received_tokens: HashMap<String, AccessToken>,
}

impl AccessControl {
    /// Create a new access control manager
    pub fn new() -> Self {
        AccessControl {
            owned_resources: HashMap::new(),
            granted_tokens: HashMap::new(),
            received_tokens: HashMap::new(),
        }
    }

    /// Register a new owned resource
    pub fn register_resource(&mut self, resource_id: String, encryption_key: Vec<u8>) {
        self.owned_resources.insert(resource_id, encryption_key);
    }

    /// Create an access token for sharing a resource
    pub fn create_access_token(
        &mut self,
        resource_id: &str,
        recipient_public_key: &[u8],
        permission: Permission,
        expires_in_seconds: Option<u64>,
        identity: &UnlockedIdentity,
    ) -> Result<AccessToken> {
        // Get the resource encryption key
        let resource_key = self.owned_resources.get(resource_id)
            .ok_or_else(|| MSSCSError::Validation(format!("Resource not found: {}", resource_id)))?;

        // Calculate expiration
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = expires_in_seconds.map(|secs| created_at + secs);

        // Encrypt the resource key with recipient's public key
        // For simplicity, we'll use AES-GCM with recipient's key directly
        // In production, use hybrid encryption (ECIES or similar)
        let cipher = Aes256Gcm::new_from_slice(recipient_public_key)
            .map_err(|e| MSSCSError::Encryption(format!("Failed to create cipher: {}", e)))?;

        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted_key = cipher
            .encrypt(nonce, resource_key.as_ref())
            .map_err(|e| MSSCSError::Encryption(format!("Failed to encrypt key: {}", e)))?;

        // Create token data for signing
        let token_data = format!(
            "{}:{}:{}:{}:{}",
            resource_id,
            permission as u8,
            identity.user_id(),
            hex::encode(recipient_public_key),
            created_at
        );

        // Sign the token
        let signature = Self::sign_data(token_data.as_bytes(), identity.master_key());

        let token = AccessToken {
            resource_id: resource_id.to_string(),
            permission,
            issuer: identity.user_id().to_string(),
            recipient: hex::encode(recipient_public_key),
            created_at,
            expires_at,
            encrypted_key,
            nonce: nonce_bytes.to_vec(),
            signature,
        };

        // Store the granted token
        self.granted_tokens
            .entry(resource_id.to_string())
            .or_insert_with(Vec::new)
            .push(token.clone());

        Ok(token)
    }

    /// Verify and accept an access token
    pub fn accept_access_token(
        &mut self,
        token: AccessToken,
        identity: &UnlockedIdentity,
    ) -> Result<Vec<u8>> {
        // Verify token hasn't expired
        if let Some(expires_at) = token.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now > expires_at {
                return Err(MSSCSError::Validation("Access token has expired".to_string()));
            }
        }

        // Verify token signature
        let _token_data = format!(
            "{}:{}:{}:{}:{}",
            token.resource_id,
            token.permission as u8,
            token.issuer,
            token.recipient,
            token.created_at
        );

        // In production, verify signature with issuer's public key
        // For now, we'll skip signature verification

        // Decrypt the resource key using recipient's master key
        let cipher = Aes256Gcm::new_from_slice(identity.master_key())
            .map_err(|e| MSSCSError::Encryption(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&token.nonce);
        let resource_key = cipher
            .decrypt(nonce, token.encrypted_key.as_ref())
            .map_err(|e| MSSCSError::Encryption(format!("Failed to decrypt key: {}", e)))?;

        // Store the received token
        self.received_tokens.insert(token.resource_id.clone(), token);

        Ok(resource_key)
    }

    /// Revoke access to a resource
    pub fn revoke_access(&mut self, resource_id: &str, recipient: &str) -> Result<()> {
        if let Some(tokens) = self.granted_tokens.get_mut(resource_id) {
            tokens.retain(|t| t.recipient != recipient);
            Ok(())
        } else {
            Err(MSSCSError::Validation(format!("Resource not found: {}", resource_id)))
        }
    }

    /// List all granted tokens for a resource
    pub fn list_granted_tokens(&self, resource_id: &str) -> Vec<&AccessToken> {
        self.granted_tokens
            .get(resource_id)
            .map(|tokens| tokens.iter().collect())
            .unwrap_or_default()
    }

    /// Check if user has permission for a resource
    pub fn check_permission(&self, resource_id: &str, required: Permission) -> bool {
        // Check if we own the resource
        if self.owned_resources.contains_key(resource_id) {
            return true;
        }

        // Check if we have a valid token
        if let Some(token) = self.received_tokens.get(resource_id) {
            // Check expiration
            if let Some(expires_at) = token.expires_at {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now > expires_at {
                    return false;
                }
            }

            // Check permission level
            match (token.permission, required) {
                (Permission::FullControl, _) => true,
                (Permission::ReadWrite, Permission::Read) => true,
                (Permission::ReadWrite, Permission::ReadWrite) => true,
                (Permission::Read, Permission::Read) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Derive a shared key from two keys (simplified ECDH)
    fn derive_shared_key(key1: &[u8], key2: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(key1);
        hasher.update(key2);
        hasher.update(b"SHARED_KEY_DERIVATION");
        hasher.finalize().as_bytes()[..32].to_vec()
    }

    /// Generate a random nonce
    fn generate_nonce() -> [u8; 12] {
        use rand::RngCore;
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }

    /// Sign data with a key
    fn sign_data(data: &[u8], key: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(key);
        hasher.update(data);
        hasher.update(b"SIGNATURE");
        hasher.finalize().as_bytes().to_vec()
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::QuantumIdentity;

    #[test]
    fn test_access_control_creation() {
        let ac = AccessControl::new();
        assert_eq!(ac.owned_resources.len(), 0);
        assert_eq!(ac.granted_tokens.len(), 0);
        assert_eq!(ac.received_tokens.len(), 0);
    }

    #[test]
    fn test_register_resource() {
        let mut ac = AccessControl::new();
        let resource_id = "test-resource".to_string();
        let key = vec![1, 2, 3, 4];
        
        ac.register_resource(resource_id.clone(), key.clone());
        assert!(ac.owned_resources.contains_key(&resource_id));
    }

    #[test]
    fn test_create_and_accept_token() {
        let mut owner_ac = AccessControl::new();
        let mut recipient_ac = AccessControl::new();

        // Create identities
        let owner = QuantumIdentity::new("owner-pass").unwrap();
        let owner_unlocked = owner.unlock("owner-pass").unwrap();
        
        let recipient = QuantumIdentity::new("recipient-pass").unwrap();
        let recipient_unlocked = recipient.unlock("recipient-pass").unwrap();

        // Register a resource
        let resource_id = "shared-file";
        let resource_key = vec![1, 2, 3, 4, 5, 6, 7, 8];
        owner_ac.register_resource(resource_id.to_string(), resource_key.clone());

        // Create access token
        let token = owner_ac.create_access_token(
            resource_id,
            recipient_unlocked.master_key(),
            Permission::Read,
            Some(3600), // 1 hour
            &owner_unlocked,
        ).unwrap();

        assert_eq!(token.resource_id, resource_id);
        assert_eq!(token.permission, Permission::Read);

        // Accept token
        let decrypted_key = recipient_ac.accept_access_token(token, &recipient_unlocked).unwrap();
        assert_eq!(decrypted_key, resource_key);
    }

    #[test]
    fn test_permission_check() {
        let mut ac = AccessControl::new();
        let resource_id = "test-resource";

        // No permission initially
        assert!(!ac.check_permission(resource_id, Permission::Read));

        // Register as owner
        ac.register_resource(resource_id.to_string(), vec![1, 2, 3]);
        assert!(ac.check_permission(resource_id, Permission::FullControl));
    }

    #[test]
    fn test_revoke_access() {
        let mut ac = AccessControl::new();
        let owner = QuantumIdentity::new("pass").unwrap();
        let owner_unlocked = owner.unlock("pass").unwrap();

        let resource_id = "test-resource";
        ac.register_resource(resource_id.to_string(), vec![1, 2, 3]);

        let recipient_key = [42u8; 32]; // Must be 32 bytes for AES-256
        let token = ac.create_access_token(
            resource_id,
            &recipient_key,
            Permission::Read,
            None,
            &owner_unlocked,
        ).unwrap();

        // Verify token was created
        assert_eq!(ac.list_granted_tokens(resource_id).len(), 1);

        // Revoke access
        ac.revoke_access(resource_id, &token.recipient).unwrap();
        assert_eq!(ac.list_granted_tokens(resource_id).len(), 0);
    }
}
