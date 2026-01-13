//! WebAuthn/Passkey Support (Point 80)
//!
//! Implements WebAuthn for passwordless authentication using
//! platform authenticators and security keys.

use chrono::{DateTime, Utc};
use rand::Rng;
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// WebAuthn credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    /// Credential ID from authenticator (base64url encoded)
    pub credential_id: String,
    /// Public key (COSE encoded, base64url)
    pub public_key: String,
    /// Credential name/nickname
    pub name: String,
    /// Authenticator AAGUID (identifies the authenticator model)
    pub aaguid: Option<String>,
    /// Signature counter (for cloning detection)
    pub sign_count: u32,
    /// Credential type
    pub credential_type: CredentialType,
    /// User verification capability
    pub user_verified: bool,
    /// Backup eligibility
    pub backup_eligible: bool,
    /// Backup state
    pub backed_up: bool,
    /// Transports (usb, nfc, ble, internal, hybrid)
    pub transports: Vec<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last used timestamp
    pub last_used_at: Option<DateTime<Utc>>,
    /// Is this the primary credential?
    pub is_primary: bool,
    /// Is active
    pub is_active: bool,
}

impl WebAuthnCredential {
    pub fn is_valid(&self) -> bool {
        self.is_active
    }
}

/// Credential type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialType {
    /// Platform authenticator (TouchID, FaceID, Windows Hello)
    Platform,
    /// Cross-platform security key (YubiKey, etc)
    CrossPlatform,
    /// Unknown/other
    Unknown,
}

impl Default for CredentialType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// WebAuthn registration challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub challenge: String,
    pub rp_id: String,
    pub rp_name: String,
    pub user_handle: String,
    pub user_name: String,
    pub user_display_name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl RegistrationChallenge {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

/// WebAuthn authentication challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub id: Uuid,
    pub challenge: String,
    pub rp_id: String,
    /// Optional user ID for username-less auth
    pub user_id: Option<Uuid>,
    /// Allowed credential IDs
    pub allowed_credentials: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AuthenticationChallenge {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

/// WebAuthn configuration
#[derive(Debug, Clone)]
pub struct WebAuthnConfig {
    /// Relying Party ID (usually the domain)
    pub rp_id: String,
    /// Relying Party name
    pub rp_name: String,
    /// Origin for verification
    pub origin: String,
    /// Challenge timeout in seconds
    pub challenge_timeout_secs: u64,
    /// Require user verification
    pub require_user_verification: bool,
    /// Allowed authenticator attachment
    pub authenticator_attachment: Option<AuthenticatorAttachment>,
    /// Resident key requirement
    pub resident_key: ResidentKeyRequirement,
    /// Maximum credentials per user
    pub max_credentials_per_user: usize,
}

impl Default for WebAuthnConfig {
    fn default() -> Self {
        Self {
            rp_id: "localhost".to_string(),
            rp_name: "RustPress".to_string(),
            origin: "https://localhost".to_string(),
            challenge_timeout_secs: 300,
            require_user_verification: true,
            authenticator_attachment: None,
            resident_key: ResidentKeyRequirement::Preferred,
            max_credentials_per_user: 10,
        }
    }
}

/// Authenticator attachment preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthenticatorAttachment {
    Platform,
    CrossPlatform,
}

/// Resident key requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResidentKeyRequirement {
    Discouraged,
    Preferred,
    Required,
}

/// Registration options (sent to client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationOptions {
    pub rp: RelyingParty,
    pub user: PublicKeyCredentialUserEntity,
    pub challenge: String,
    pub pub_key_cred_params: Vec<PublicKeyCredentialParameters>,
    pub timeout: u64,
    pub attestation: AttestationConveyancePreference,
    pub exclude_credentials: Vec<PublicKeyCredentialDescriptor>,
    pub authenticator_selection: Option<AuthenticatorSelectionCriteria>,
}

/// Relying party entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialUserEntity {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

/// Public key credential parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialParameters {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub alg: i32,
}

/// Attestation conveyance preference
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttestationConveyancePreference {
    None,
    Indirect,
    Direct,
    Enterprise,
}

/// Credential descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialDescriptor {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transports: Option<Vec<String>>,
}

/// Authenticator selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorSelectionCriteria {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticator_attachment: Option<String>,
    pub resident_key: String,
    pub user_verification: String,
}

/// Authentication options (sent to client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationOptions {
    pub challenge: String,
    pub timeout: u64,
    pub rp_id: String,
    pub allow_credentials: Vec<PublicKeyCredentialDescriptor>,
    pub user_verification: String,
}

/// Registration response (from client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: String,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub response: AuthenticatorAttestationResponse,
    pub client_extension_results: Option<serde_json::Value>,
    pub authenticator_attachment: Option<String>,
}

/// Attestation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: String,
    pub attestation_object: String,
    pub transports: Option<Vec<String>>,
}

/// Authentication response (from client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub id: String,
    pub raw_id: String,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub response: AuthenticatorAssertionResponse,
    pub client_extension_results: Option<serde_json::Value>,
}

/// Assertion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    pub user_handle: Option<String>,
}

/// WebAuthn storage trait
#[async_trait::async_trait]
pub trait WebAuthnStore: Send + Sync {
    // Credentials
    async fn store_credential(&self, credential: &WebAuthnCredential) -> Result<()>;
    async fn get_credential(&self, credential_id: &str) -> Result<Option<WebAuthnCredential>>;
    async fn get_user_credentials(&self, user_id: Uuid) -> Result<Vec<WebAuthnCredential>>;
    async fn update_credential(&self, credential: &WebAuthnCredential) -> Result<()>;
    async fn delete_credential(&self, id: Uuid) -> Result<()>;
    async fn count_user_credentials(&self, user_id: Uuid) -> Result<usize>;

    // Registration challenges
    async fn store_registration_challenge(&self, challenge: &RegistrationChallenge) -> Result<()>;
    async fn get_registration_challenge(&self, challenge: &str) -> Result<Option<RegistrationChallenge>>;
    async fn delete_registration_challenge(&self, id: Uuid) -> Result<()>;

    // Authentication challenges
    async fn store_auth_challenge(&self, challenge: &AuthenticationChallenge) -> Result<()>;
    async fn get_auth_challenge(&self, challenge: &str) -> Result<Option<AuthenticationChallenge>>;
    async fn delete_auth_challenge(&self, id: Uuid) -> Result<()>;

    // Cleanup
    async fn cleanup_expired_challenges(&self) -> Result<u64>;
}

/// WebAuthn manager
pub struct WebAuthnManager<S: WebAuthnStore> {
    store: S,
    config: WebAuthnConfig,
}

impl<S: WebAuthnStore> WebAuthnManager<S> {
    pub fn new(store: S, config: WebAuthnConfig) -> Self {
        Self { store, config }
    }

    /// Generate a random challenge
    fn generate_challenge() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64_url_encode(&bytes)
    }

    /// Generate user handle
    fn generate_user_handle(user_id: Uuid) -> String {
        base64_url_encode(user_id.as_bytes())
    }

    /// Begin registration ceremony
    pub async fn begin_registration(
        &self,
        user_id: Uuid,
        user_name: &str,
        user_display_name: &str,
    ) -> Result<(RegistrationChallenge, RegistrationOptions)> {
        // Check credential limit
        let count = self.store.count_user_credentials(user_id).await?;
        if count >= self.config.max_credentials_per_user {
            return Err(Error::InvalidInput {
                field: "credentials".to_string(),
                message: format!("Maximum {} credentials per user", self.config.max_credentials_per_user),
            });
        }

        let challenge = Self::generate_challenge();
        let user_handle = Self::generate_user_handle(user_id);
        let now = Utc::now();

        let reg_challenge = RegistrationChallenge {
            id: Uuid::now_v7(),
            user_id,
            challenge: challenge.clone(),
            rp_id: self.config.rp_id.clone(),
            rp_name: self.config.rp_name.clone(),
            user_handle: user_handle.clone(),
            user_name: user_name.to_string(),
            user_display_name: user_display_name.to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::seconds(self.config.challenge_timeout_secs as i64),
        };

        self.store.store_registration_challenge(&reg_challenge).await?;

        // Get existing credentials to exclude
        let existing = self.store.get_user_credentials(user_id).await?;
        let exclude_credentials: Vec<_> = existing
            .iter()
            .map(|c| PublicKeyCredentialDescriptor {
                cred_type: "public-key".to_string(),
                id: c.credential_id.clone(),
                transports: Some(c.transports.clone()),
            })
            .collect();

        let options = RegistrationOptions {
            rp: RelyingParty {
                id: self.config.rp_id.clone(),
                name: self.config.rp_name.clone(),
            },
            user: PublicKeyCredentialUserEntity {
                id: user_handle,
                name: user_name.to_string(),
                display_name: user_display_name.to_string(),
            },
            challenge,
            pub_key_cred_params: vec![
                PublicKeyCredentialParameters {
                    cred_type: "public-key".to_string(),
                    alg: -7,  // ES256
                },
                PublicKeyCredentialParameters {
                    cred_type: "public-key".to_string(),
                    alg: -257, // RS256
                },
            ],
            timeout: self.config.challenge_timeout_secs * 1000,
            attestation: AttestationConveyancePreference::None,
            exclude_credentials,
            authenticator_selection: Some(AuthenticatorSelectionCriteria {
                authenticator_attachment: self.config.authenticator_attachment.map(|a| {
                    match a {
                        AuthenticatorAttachment::Platform => "platform",
                        AuthenticatorAttachment::CrossPlatform => "cross-platform",
                    }.to_string()
                }),
                resident_key: match self.config.resident_key {
                    ResidentKeyRequirement::Discouraged => "discouraged",
                    ResidentKeyRequirement::Preferred => "preferred",
                    ResidentKeyRequirement::Required => "required",
                }.to_string(),
                user_verification: if self.config.require_user_verification {
                    "required"
                } else {
                    "preferred"
                }.to_string(),
            }),
        };

        Ok((reg_challenge, options))
    }

    /// Complete registration ceremony
    pub async fn complete_registration(
        &self,
        challenge_str: &str,
        response: &RegistrationResponse,
        name: &str,
    ) -> Result<WebAuthnCredential> {
        // Get and validate challenge
        let challenge = self.store
            .get_registration_challenge(challenge_str)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid registration challenge".to_string(),
            })?;

        if !challenge.is_valid() {
            return Err(Error::Authentication {
                message: "Registration challenge expired".to_string(),
            });
        }

        // In a real implementation, you would:
        // 1. Decode and verify client_data_json
        // 2. Verify the origin matches
        // 3. Verify the challenge matches
        // 4. Decode attestation_object
        // 5. Verify attestation (if required)
        // 6. Extract and store the public key

        // For now, create a credential with placeholder data
        let credential = WebAuthnCredential {
            id: Uuid::now_v7(),
            user_id: challenge.user_id,
            credential_id: response.id.clone(),
            public_key: response.response.attestation_object.clone(), // Would be extracted
            name: name.to_string(),
            aaguid: None,
            sign_count: 0,
            credential_type: response.authenticator_attachment
                .as_ref()
                .map(|a| {
                    if a == "platform" {
                        CredentialType::Platform
                    } else {
                        CredentialType::CrossPlatform
                    }
                })
                .unwrap_or(CredentialType::Unknown),
            user_verified: self.config.require_user_verification,
            backup_eligible: false,
            backed_up: false,
            transports: response.response.transports.clone().unwrap_or_default(),
            created_at: Utc::now(),
            last_used_at: None,
            is_primary: false,
            is_active: true,
        };

        self.store.store_credential(&credential).await?;
        self.store.delete_registration_challenge(challenge.id).await?;

        Ok(credential)
    }

    /// Begin authentication ceremony
    pub async fn begin_authentication(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<(AuthenticationChallenge, AuthenticationOptions)> {
        let challenge = Self::generate_challenge();
        let now = Utc::now();

        // Get allowed credentials
        let allowed_credentials = if let Some(uid) = user_id {
            let creds = self.store.get_user_credentials(uid).await?;
            creds
                .iter()
                .filter(|c| c.is_valid())
                .map(|c| c.credential_id.clone())
                .collect()
        } else {
            Vec::new()
        };

        let auth_challenge = AuthenticationChallenge {
            id: Uuid::now_v7(),
            challenge: challenge.clone(),
            rp_id: self.config.rp_id.clone(),
            user_id,
            allowed_credentials: allowed_credentials.clone(),
            created_at: now,
            expires_at: now + chrono::Duration::seconds(self.config.challenge_timeout_secs as i64),
        };

        self.store.store_auth_challenge(&auth_challenge).await?;

        let options = AuthenticationOptions {
            challenge,
            timeout: self.config.challenge_timeout_secs * 1000,
            rp_id: self.config.rp_id.clone(),
            allow_credentials: if let Some(uid) = user_id {
                let creds = self.store.get_user_credentials(uid).await?;
                creds
                    .iter()
                    .filter(|c| c.is_valid())
                    .map(|c| PublicKeyCredentialDescriptor {
                        cred_type: "public-key".to_string(),
                        id: c.credential_id.clone(),
                        transports: Some(c.transports.clone()),
                    })
                    .collect()
            } else {
                Vec::new()
            },
            user_verification: if self.config.require_user_verification {
                "required"
            } else {
                "preferred"
            }.to_string(),
        };

        Ok((auth_challenge, options))
    }

    /// Complete authentication ceremony
    pub async fn complete_authentication(
        &self,
        challenge_str: &str,
        response: &AuthenticationResponse,
    ) -> Result<WebAuthnCredential> {
        // Get and validate challenge
        let challenge = self.store
            .get_auth_challenge(challenge_str)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid authentication challenge".to_string(),
            })?;

        if !challenge.is_valid() {
            return Err(Error::Authentication {
                message: "Authentication challenge expired".to_string(),
            });
        }

        // Get credential
        let mut credential = self.store
            .get_credential(&response.id)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Unknown credential".to_string(),
            })?;

        if !credential.is_valid() {
            return Err(Error::Authentication {
                message: "Credential is disabled".to_string(),
            });
        }

        // In a real implementation, you would:
        // 1. Decode and verify client_data_json
        // 2. Verify the origin matches
        // 3. Verify the challenge matches
        // 4. Verify the signature using the stored public key
        // 5. Check and update the sign counter

        // Update credential usage
        credential.last_used_at = Some(Utc::now());
        credential.sign_count += 1;
        self.store.update_credential(&credential).await?;
        self.store.delete_auth_challenge(challenge.id).await?;

        Ok(credential)
    }

    /// Get user's credentials
    pub async fn get_user_credentials(&self, user_id: Uuid) -> Result<Vec<WebAuthnCredential>> {
        self.store.get_user_credentials(user_id).await
    }

    /// Delete a credential
    pub async fn delete_credential(&self, user_id: Uuid, credential_id: Uuid) -> Result<()> {
        let cred = self.store.get_credential(&credential_id.to_string()).await?;
        if let Some(c) = cred {
            if c.user_id != user_id {
                return Err(Error::Authorization {
                    action: "Cannot delete another user's credential".to_string(),
                    required: "webauthn:delete".to_string(),
                });
            }
        }
        self.store.delete_credential(credential_id).await
    }

    /// Rename a credential
    pub async fn rename_credential(&self, credential_id: Uuid, new_name: &str) -> Result<()> {
        if let Some(mut cred) = self.store.get_credential(&credential_id.to_string()).await? {
            cred.name = new_name.to_string();
            self.store.update_credential(&cred).await?;
        }
        Ok(())
    }

    /// Check if user has any credentials
    pub async fn has_credentials(&self, user_id: Uuid) -> Result<bool> {
        let count = self.store.count_user_credentials(user_id).await?;
        Ok(count > 0)
    }

    /// Cleanup expired challenges
    pub async fn cleanup(&self) -> Result<u64> {
        self.store.cleanup_expired_challenges().await
    }

    /// Get config
    pub fn config(&self) -> &WebAuthnConfig {
        &self.config
    }
}

/// In-memory WebAuthn store
pub struct InMemoryWebAuthnStore {
    credentials: RwLock<HashMap<String, WebAuthnCredential>>,
    registration_challenges: RwLock<HashMap<String, RegistrationChallenge>>,
    auth_challenges: RwLock<HashMap<String, AuthenticationChallenge>>,
}

impl InMemoryWebAuthnStore {
    pub fn new() -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
            registration_challenges: RwLock::new(HashMap::new()),
            auth_challenges: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryWebAuthnStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl WebAuthnStore for InMemoryWebAuthnStore {
    async fn store_credential(&self, credential: &WebAuthnCredential) -> Result<()> {
        let mut creds = self.credentials.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        creds.insert(credential.credential_id.clone(), credential.clone());
        Ok(())
    }

    async fn get_credential(&self, credential_id: &str) -> Result<Option<WebAuthnCredential>> {
        let creds = self.credentials.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(creds.get(credential_id).cloned())
    }

    async fn get_user_credentials(&self, user_id: Uuid) -> Result<Vec<WebAuthnCredential>> {
        let creds = self.credentials.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(creds.values().filter(|c| c.user_id == user_id).cloned().collect())
    }

    async fn update_credential(&self, credential: &WebAuthnCredential) -> Result<()> {
        let mut creds = self.credentials.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        creds.insert(credential.credential_id.clone(), credential.clone());
        Ok(())
    }

    async fn delete_credential(&self, id: Uuid) -> Result<()> {
        let mut creds = self.credentials.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        creds.retain(|_, c| c.id != id);
        Ok(())
    }

    async fn count_user_credentials(&self, user_id: Uuid) -> Result<usize> {
        let creds = self.credentials.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(creds.values().filter(|c| c.user_id == user_id && c.is_active).count())
    }

    async fn store_registration_challenge(&self, challenge: &RegistrationChallenge) -> Result<()> {
        let mut challenges = self.registration_challenges.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        challenges.insert(challenge.challenge.clone(), challenge.clone());
        Ok(())
    }

    async fn get_registration_challenge(&self, challenge: &str) -> Result<Option<RegistrationChallenge>> {
        let challenges = self.registration_challenges.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(challenges.get(challenge).cloned())
    }

    async fn delete_registration_challenge(&self, id: Uuid) -> Result<()> {
        let mut challenges = self.registration_challenges.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        challenges.retain(|_, c| c.id != id);
        Ok(())
    }

    async fn store_auth_challenge(&self, challenge: &AuthenticationChallenge) -> Result<()> {
        let mut challenges = self.auth_challenges.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        challenges.insert(challenge.challenge.clone(), challenge.clone());
        Ok(())
    }

    async fn get_auth_challenge(&self, challenge: &str) -> Result<Option<AuthenticationChallenge>> {
        let challenges = self.auth_challenges.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(challenges.get(challenge).cloned())
    }

    async fn delete_auth_challenge(&self, id: Uuid) -> Result<()> {
        let mut challenges = self.auth_challenges.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        challenges.retain(|_, c| c.id != id);
        Ok(())
    }

    async fn cleanup_expired_challenges(&self) -> Result<u64> {
        let now = Utc::now();
        let mut count = 0;

        {
            let mut challenges = self.registration_challenges.write().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;
            let before = challenges.len();
            challenges.retain(|_, c| c.expires_at > now);
            count += (before - challenges.len()) as u64;
        }

        {
            let mut challenges = self.auth_challenges.write().map_err(|_| Error::Internal {
                message: "Lock poisoned".to_string(),
                request_id: None,
            })?;
            let before = challenges.len();
            challenges.retain(|_, c| c.expires_at > now);
            count += (before - challenges.len()) as u64;
        }

        Ok(count)
    }
}

/// Base64 URL-safe encoding
fn base64_url_encode(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_begin_registration() {
        let store = InMemoryWebAuthnStore::new();
        let manager = WebAuthnManager::new(store, WebAuthnConfig::default());

        let user_id = Uuid::now_v7();
        let (challenge, options) = manager
            .begin_registration(user_id, "user@example.com", "Test User")
            .await
            .unwrap();

        assert!(!challenge.challenge.is_empty());
        assert_eq!(options.rp.id, "localhost");
        assert_eq!(options.user.name, "user@example.com");
    }

    #[tokio::test]
    async fn test_begin_authentication() {
        let store = InMemoryWebAuthnStore::new();
        let manager = WebAuthnManager::new(store, WebAuthnConfig::default());

        let (challenge, options) = manager
            .begin_authentication(None)
            .await
            .unwrap();

        assert!(!challenge.challenge.is_empty());
        assert!(options.allow_credentials.is_empty());
    }

    #[tokio::test]
    async fn test_max_credentials() {
        let store = InMemoryWebAuthnStore::new();
        let config = WebAuthnConfig {
            max_credentials_per_user: 2,
            ..Default::default()
        };
        let manager = WebAuthnManager::new(store, config);

        let user_id = Uuid::now_v7();

        // Add two credentials directly to the store
        for i in 0..2 {
            let cred = WebAuthnCredential {
                id: Uuid::now_v7(),
                user_id,
                credential_id: format!("cred_{}", i),
                public_key: "test".to_string(),
                name: format!("Key {}", i),
                aaguid: None,
                sign_count: 0,
                credential_type: CredentialType::Platform,
                user_verified: true,
                backup_eligible: false,
                backed_up: false,
                transports: vec![],
                created_at: Utc::now(),
                last_used_at: None,
                is_primary: false,
                is_active: true,
            };
            manager.store.store_credential(&cred).await.unwrap();
        }

        // Should fail to start new registration
        let result = manager.begin_registration(user_id, "user", "User").await;
        assert!(result.is_err());
    }
}
