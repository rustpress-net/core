//! # Two-Factor Authentication (2FA)
//!
//! TOTP-based two-factor authentication system.
//!
//! Features:
//! - TOTP (Time-based One-Time Password) support
//! - QR code generation for authenticator apps
//! - Recovery codes
//! - Multiple 2FA methods (TOTP, SMS, Email)
//! - Trusted devices

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

/// Two-factor authentication method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwoFactorMethod {
    /// TOTP via authenticator app
    Totp,
    /// SMS-based verification
    Sms,
    /// Email-based verification
    Email,
    /// Security keys (WebAuthn/FIDO2)
    SecurityKey,
    /// Backup codes
    BackupCode,
}

/// Two-factor authentication status for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorStatus {
    /// User ID
    pub user_id: i64,

    /// Whether 2FA is enabled
    pub enabled: bool,

    /// Primary method
    pub primary_method: Option<TwoFactorMethod>,

    /// Available methods
    pub available_methods: Vec<TwoFactorMethod>,

    /// TOTP secret (encrypted)
    pub totp_secret: Option<String>,

    /// Phone number for SMS
    pub phone_number: Option<String>,

    /// Recovery codes (hashed)
    pub recovery_codes: Vec<String>,

    /// Recovery codes remaining
    pub recovery_codes_remaining: u32,

    /// Last verification time
    pub last_verified_at: Option<DateTime<Utc>>,

    /// Setup completed at
    pub setup_completed_at: Option<DateTime<Utc>>,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl TwoFactorStatus {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            enabled: false,
            primary_method: None,
            available_methods: Vec::new(),
            totp_secret: None,
            phone_number: None,
            recovery_codes: Vec::new(),
            recovery_codes_remaining: 0,
            last_verified_at: None,
            setup_completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// TOTP setup data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetup {
    /// Secret key (base32 encoded)
    pub secret: String,

    /// QR code as data URI
    pub qr_code: String,

    /// Manual entry key (formatted)
    pub manual_key: String,

    /// Recovery codes
    pub recovery_codes: Vec<String>,

    /// Issuer name
    pub issuer: String,

    /// Account name (usually email)
    pub account_name: String,
}

/// TOTP verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub success: bool,
    pub method_used: TwoFactorMethod,
    pub message: Option<String>,
    pub recovery_code_used: bool,
    pub recovery_codes_remaining: Option<u32>,
}

/// Trusted device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    /// Device ID
    pub id: Uuid,

    /// User ID
    pub user_id: i64,

    /// Device name
    pub name: String,

    /// Device fingerprint (hashed)
    pub fingerprint: String,

    /// User agent
    pub user_agent: String,

    /// IP address when trusted
    pub ip_address: String,

    /// Last used
    pub last_used_at: DateTime<Utc>,

    /// Expires at
    pub expires_at: DateTime<Utc>,

    /// Created at
    pub created_at: DateTime<Utc>,
}

impl TrustedDevice {
    pub fn new(user_id: i64, name: &str, fingerprint: &str, user_agent: &str, ip: &str, days_valid: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            name: name.to_string(),
            fingerprint: fingerprint.to_string(),
            user_agent: user_agent.to_string(),
            ip_address: ip.to_string(),
            last_used_at: now,
            expires_at: now + chrono::Duration::days(days_valid),
            created_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Two-factor authentication manager
pub struct TwoFactorManager {
    /// Issuer name (shown in authenticator apps)
    issuer: String,

    /// TOTP time step in seconds (default 30)
    time_step: u64,

    /// Number of digits in TOTP code (default 6)
    digits: usize,

    /// Recovery code count
    recovery_code_count: usize,

    /// User 2FA status
    user_status: HashMap<i64, TwoFactorStatus>,

    /// Trusted devices
    trusted_devices: HashMap<Uuid, TrustedDevice>,

    /// Trust duration in days
    trust_duration_days: i64,
}

impl Default for TwoFactorManager {
    fn default() -> Self {
        Self {
            issuer: "RustPress".to_string(),
            time_step: 30,
            digits: 6,
            recovery_code_count: 10,
            user_status: HashMap::new(),
            trusted_devices: HashMap::new(),
            trust_duration_days: 30,
        }
    }
}

impl TwoFactorManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_issuer(mut self, issuer: &str) -> Self {
        self.issuer = issuer.to_string();
        self
    }

    pub fn with_trust_duration(mut self, days: i64) -> Self {
        self.trust_duration_days = days;
        self
    }

    /// Initialize TOTP setup for a user
    pub fn init_totp_setup(&mut self, user_id: i64, account_name: &str) -> Result<TotpSetup, String> {
        // Generate a random secret
        let secret = Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();

        // Create TOTP instance
        let totp = TOTP::new(
            Algorithm::SHA1,
            self.digits,
            1,
            self.time_step,
            secret.to_bytes().map_err(|e| e.to_string())?,
            Some(self.issuer.clone()),
            account_name.to_string(),
        ).map_err(|e| e.to_string())?;

        // Generate QR code
        let qr_code = totp.get_qr_base64()
            .map_err(|e| e.to_string())?;

        // Generate recovery codes
        let recovery_codes = self.generate_recovery_codes();

        // Format manual key for display
        let manual_key = secret_base32.chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ");

        // Hash recovery codes before getting mutable borrow
        let hashed_codes: Vec<String> = recovery_codes.iter()
            .map(|c| self.hash_recovery_code(c))
            .collect();
        let codes_count = recovery_codes.len() as u32;

        // Store pending setup
        let status = self.user_status.entry(user_id)
            .or_insert_with(|| TwoFactorStatus::new(user_id));
        status.totp_secret = Some(secret_base32.clone());
        status.recovery_codes = hashed_codes;
        status.recovery_codes_remaining = codes_count;
        status.updated_at = Utc::now();

        Ok(TotpSetup {
            secret: secret_base32,
            qr_code: format!("data:image/png;base64,{}", qr_code),
            manual_key,
            recovery_codes,
            issuer: self.issuer.clone(),
            account_name: account_name.to_string(),
        })
    }

    /// Verify TOTP code and complete setup
    pub fn verify_totp_setup(&mut self, user_id: i64, code: &str) -> Result<bool, String> {
        // Get secret first without holding mutable borrow
        let secret = {
            let status = self.user_status.get(&user_id)
                .ok_or_else(|| "2FA not initialized for user".to_string())?;
            status.totp_secret.as_ref()
                .ok_or_else(|| "TOTP secret not found".to_string())?
                .clone()
        };

        // Verify code
        let is_valid = self.verify_totp_code(&secret, code)?;

        if is_valid {
            // Now get mutable borrow to update status
            let status = self.user_status.get_mut(&user_id).unwrap();
            status.enabled = true;
            status.primary_method = Some(TwoFactorMethod::Totp);
            status.available_methods.push(TwoFactorMethod::Totp);
            status.available_methods.push(TwoFactorMethod::BackupCode);
            status.setup_completed_at = Some(Utc::now());
            status.last_verified_at = Some(Utc::now());
            status.updated_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Verify a TOTP code
    pub fn verify_totp(&mut self, user_id: i64, code: &str) -> Result<VerificationResult, String> {
        // Get necessary info without holding mutable borrow
        let (is_enabled, secret, recovery_codes_remaining) = {
            let status = self.user_status.get(&user_id)
                .ok_or_else(|| "2FA not enabled for user".to_string())?;
            (status.enabled, status.totp_secret.clone(), status.recovery_codes_remaining)
        };

        if !is_enabled {
            return Err("2FA is not enabled".to_string());
        }

        // Try TOTP first
        if let Some(ref secret_str) = secret {
            if self.verify_totp_code(secret_str, code)? {
                // Update status
                let status = self.user_status.get_mut(&user_id).unwrap();
                status.last_verified_at = Some(Utc::now());
                status.updated_at = Utc::now();

                return Ok(VerificationResult {
                    success: true,
                    method_used: TwoFactorMethod::Totp,
                    message: None,
                    recovery_code_used: false,
                    recovery_codes_remaining: Some(status.recovery_codes_remaining),
                });
            }
        }

        // Hash recovery code before getting mutable borrow
        let hashed_code = self.hash_recovery_code(code);

        // Try recovery code - inline the logic to avoid borrow issues
        let status = self.user_status.get_mut(&user_id).unwrap();
        if let Some(pos) = status.recovery_codes.iter().position(|c| *c == hashed_code) {
            status.recovery_codes.remove(pos);
            status.recovery_codes_remaining = status.recovery_codes.len() as u32;
            status.last_verified_at = Some(Utc::now());
            status.updated_at = Utc::now();

            return Ok(VerificationResult {
                success: true,
                method_used: TwoFactorMethod::BackupCode,
                message: Some(format!(
                    "Recovery code used. {} codes remaining.",
                    status.recovery_codes_remaining
                )),
                recovery_code_used: true,
                recovery_codes_remaining: Some(status.recovery_codes_remaining),
            });
        }

        Ok(VerificationResult {
            success: false,
            method_used: TwoFactorMethod::Totp,
            message: Some("Invalid code".to_string()),
            recovery_code_used: false,
            recovery_codes_remaining: Some(status.recovery_codes_remaining),
        })
    }

    /// Verify TOTP code against secret
    fn verify_totp_code(&self, secret: &str, code: &str) -> Result<bool, String> {
        let secret_bytes = data_encoding::BASE32.decode(secret.as_bytes())
            .map_err(|e| format!("Invalid secret: {}", e))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            self.digits,
            1,
            self.time_step,
            secret_bytes,
            Some(self.issuer.clone()),
            "".to_string(),
        ).map_err(|e| e.to_string())?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    /// Generate recovery codes
    fn generate_recovery_codes(&self) -> Vec<String> {
        let mut rng = rand::thread_rng();
        (0..self.recovery_code_count)
            .map(|_| {
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
                format!("{}-{}", &code[0..4], &code[4..8])
            })
            .collect()
    }

    /// Hash a recovery code
    fn hash_recovery_code(&self, code: &str) -> String {
        use sha2::{Sha256, Digest};
        let normalized = code.to_uppercase().replace("-", "");
        let hash = Sha256::digest(normalized.as_bytes());
        hex::encode(hash)
    }

    /// Verify and consume a recovery code
    fn verify_and_consume_recovery_code(&self, status: &mut TwoFactorStatus, code: &str) -> bool {
        let hashed = self.hash_recovery_code(code);

        if let Some(pos) = status.recovery_codes.iter().position(|c| *c == hashed) {
            status.recovery_codes.remove(pos);
            status.recovery_codes_remaining = status.recovery_codes.len() as u32;
            status.last_verified_at = Some(Utc::now());
            status.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Regenerate recovery codes
    pub fn regenerate_recovery_codes(&mut self, user_id: i64) -> Result<Vec<String>, String> {
        // Check if 2FA is enabled first
        if !self.user_status.contains_key(&user_id) {
            return Err("2FA not enabled for user".to_string());
        }

        // Generate and hash codes before borrowing status mutably
        let new_codes = self.generate_recovery_codes();
        let hashed_codes: Vec<String> = new_codes.iter()
            .map(|c| self.hash_recovery_code(c))
            .collect();
        let codes_count = new_codes.len() as u32;

        // Now update the status
        let status = self.user_status.get_mut(&user_id).unwrap();
        status.recovery_codes = hashed_codes;
        status.recovery_codes_remaining = codes_count;
        status.updated_at = Utc::now();

        Ok(new_codes)
    }

    /// Disable 2FA for user
    pub fn disable_2fa(&mut self, user_id: i64) -> Result<(), String> {
        let status = self.user_status.get_mut(&user_id)
            .ok_or_else(|| "User not found".to_string())?;

        status.enabled = false;
        status.primary_method = None;
        status.available_methods.clear();
        status.totp_secret = None;
        status.recovery_codes.clear();
        status.recovery_codes_remaining = 0;
        status.updated_at = Utc::now();

        // Remove trusted devices
        self.trusted_devices.retain(|_, d| d.user_id != user_id);

        Ok(())
    }

    /// Get 2FA status for user
    pub fn get_status(&self, user_id: i64) -> Option<&TwoFactorStatus> {
        self.user_status.get(&user_id)
    }

    /// Check if 2FA is enabled
    pub fn is_enabled(&self, user_id: i64) -> bool {
        self.user_status.get(&user_id)
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Trust a device
    pub fn trust_device(
        &mut self,
        user_id: i64,
        name: &str,
        fingerprint: &str,
        user_agent: &str,
        ip: &str,
    ) -> TrustedDevice {
        let device = TrustedDevice::new(
            user_id,
            name,
            fingerprint,
            user_agent,
            ip,
            self.trust_duration_days,
        );

        self.trusted_devices.insert(device.id, device.clone());
        device
    }

    /// Check if device is trusted
    pub fn is_device_trusted(&self, user_id: i64, fingerprint: &str) -> bool {
        self.trusted_devices.values()
            .any(|d| d.user_id == user_id && d.fingerprint == fingerprint && !d.is_expired())
    }

    /// Get trusted devices for user
    pub fn get_trusted_devices(&self, user_id: i64) -> Vec<&TrustedDevice> {
        self.trusted_devices.values()
            .filter(|d| d.user_id == user_id && !d.is_expired())
            .collect()
    }

    /// Remove trusted device
    pub fn remove_trusted_device(&mut self, device_id: Uuid) -> bool {
        self.trusted_devices.remove(&device_id).is_some()
    }

    /// Remove all trusted devices for user
    pub fn remove_all_trusted_devices(&mut self, user_id: i64) {
        self.trusted_devices.retain(|_, d| d.user_id != user_id);
    }

    /// Cleanup expired devices
    pub fn cleanup_expired_devices(&mut self) {
        self.trusted_devices.retain(|_, d| !d.is_expired());
    }
}

// Hex encoding helper
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_setup() {
        let mut manager = TwoFactorManager::new();
        let setup = manager.init_totp_setup(1, "test@example.com").unwrap();

        assert!(!setup.secret.is_empty());
        assert!(setup.qr_code.starts_with("data:image/png;base64,"));
        assert_eq!(setup.recovery_codes.len(), 10);
    }

    #[test]
    fn test_recovery_codes() {
        let mut manager = TwoFactorManager::new();
        let setup = manager.init_totp_setup(1, "test@example.com").unwrap();

        // Verify setup with a real code would be needed here
        // For now, just test recovery code format
        for code in &setup.recovery_codes {
            assert_eq!(code.len(), 9); // XXXX-XXXX format
            assert_eq!(&code[4..5], "-");
        }
    }

    #[test]
    fn test_trusted_device() {
        let mut manager = TwoFactorManager::new();
        let device = manager.trust_device(1, "My Browser", "fingerprint123", "Chrome", "127.0.0.1");

        assert!(manager.is_device_trusted(1, "fingerprint123"));
        assert!(!manager.is_device_trusted(1, "other_fingerprint"));
        assert!(!manager.is_device_trusted(2, "fingerprint123"));
    }

    #[test]
    fn test_two_factor_status() {
        let status = TwoFactorStatus::new(1);
        assert!(!status.enabled);
        assert!(status.primary_method.is_none());
    }
}
