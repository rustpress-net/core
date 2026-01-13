//! Password hashing and validation.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};
use rustpress_core::error::{Error, Result, ValidationErrors};

/// Password hasher using Argon2
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash a password
    pub fn hash(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::Internal {
                message: format!("Failed to hash password: {}", e),
                request_id: None,
            })?;

        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| Error::Internal {
            message: format!("Invalid password hash format: {}", e),
            request_id: None,
        })?;

        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Password validation rules
#[derive(Debug, Clone)]
pub struct PasswordRules {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub special_chars: String,
}

impl Default for PasswordRules {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
            special_chars: "!@#$%^&*()_+-=[]{}|;:'\",.<>?/`~".to_string(),
        }
    }
}

impl PasswordRules {
    pub fn strict() -> Self {
        Self {
            min_length: 12,
            require_special: true,
            ..Default::default()
        }
    }

    pub fn relaxed() -> Self {
        Self {
            min_length: 6,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            ..Default::default()
        }
    }
}

/// Password validator
pub struct PasswordValidator {
    rules: PasswordRules,
}

impl PasswordValidator {
    pub fn new(rules: PasswordRules) -> Self {
        Self { rules }
    }

    /// Validate a password against the rules
    pub fn validate(&self, password: &str) -> Result<()> {
        let mut errors = ValidationErrors::new();

        // Length checks
        if password.len() < self.rules.min_length {
            errors.add_with_code(
                "password",
                format!("Password must be at least {} characters", self.rules.min_length),
                "PASSWORD_TOO_SHORT",
            );
        }

        if password.len() > self.rules.max_length {
            errors.add_with_code(
                "password",
                format!("Password must not exceed {} characters", self.rules.max_length),
                "PASSWORD_TOO_LONG",
            );
        }

        // Character requirement checks
        if self.rules.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.add_with_code(
                "password",
                "Password must contain at least one uppercase letter",
                "PASSWORD_NO_UPPERCASE",
            );
        }

        if self.rules.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.add_with_code(
                "password",
                "Password must contain at least one lowercase letter",
                "PASSWORD_NO_LOWERCASE",
            );
        }

        if self.rules.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.add_with_code(
                "password",
                "Password must contain at least one digit",
                "PASSWORD_NO_DIGIT",
            );
        }

        if self.rules.require_special
            && !password.chars().any(|c| self.rules.special_chars.contains(c))
        {
            errors.add_with_code(
                "password",
                "Password must contain at least one special character",
                "PASSWORD_NO_SPECIAL",
            );
        }

        errors.into_result(())
    }

    /// Check password strength (0-4 scale)
    pub fn strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;

        // Length
        if password.len() >= 8 {
            score += 1;
        }
        if password.len() >= 12 {
            score += 1;
        }
        if password.len() >= 16 {
            score += 1;
        }

        // Character variety
        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_ascii_digit()) {
            score += 1;
        }
        if password.chars().any(|c| self.rules.special_chars.contains(c)) {
            score += 1;
        }

        // Variety of characters
        let unique_chars: std::collections::HashSet<char> = password.chars().collect();
        if unique_chars.len() >= password.len() / 2 {
            score += 1;
        }

        match score {
            0..=2 => PasswordStrength::Weak,
            3..=4 => PasswordStrength::Fair,
            5..=6 => PasswordStrength::Good,
            _ => PasswordStrength::Strong,
        }
    }
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self::new(PasswordRules::default())
    }
}

/// Password strength level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrength {
    pub fn score(&self) -> u8 {
        match self {
            Self::Weak => 0,
            Self::Fair => 1,
            Self::Good => 2,
            Self::Strong => 3,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Weak => "Weak",
            Self::Fair => "Fair",
            Self::Good => "Good",
            Self::Strong => "Strong",
        }
    }

    pub fn is_acceptable(&self) -> bool {
        matches!(self, Self::Fair | Self::Good | Self::Strong)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = PasswordHasher::new();

        let password = "MySecurePassword123!";
        let hash = hasher.hash(password).unwrap();

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let validator = PasswordValidator::default();

        // Valid password
        assert!(validator.validate("SecurePass123").is_ok());

        // Too short
        assert!(validator.validate("Short1").is_err());

        // No uppercase
        assert!(validator.validate("nouppercase123").is_err());

        // No lowercase
        assert!(validator.validate("NOLOWERCASE123").is_err());

        // No digit
        assert!(validator.validate("NoDigitsHere").is_err());
    }

    #[test]
    fn test_password_strength() {
        let validator = PasswordValidator::default();

        assert_eq!(
            validator.strength("weak"),
            PasswordStrength::Weak
        );
        assert_eq!(
            validator.strength("Password1"),
            PasswordStrength::Good
        );
        assert_eq!(
            validator.strength("SecurePass123"),
            PasswordStrength::Good
        );
        assert_eq!(
            validator.strength("V3ryS3cur3P@ssw0rd!"),
            PasswordStrength::Strong
        );
    }

    #[test]
    fn test_strict_rules() {
        let validator = PasswordValidator::new(PasswordRules::strict());

        // Fails without special char
        assert!(validator.validate("SecurePass123").is_err());

        // Passes with special char
        assert!(validator.validate("SecurePass123!").is_ok());
    }

    #[test]
    fn test_relaxed_rules() {
        let validator = PasswordValidator::new(PasswordRules::relaxed());

        // Passes with just minimum length
        assert!(validator.validate("simple").is_ok());
    }
}
