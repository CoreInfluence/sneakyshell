//! Reticulum identity management

use crate::{DestinationHash, NetworkError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// A Reticulum identity (Ed25519 keypair)
#[derive(Clone)]
pub struct Identity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Identity {
    /// Generate a new random identity
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create identity from existing private key bytes
    pub fn from_bytes(private_key: &[u8]) -> Result<Self> {
        if private_key.len() != 32 {
            return Err(NetworkError::Identity(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(private_key);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Get the public key bytes
    pub fn public_key(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    /// Get the private key bytes
    pub fn private_key(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    /// Get the destination hash (SHA-256 of public key)
    pub fn destination_hash(&self) -> DestinationHash {
        let mut hasher = Sha256::new();
        hasher.update(self.verifying_key.to_bytes());
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Get destination hash as hex string
    pub fn destination_hex(&self) -> String {
        hex::encode(self.destination_hash())
    }

    /// Sign data
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(data);
        signature.to_bytes().to_vec()
    }

    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        if signature.len() != 64 {
            return Err(NetworkError::Crypto(
                "Signature must be 64 bytes".to_string(),
            ));
        }

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let sig = Signature::from_bytes(&sig_bytes);

        self.verifying_key
            .verify(data, &sig)
            .map_err(|e| NetworkError::Crypto(format!("Signature verification failed: {}", e)))
    }

    /// Save identity to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let private_key = self.private_key();
        fs::write(path, private_key)?;
        Ok(())
    }

    /// Load identity from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let private_key = fs::read(path)?;
        Self::from_bytes(&private_key)
    }

    /// Verify signature from another identity's public key
    pub fn verify_external(
        public_key: &[u8],
        data: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        if public_key.len() != 32 {
            return Err(NetworkError::Crypto(
                "Public key must be 32 bytes".to_string(),
            ));
        }

        if signature.len() != 64 {
            return Err(NetworkError::Crypto(
                "Signature must be 64 bytes".to_string(),
            ));
        }

        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(public_key);
        let verifying_key = VerifyingKey::from_bytes(&pk_bytes)
            .map_err(|e| NetworkError::Crypto(format!("Invalid public key: {}", e)))?;

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let sig = Signature::from_bytes(&sig_bytes);

        verifying_key
            .verify(data, &sig)
            .map_err(|e| NetworkError::Crypto(format!("Signature verification failed: {}", e)))
    }

    /// Calculate destination hash from public key
    pub fn hash_from_public_key(public_key: &[u8]) -> DestinationHash {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("destination", &self.destination_hex())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = Identity::generate();
        assert_eq!(identity.public_key().len(), 32);
        assert_eq!(identity.private_key().len(), 32);
        assert_eq!(identity.destination_hash().len(), 32);
    }

    #[test]
    fn test_identity_from_bytes() {
        let identity1 = Identity::generate();
        let private_key = identity1.private_key();

        let identity2 = Identity::from_bytes(&private_key).unwrap();

        assert_eq!(identity1.public_key(), identity2.public_key());
        assert_eq!(identity1.destination_hash(), identity2.destination_hash());
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = Identity::generate();
        let data = b"Hello, Reticulum!";

        let signature = identity.sign(data);
        assert!(identity.verify(data, &signature).is_ok());

        // Wrong data should fail
        let wrong_data = b"Wrong data";
        assert!(identity.verify(wrong_data, &signature).is_err());
    }

    #[test]
    fn test_external_verify() {
        let identity = Identity::generate();
        let data = b"Test message";
        let signature = identity.sign(data);

        let result = Identity::verify_external(&identity.public_key(), data, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_destination_hash() {
        let identity = Identity::generate();
        let hash1 = identity.destination_hash();
        let hash2 = Identity::hash_from_public_key(&identity.public_key());

        assert_eq!(hash1, hash2);
    }
}
