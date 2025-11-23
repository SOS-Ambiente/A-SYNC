// Unlocked identity with decrypted secret keys
use crate::identity::QuantumIdentity;
use crate::error::Result;
use uuid::Uuid;

/// Unlocked identity with access to secret keys
pub struct UnlockedIdentity {
    pub identity: QuantumIdentity,
    pub(crate) ed25519_secret: Vec<u8>,
    pub(crate) kyber_secret: Vec<u8>,
    pub(crate) dilithium_secret: Vec<u8>,
}

impl UnlockedIdentity {
    /// Get user ID
    pub fn user_id(&self) -> &Uuid {
        &self.identity.id
    }

    /// Get user name
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    /// Sign data with Ed25519
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        use ed25519_dalek::{SigningKey, Signer};
        use crate::error::MSSCSError;

        let secret_bytes: [u8; 32] = self.ed25519_secret[..32].try_into()
            .map_err(|_| MSSCSError::Crypto("Invalid Ed25519 secret key length".to_string()))?;

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let signature = signing_key.sign(data);

        Ok(signature.to_bytes().to_vec())
    }

    /// Sign data with Dilithium (post-quantum)
    pub fn sign_dilithium(&self, data: &[u8]) -> Result<Vec<u8>> {
        use pqcrypto_dilithium::dilithium5;
        use pqcrypto_traits::sign::{SecretKey, DetachedSignature};
        use crate::error::MSSCSError;

        let secret_key = dilithium5::SecretKey::from_bytes(&self.dilithium_secret)
            .map_err(|_| MSSCSError::Crypto("Invalid Dilithium secret key".to_string()))?;

        let signature = dilithium5::detached_sign(data, &secret_key);

        Ok(signature.as_bytes().to_vec())
    }

    /// Get Kyber secret key for decryption
    pub fn kyber_secret_key(&self) -> Result<pqc_kyber::SecretKey> {
        use pqc_kyber::*;
        use crate::error::MSSCSError;

        let secret_bytes: [u8; KYBER_SECRETKEYBYTES] = self.kyber_secret.clone()
            .try_into()
            .map_err(|_| MSSCSError::Crypto("Invalid Kyber secret key length".to_string()))?;

        Ok(secret_bytes)
    }

    /// Get Kyber public key
    pub fn kyber_public_key(&self) -> Result<pqc_kyber::PublicKey> {
        use pqc_kyber::*;
        use crate::error::MSSCSError;

        let public_bytes: [u8; KYBER_PUBLICKEYBYTES] = self.identity.pq_public_key.clone()
            .try_into()
            .map_err(|_| MSSCSError::Crypto("Invalid Kyber public key length".to_string()))?;

        Ok(public_bytes)
    }
}
