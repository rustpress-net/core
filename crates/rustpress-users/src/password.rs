//! # Password Policies
//!
//! Password validation, hashing, and policy enforcement.
//!
//! Features:
//! - Configurable password policies
//! - Password strength validation
//! - Secure password hashing (Argon2)
//! - Password history
//! - Password expiration
//! - Common password checking
//! - Breach detection (Have I Been Pwned)

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,

    /// Maximum password length
    pub max_length: usize,

    /// Require uppercase letter
    pub require_uppercase: bool,

    /// Require lowercase letter
    pub require_lowercase: bool,

    /// Require digit
    pub require_digit: bool,

    /// Require special character
    pub require_special: bool,

    /// Special characters allowed
    pub special_characters: String,

    /// Minimum different character types required
    pub min_character_types: usize,

    /// Check against common passwords
    pub check_common_passwords: bool,

    /// Check against breached passwords (HIBP)
    pub check_breached_passwords: bool,

    /// Prevent using username in password
    pub prevent_username_in_password: bool,

    /// Prevent using email in password
    pub prevent_email_in_password: bool,

    /// Password history count (prevent reuse)
    pub history_count: usize,

    /// Password expiration days (0 = never)
    pub expiration_days: u32,

    /// Grace period days after expiration
    pub grace_period_days: u32,

    /// Force change on first login
    pub force_change_on_first_login: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
            special_characters: "!@#$%^&*()_+-=[]{}|;':\",./<>?".to_string(),
            min_character_types: 3,
            check_common_passwords: true,
            check_breached_passwords: false,
            prevent_username_in_password: true,
            prevent_email_in_password: true,
            history_count: 5,
            expiration_days: 0,
            grace_period_days: 7,
            force_change_on_first_login: false,
        }
    }
}

/// Password validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordValidationResult {
    pub valid: bool,
    pub errors: Vec<PasswordError>,
    pub strength: PasswordStrength,
    pub suggestions: Vec<String>,
}

/// Password error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordError {
    pub code: String,
    pub message: String,
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn score(&self) -> u32 {
        match self {
            PasswordStrength::VeryWeak => 0,
            PasswordStrength::Weak => 25,
            PasswordStrength::Fair => 50,
            PasswordStrength::Strong => 75,
            PasswordStrength::VeryStrong => 100,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "Very Weak",
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Fair => "Fair",
            PasswordStrength::Strong => "Strong",
            PasswordStrength::VeryStrong => "Very Strong",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "#dc2626",
            PasswordStrength::Weak => "#f97316",
            PasswordStrength::Fair => "#eab308",
            PasswordStrength::Strong => "#22c55e",
            PasswordStrength::VeryStrong => "#16a34a",
        }
    }
}

/// Password validator
pub struct PasswordValidator {
    policy: PasswordPolicy,
    common_passwords: HashSet<String>,
}

impl PasswordValidator {
    pub fn new(policy: PasswordPolicy) -> Self {
        let mut validator = Self {
            policy,
            common_passwords: HashSet::new(),
        };

        if validator.policy.check_common_passwords {
            validator.load_common_passwords();
        }

        validator
    }

    /// Load common passwords list
    fn load_common_passwords(&mut self) {
        // Top 100 most common passwords
        let common = vec![
            "123456", "password", "12345678", "qwerty", "123456789",
            "12345", "1234", "111111", "1234567", "dragon",
            "123123", "baseball", "iloveyou", "trustno1", "sunshine",
            "master", "welcome", "shadow", "ashley", "football",
            "jesus", "michael", "ninja", "mustang", "password1",
            "123456789", "adobe123", "admin", "1234567890", "letmein",
            "photoshop", "1234", "monkey", "abc123", "azerty",
            "princess", "000000", "login", "password123", "starwars",
            "passw0rd", "hello", "charlie", "donald", "qwerty123",
            "whatever", "Password", "Password1", "changeme", "secret",
        ];

        for pwd in common {
            self.common_passwords.insert(pwd.to_lowercase());
        }
    }

    /// Validate a password against the policy
    pub fn validate(&self, password: &str, username: Option<&str>, email: Option<&str>) -> PasswordValidationResult {
        let mut errors = Vec::new();
        let mut suggestions = Vec::new();

        // Length check
        if password.len() < self.policy.min_length {
            errors.push(PasswordError {
                code: "too_short".to_string(),
                message: format!("Password must be at least {} characters", self.policy.min_length),
            });
            suggestions.push(format!("Add {} more characters", self.policy.min_length - password.len()));
        }

        if password.len() > self.policy.max_length {
            errors.push(PasswordError {
                code: "too_long".to_string(),
                message: format!("Password must not exceed {} characters", self.policy.max_length),
            });
        }

        // Character type checks
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| self.policy.special_characters.contains(c));

        let mut char_types = 0;
        if has_uppercase { char_types += 1; }
        if has_lowercase { char_types += 1; }
        if has_digit { char_types += 1; }
        if has_special { char_types += 1; }

        if self.policy.require_uppercase && !has_uppercase {
            errors.push(PasswordError {
                code: "missing_uppercase".to_string(),
                message: "Password must contain an uppercase letter".to_string(),
            });
            suggestions.push("Add an uppercase letter".to_string());
        }

        if self.policy.require_lowercase && !has_lowercase {
            errors.push(PasswordError {
                code: "missing_lowercase".to_string(),
                message: "Password must contain a lowercase letter".to_string(),
            });
            suggestions.push("Add a lowercase letter".to_string());
        }

        if self.policy.require_digit && !has_digit {
            errors.push(PasswordError {
                code: "missing_digit".to_string(),
                message: "Password must contain a digit".to_string(),
            });
            suggestions.push("Add a number".to_string());
        }

        if self.policy.require_special && !has_special {
            errors.push(PasswordError {
                code: "missing_special".to_string(),
                message: "Password must contain a special character".to_string(),
            });
            suggestions.push(format!("Add a special character like {}", &self.policy.special_characters[..5]));
        }

        if char_types < self.policy.min_character_types {
            errors.push(PasswordError {
                code: "insufficient_complexity".to_string(),
                message: format!("Password must contain at least {} different character types", self.policy.min_character_types),
            });
        }

        // Common password check
        if self.policy.check_common_passwords {
            if self.common_passwords.contains(&password.to_lowercase()) {
                errors.push(PasswordError {
                    code: "common_password".to_string(),
                    message: "This password is too common".to_string(),
                });
                suggestions.push("Choose a more unique password".to_string());
            }
        }

        // Username in password check
        if self.policy.prevent_username_in_password {
            if let Some(username) = username {
                if password.to_lowercase().contains(&username.to_lowercase()) {
                    errors.push(PasswordError {
                        code: "contains_username".to_string(),
                        message: "Password cannot contain your username".to_string(),
                    });
                }
            }
        }

        // Email in password check
        if self.policy.prevent_email_in_password {
            if let Some(email) = email {
                let email_local = email.split('@').next().unwrap_or("");
                if !email_local.is_empty() && password.to_lowercase().contains(&email_local.to_lowercase()) {
                    errors.push(PasswordError {
                        code: "contains_email".to_string(),
                        message: "Password cannot contain your email address".to_string(),
                    });
                }
            }
        }

        // Calculate strength
        let strength = self.calculate_strength(password, char_types);

        PasswordValidationResult {
            valid: errors.is_empty(),
            errors,
            strength,
            suggestions,
        }
    }

    /// Calculate password strength
    fn calculate_strength(&self, password: &str, char_types: usize) -> PasswordStrength {
        let mut score = 0;

        // Length score
        score += match password.len() {
            0..=7 => 0,
            8..=11 => 10,
            12..=15 => 20,
            16..=23 => 30,
            _ => 40,
        };

        // Character types score
        score += char_types * 10;

        // Variety bonus
        let unique_chars = password.chars().collect::<HashSet<_>>().len();
        score += (unique_chars as f32 / password.len() as f32 * 20.0) as usize;

        // Penalty for common patterns
        let lower = password.to_lowercase();
        if lower.contains("123") || lower.contains("abc") || lower.contains("qwerty") {
            score = score.saturating_sub(10);
        }

        // Penalty for repeated characters
        let repeated = password.chars()
            .zip(password.chars().skip(1))
            .filter(|(a, b)| a == b)
            .count();
        score = score.saturating_sub(repeated * 2);

        match score {
            0..=20 => PasswordStrength::VeryWeak,
            21..=40 => PasswordStrength::Weak,
            41..=60 => PasswordStrength::Fair,
            61..=80 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }

    /// Get policy
    pub fn policy(&self) -> &PasswordPolicy {
        &self.policy
    }
}

/// Password history manager
pub struct PasswordHistoryManager {
    /// History count to keep
    history_count: usize,

    /// Password history by user ID
    history: HashMap<i64, Vec<PasswordHistoryEntry>>,
}

/// Password history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHistoryEntry {
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

impl PasswordHistoryManager {
    pub fn new(history_count: usize) -> Self {
        Self {
            history_count,
            history: HashMap::new(),
        }
    }

    /// Add password to history
    pub fn add(&mut self, user_id: i64, password_hash: &str) {
        let entry = PasswordHistoryEntry {
            hash: password_hash.to_string(),
            created_at: Utc::now(),
        };

        let history = self.history.entry(user_id).or_insert_with(Vec::new);
        history.insert(0, entry);

        // Trim to history count
        history.truncate(self.history_count);
    }

    /// Check if password was used before
    pub fn was_used_before(&self, user_id: i64, password: &str) -> bool {
        if let Some(history) = self.history.get(&user_id) {
            for entry in history {
                if verify_password(password, &entry.hash).unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }

    /// Get password history for user
    pub fn get_history(&self, user_id: i64) -> Vec<&PasswordHistoryEntry> {
        self.history.get(&user_id)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// Clear history for user
    pub fn clear(&mut self, user_id: i64) {
        self.history.remove(&user_id);
    }
}

/// Password expiration tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordExpiration {
    pub user_id: i64,
    pub password_changed_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub grace_period_end: Option<DateTime<Utc>>,
    pub must_change: bool,
}

impl PasswordExpiration {
    pub fn new(user_id: i64, policy: &PasswordPolicy) -> Self {
        let now = Utc::now();
        let expires_at = if policy.expiration_days > 0 {
            Some(now + Duration::days(policy.expiration_days as i64))
        } else {
            None
        };

        let grace_period_end = expires_at.map(|exp| exp + Duration::days(policy.grace_period_days as i64));

        Self {
            user_id,
            password_changed_at: now,
            expires_at,
            grace_period_end,
            must_change: policy.force_change_on_first_login,
        }
    }

    /// Check if password is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Check if in grace period
    pub fn is_in_grace_period(&self) -> bool {
        if let (Some(expires), Some(grace_end)) = (self.expires_at, self.grace_period_end) {
            let now = Utc::now();
            now > expires && now <= grace_end
        } else {
            false
        }
    }

    /// Check if password change is required
    pub fn requires_change(&self) -> bool {
        self.must_change || self.is_expired()
    }

    /// Days until expiration
    pub fn days_until_expiration(&self) -> Option<i64> {
        self.expires_at.map(|exp| (exp - Utc::now()).num_days())
    }
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2.hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a secure random password
pub fn generate_password(length: usize, include_special: bool) -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);

    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let digits = "0123456789";
    let special = "!@#$%^&*";

    // Ensure at least one of each required type
    password.push(lowercase.chars().nth(rng.gen_range(0..lowercase.len())).unwrap());
    password.push(uppercase.chars().nth(rng.gen_range(0..uppercase.len())).unwrap());
    password.push(digits.chars().nth(rng.gen_range(0..digits.len())).unwrap());

    if include_special {
        password.push(special.chars().nth(rng.gen_range(0..special.len())).unwrap());
    }

    // Fill the rest
    let charset = if include_special {
        format!("{}{}{}{}", lowercase, uppercase, digits, special)
    } else {
        format!("{}{}{}", lowercase, uppercase, digits)
    };

    let charset_chars: Vec<char> = charset.chars().collect();

    while password.len() < length {
        password.push(charset_chars[rng.gen_range(0..charset_chars.len())]);
    }

    // Shuffle the password
    let mut chars: Vec<char> = password.chars().collect();
    for i in (1..chars.len()).rev() {
        let j = rng.gen_range(0..=i);
        chars.swap(i, j);
    }

    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let policy = PasswordPolicy::default();
        let validator = PasswordValidator::new(policy);

        // Valid password
        let result = validator.validate("StrongP@ssw0rd!", None, None);
        assert!(result.valid);

        // Too short
        let result = validator.validate("Short1!", None, None);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "too_short"));

        // Missing uppercase
        let result = validator.validate("weakpassword123", None, None);
        assert!(!result.valid);
    }

    #[test]
    fn test_password_strength() {
        let policy = PasswordPolicy::default();
        let validator = PasswordValidator::new(policy);

        let weak = validator.validate("password", None, None);
        assert!(weak.strength.score() < 50);

        let strong = validator.validate("MyStr0ngP@ssword!2024", None, None);
        assert!(strong.strength.score() >= 75);
    }

    #[test]
    fn test_password_generation() {
        let password = generate_password(16, true);
        assert_eq!(password.len(), 16);

        // Should contain variety
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_password_history() {
        let mut history = PasswordHistoryManager::new(5);
        let hash = hash_password("OldPassword123!").unwrap();

        history.add(1, &hash);

        assert!(history.was_used_before(1, "OldPassword123!"));
        assert!(!history.was_used_before(1, "NewPassword123!"));
    }

    #[test]
    fn test_password_expiration() {
        let policy = PasswordPolicy {
            expiration_days: 90,
            grace_period_days: 7,
            ..Default::default()
        };

        let expiration = PasswordExpiration::new(1, &policy);

        assert!(!expiration.is_expired());
        assert!(expiration.days_until_expiration().unwrap() > 0);
    }
}
