//! Two-Factor Authentication with TOTP (Point 66)
//!
//! Implements Time-based One-Time Password (TOTP) authentication
//! compatible with Google Authenticator, Authy, etc.

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

/// TOTP configuration
#[derive(Debug, Clone)]
pub struct TotpConfig {
    /// Issuer name (shown in authenticator apps)
    pub issuer: String,
    /// Number of digits in the code
    pub digits: u32,
    /// Time step in seconds (usually 30)
    pub period: u64,
    /// Algorithm (SHA1 is most compatible)
    pub algorithm: TotpAlgorithm,
    /// Number of recovery codes to generate
    pub recovery_codes_count: usize,
    /// Window for code validation (allows for clock skew)
    pub validation_window: u32,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            issuer: "RustPress".to_string(),
            digits: 6,
            period: 30,
            algorithm: TotpAlgorithm::SHA1,
            recovery_codes_count: 10,
            validation_window: 1, // Allow 1 step before/after
        }
    }
}

/// TOTP Algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TotpAlgorithm {
    SHA1,
    SHA256,
    SHA512,
}

impl TotpAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SHA1 => "SHA1",
            Self::SHA256 => "SHA256",
            Self::SHA512 => "SHA512",
        }
    }
}

/// TOTP secret for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    /// Base32-encoded secret
    pub secret: String,
    /// Recovery codes (hashed)
    pub recovery_codes: Vec<String>,
    /// Used recovery codes
    pub used_recovery_codes: Vec<String>,
    /// When 2FA was enabled
    pub enabled_at: DateTime<Utc>,
    /// Last successful verification
    pub last_used_at: Option<DateTime<Utc>>,
    /// Last used counter (to prevent replay attacks)
    pub last_counter: Option<u64>,
}

impl TotpSecret {
    pub fn remaining_recovery_codes(&self) -> usize {
        self.recovery_codes.len() - self.used_recovery_codes.len()
    }
}

/// TOTP code wrapper
#[derive(Debug, Clone)]
pub struct TotpCode {
    pub code: String,
    pub counter: u64,
}

/// TOTP Manager
pub struct TotpManager {
    config: TotpConfig,
}

impl TotpManager {
    pub fn new(config: TotpConfig) -> Self {
        Self { config }
    }

    /// Generate a new TOTP secret for a user
    pub fn generate_secret(&self, user_id: Uuid) -> TotpSecret {
        let secret_bytes: [u8; 20] = rand::thread_rng().gen();
        let secret = base32_encode(&secret_bytes);

        let recovery_codes: Vec<String> = (0..self.config.recovery_codes_count)
            .map(|_| self.generate_recovery_code())
            .collect();

        TotpSecret {
            id: Uuid::now_v7(),
            user_id,
            secret,
            recovery_codes: recovery_codes.iter().map(|c| hash_recovery_code(c)).collect(),
            used_recovery_codes: Vec::new(),
            enabled_at: Utc::now(),
            last_used_at: None,
            last_counter: None,
        }
    }

    /// Generate a recovery code
    fn generate_recovery_code(&self) -> String {
        let mut rng = rand::thread_rng();
        let code: String = (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'A' + idx - 10) as char
                }
            })
            .collect();

        // Format as XXXX-XXXX
        format!("{}-{}", &code[0..4], &code[4..8])
    }

    /// Generate provisioning URI for QR code
    pub fn get_provisioning_uri(&self, secret: &TotpSecret, account_name: &str) -> String {
        let encoded_issuer = urlencoding::encode(&self.config.issuer);
        let encoded_account = urlencoding::encode(account_name);

        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            encoded_issuer,
            encoded_account,
            secret.secret,
            encoded_issuer,
            self.config.algorithm.as_str(),
            self.config.digits,
            self.config.period
        )
    }

    /// Generate current TOTP code
    pub fn generate_code(&self, secret: &TotpSecret) -> Result<TotpCode> {
        let counter = self.current_counter();
        let code = self.generate_code_for_counter(&secret.secret, counter)?;

        Ok(TotpCode { code, counter })
    }

    /// Generate TOTP code for a specific counter
    fn generate_code_for_counter(&self, secret: &str, counter: u64) -> Result<String> {
        let secret_bytes = base32_decode(secret).ok_or_else(|| Error::Internal {
            message: "Invalid TOTP secret".to_string(),
            request_id: None,
        })?;

        let counter_bytes = counter.to_be_bytes();

        let mut mac = HmacSha1::new_from_slice(&secret_bytes)
            .map_err(|_| Error::Internal {
                message: "HMAC error".to_string(),
                request_id: None,
            })?;
        mac.update(&counter_bytes);
        let result = mac.finalize().into_bytes();

        // Dynamic truncation
        let offset = (result[result.len() - 1] & 0x0f) as usize;
        let binary = ((result[offset] & 0x7f) as u32) << 24
            | (result[offset + 1] as u32) << 16
            | (result[offset + 2] as u32) << 8
            | (result[offset + 3] as u32);

        let otp = binary % 10u32.pow(self.config.digits);

        Ok(format!("{:0>width$}", otp, width = self.config.digits as usize))
    }

    /// Get current counter value
    fn current_counter(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        now / self.config.period
    }

    /// Verify a TOTP code
    pub fn verify_code(&self, secret: &TotpSecret, code: &str) -> Result<bool> {
        let current = self.current_counter();

        // Check against replay attacks
        if let Some(last) = secret.last_counter {
            if current <= last {
                return Err(Error::InvalidToken {
                    reason: "Code already used".to_string(),
                });
            }
        }

        // Check within window
        let window = self.config.validation_window as u64;
        for counter in (current.saturating_sub(window))..=(current + window) {
            let expected = self.generate_code_for_counter(&secret.secret, counter)?;
            if constant_time_compare(&expected, code) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Verify a recovery code
    pub fn verify_recovery_code(&self, secret: &mut TotpSecret, code: &str) -> Result<bool> {
        let normalized = code.to_uppercase().replace("-", "");
        let hashed = hash_recovery_code(&format!(
            "{}-{}",
            &normalized[0..4],
            &normalized[4..8]
        ));

        if secret.recovery_codes.contains(&hashed) && !secret.used_recovery_codes.contains(&hashed) {
            secret.used_recovery_codes.push(hashed);
            return Ok(true);
        }

        Ok(false)
    }

    /// Regenerate recovery codes
    pub fn regenerate_recovery_codes(&self, secret: &mut TotpSecret) -> Vec<String> {
        let new_codes: Vec<String> = (0..self.config.recovery_codes_count)
            .map(|_| self.generate_recovery_code())
            .collect();

        secret.recovery_codes = new_codes.iter().map(|c| hash_recovery_code(c)).collect();
        secret.used_recovery_codes.clear();

        new_codes
    }
}

impl Default for TotpManager {
    fn default() -> Self {
        Self::new(TotpConfig::default())
    }
}

/// Base32 encode bytes
fn base32_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut buffer = 0u64;
    let mut bits = 0;

    for &byte in bytes {
        buffer = (buffer << 8) | (byte as u64);
        bits += 8;

        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 0x1f) as usize;
            result.push(ALPHABET[idx] as char);
        }
    }

    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 0x1f) as usize;
        result.push(ALPHABET[idx] as char);
    }

    result
}

/// Base32 decode string
fn base32_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let input = input.to_uppercase();
    let input = input.trim_end_matches('=');

    let mut result = Vec::new();
    let mut buffer = 0u64;
    let mut bits = 0;

    for c in input.chars() {
        let idx = ALPHABET.iter().position(|&b| b == c as u8)?;
        buffer = (buffer << 5) | (idx as u64);
        bits += 5;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }

    Some(result)
}

/// Hash a recovery code
fn hash_recovery_code(code: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// Constant-time string comparison
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let manager = TotpManager::default();
        let user_id = Uuid::now_v7();
        let secret = manager.generate_secret(user_id);

        assert!(!secret.secret.is_empty());
        assert_eq!(secret.recovery_codes.len(), 10);
    }

    #[test]
    fn test_generate_and_verify_code() {
        let manager = TotpManager::default();
        let user_id = Uuid::now_v7();
        let secret = manager.generate_secret(user_id);

        let code = manager.generate_code(&secret).unwrap();
        assert_eq!(code.code.len(), 6);

        let is_valid = manager.verify_code(&secret, &code.code).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_provisioning_uri() {
        let manager = TotpManager::default();
        let user_id = Uuid::now_v7();
        let secret = manager.generate_secret(user_id);

        let uri = manager.get_provisioning_uri(&secret, "user@example.com");
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains(&secret.secret));
    }

    #[test]
    fn test_recovery_codes() {
        let manager = TotpManager::default();
        let user_id = Uuid::now_v7();
        let mut secret = manager.generate_secret(user_id);

        // Generate a fresh recovery code (unhashed)
        let code = manager.generate_recovery_code();
        secret.recovery_codes = vec![hash_recovery_code(&code)];

        let is_valid = manager.verify_recovery_code(&mut secret, &code).unwrap();
        assert!(is_valid);

        // Code should not work twice
        let is_valid = manager.verify_recovery_code(&mut secret, &code).unwrap();
        assert!(!is_valid);
    }
}
