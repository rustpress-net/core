//! Cryptographic utilities for plugin secrets and API keys.
//!
//! This module provides encryption and decryption for sensitive plugin data
//! like API keys, secrets, and credentials using ChaCha20-Poly1305 AEAD.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Nonce size for ChaCha20-Poly1305 (12 bytes)
const NONCE_SIZE: usize = 12;

/// Key size for ChaCha20-Poly1305 (32 bytes)
const KEY_SIZE: usize = 32;

/// Encryption errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid ciphertext format")]
    InvalidCiphertext,

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// Result type for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Encryption key wrapper with automatic zeroization
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey {
    key: [u8; KEY_SIZE],
}

impl EncryptionKey {
    /// Create a new random encryption key
    pub fn generate() -> Self {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    /// Create from a hex-encoded string
    pub fn from_hex(hex_str: &str) -> CryptoResult<Self> {
        let bytes = hex::decode(hex_str).map_err(|e| {
            CryptoError::InvalidKeyLength {
                expected: KEY_SIZE,
                actual: 0,
            }
        })?;

        Self::from_bytes(&bytes)
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != KEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: KEY_SIZE,
                actual: bytes.len(),
            });
        }

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Derive a key from a master key and context using BLAKE3
    pub fn derive(master_key: &EncryptionKey, context: &str) -> Self {
        let mut hasher = blake3::Hasher::new_derive_key(context);
        hasher.update(&master_key.key);
        let hash = hasher.finalize();

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(hash.as_bytes());
        Self { key }
    }

    /// Export as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.key)
    }

    /// Get the raw key bytes (use carefully)
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.key
    }
}

/// API key encryptor for securing plugin credentials
pub struct ApiKeyEncryptor {
    cipher: ChaCha20Poly1305,
}

impl ApiKeyEncryptor {
    /// Create a new encryptor with the given key
    pub fn new(key: &EncryptionKey) -> Self {
        let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes())
            .expect("Invalid key length - this should never happen");
        Self { cipher }
    }

    /// Encrypt a plaintext API key
    ///
    /// Returns a base64-encoded string containing: nonce || ciphertext
    pub fn encrypt(&self, plaintext: &str) -> CryptoResult<String> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Combine nonce + ciphertext and base64 encode
        let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// Decrypt a base64-encoded ciphertext
    pub fn decrypt(&self, encrypted: &str) -> CryptoResult<String> {
        // Decode base64
        let combined = BASE64.decode(encrypted)?;

        // Validate length
        if combined.len() < NONCE_SIZE + 16 {
            // 16 = auth tag size
            return Err(CryptoError::InvalidCiphertext);
        }

        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&combined[..NONCE_SIZE]);
        let ciphertext = &combined[NONCE_SIZE..];

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }
}

/// Encrypted value wrapper for serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedValue {
    /// The encrypted value (base64-encoded)
    pub ciphertext: String,
    /// Whether this value is encrypted (for backwards compatibility)
    pub encrypted: bool,
}

impl EncryptedValue {
    /// Create from plaintext by encrypting it
    pub fn encrypt(plaintext: &str, encryptor: &ApiKeyEncryptor) -> CryptoResult<Self> {
        let ciphertext = encryptor.encrypt(plaintext)?;
        Ok(Self {
            ciphertext,
            encrypted: true,
        })
    }

    /// Create from already-encrypted ciphertext
    pub fn from_encrypted(ciphertext: String) -> Self {
        Self {
            ciphertext,
            encrypted: true,
        }
    }

    /// Create from plaintext without encrypting (for migration/legacy support)
    pub fn from_plaintext(plaintext: String) -> Self {
        Self {
            ciphertext: plaintext,
            encrypted: false,
        }
    }

    /// Decrypt the value
    pub fn decrypt(&self, encryptor: &ApiKeyEncryptor) -> CryptoResult<String> {
        if self.encrypted {
            encryptor.decrypt(&self.ciphertext)
        } else {
            // Return as-is for legacy unencrypted values
            Ok(self.ciphertext.clone())
        }
    }
}

/// Helper to check if a string appears to be encrypted
pub fn is_encrypted(value: &str) -> bool {
    // Encrypted values are base64 and start with the nonce
    // They should be at least nonce_size + tag_size (12 + 16 = 28 bytes) when decoded
    if let Ok(decoded) = BASE64.decode(value) {
        decoded.len() >= 28
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();

        // Keys should be different
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_key_from_hex() {
        let key = EncryptionKey::generate();
        let hex = key.to_hex();
        let recovered = EncryptionKey::from_hex(&hex).unwrap();

        assert_eq!(key.as_bytes(), recovered.as_bytes());
    }

    #[test]
    fn test_key_derivation() {
        let master = EncryptionKey::generate();
        let derived1 = EncryptionKey::derive(&master, "api_keys");
        let derived2 = EncryptionKey::derive(&master, "secrets");

        // Derived keys should be different
        assert_ne!(derived1.as_bytes(), derived2.as_bytes());

        // Same context should produce same key
        let derived1_again = EncryptionKey::derive(&master, "api_keys");
        assert_eq!(derived1.as_bytes(), derived1_again.as_bytes());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::generate();
        let encryptor = ApiKeyEncryptor::new(&key);

        let plaintext = "sk_live_abc123xyz789";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // Encrypted should be different from plaintext
        assert_ne!(encrypted, plaintext);

        // Decrypt should return original
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_each_time() {
        let key = EncryptionKey::generate();
        let encryptor = ApiKeyEncryptor::new(&key);

        let plaintext = "my_secret_key";
        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Each encryption should produce different ciphertext (due to random nonce)
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt to the same value
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_encrypted_value() {
        let key = EncryptionKey::generate();
        let encryptor = ApiKeyEncryptor::new(&key);

        let plaintext = "api_key_12345";
        let encrypted = EncryptedValue::encrypt(plaintext, &encryptor).unwrap();

        assert!(encrypted.encrypted);
        assert_ne!(encrypted.ciphertext, plaintext);

        let decrypted = encrypted.decrypt(&encryptor).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_legacy_plaintext_value() {
        let key = EncryptionKey::generate();
        let encryptor = ApiKeyEncryptor::new(&key);

        // Simulate legacy unencrypted value
        let legacy = EncryptedValue::from_plaintext("legacy_key".to_string());

        assert!(!legacy.encrypted);

        // Should return as-is without decryption
        let value = legacy.decrypt(&encryptor).unwrap();
        assert_eq!(value, "legacy_key");
    }

    #[test]
    fn test_invalid_ciphertext() {
        let key = EncryptionKey::generate();
        let encryptor = ApiKeyEncryptor::new(&key);

        // Too short
        let result = encryptor.decrypt("dG9vIHNob3J0");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();

        let encryptor1 = ApiKeyEncryptor::new(&key1);
        let encryptor2 = ApiKeyEncryptor::new(&key2);

        let encrypted = encryptor1.encrypt("secret").unwrap();

        // Decrypting with wrong key should fail
        let result = encryptor2.decrypt(&encrypted);
        assert!(result.is_err());
    }
}
