//! # User Registration System
//!
//! Customizable user registration with flexible fields.
//!
//! Features:
//! - Customizable registration fields
//! - Field validation rules
//! - Email verification workflow
//! - Registration approval workflow
//! - Spam prevention

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Registration field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationFieldType {
    Text,
    Email,
    Password,
    Textarea,
    Select(Vec<SelectOption>),
    Checkbox,
    Radio(Vec<SelectOption>),
    Date,
    Phone,
    Url,
    Number,
    Hidden,
}

/// Select option for dropdowns and radios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub default: bool,
}

impl SelectOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            default: false,
        }
    }

    pub fn as_default(mut self) -> Self {
        self.default = true;
        self
    }
}

/// Registration field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationField {
    /// Field identifier
    pub id: String,

    /// Display label
    pub label: String,

    /// Field type
    pub field_type: RegistrationFieldType,

    /// Placeholder text
    pub placeholder: Option<String>,

    /// Help text shown below field
    pub help_text: Option<String>,

    /// Whether field is required
    pub required: bool,

    /// Field display order
    pub order: i32,

    /// Field is enabled
    pub enabled: bool,

    /// Field visibility
    pub visibility: FieldVisibility,

    /// Validation rules
    pub validation: FieldValidation,

    /// Default value
    pub default_value: Option<String>,

    /// CSS classes for styling
    pub css_classes: Vec<String>,

    /// Whether to save to user meta
    pub save_to_meta: bool,

    /// Meta key if saving to user meta
    pub meta_key: Option<String>,
}

/// Field visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldVisibility {
    /// Always visible
    Always,
    /// Visible only on registration form
    RegistrationOnly,
    /// Visible only on profile
    ProfileOnly,
    /// Admin only
    AdminOnly,
}

/// Field validation rules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldValidation {
    /// Minimum length
    pub min_length: Option<usize>,

    /// Maximum length
    pub max_length: Option<usize>,

    /// Regex pattern
    pub pattern: Option<String>,

    /// Custom error message
    pub error_message: Option<String>,

    /// Minimum value (for numbers)
    pub min_value: Option<f64>,

    /// Maximum value (for numbers)
    pub max_value: Option<f64>,

    /// Allowed file extensions (for file fields)
    pub allowed_extensions: Vec<String>,

    /// Must match another field (for password confirmation)
    pub must_match: Option<String>,

    /// Custom validator function name
    pub custom_validator: Option<String>,
}

impl RegistrationField {
    pub fn new(id: &str, label: &str, field_type: RegistrationFieldType) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            field_type,
            placeholder: None,
            help_text: None,
            required: false,
            order: 0,
            enabled: true,
            visibility: FieldVisibility::Always,
            validation: FieldValidation::default(),
            default_value: None,
            css_classes: Vec::new(),
            save_to_meta: false,
            meta_key: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn placeholder(mut self, text: &str) -> Self {
        self.placeholder = Some(text.to_string());
        self
    }

    pub fn help_text(mut self, text: &str) -> Self {
        self.help_text = Some(text.to_string());
        self
    }

    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn save_to_meta(mut self, meta_key: &str) -> Self {
        self.save_to_meta = true;
        self.meta_key = Some(meta_key.to_string());
        self
    }

    pub fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = validation;
        self
    }

    // Common field builders
    pub fn username() -> Self {
        Self::new("username", "Username", RegistrationFieldType::Text)
            .required()
            .placeholder("Choose a username")
            .validation(FieldValidation {
                min_length: Some(3),
                max_length: Some(60),
                pattern: Some(r"^[a-zA-Z0-9_-]+$".to_string()),
                error_message: Some("Username can only contain letters, numbers, underscores, and hyphens".to_string()),
                ..Default::default()
            })
            .order(1)
    }

    pub fn email() -> Self {
        Self::new("email", "Email Address", RegistrationFieldType::Email)
            .required()
            .placeholder("you@example.com")
            .order(2)
    }

    pub fn password() -> Self {
        Self::new("password", "Password", RegistrationFieldType::Password)
            .required()
            .placeholder("Choose a strong password")
            .validation(FieldValidation {
                min_length: Some(8),
                max_length: Some(128),
                error_message: Some("Password must be at least 8 characters".to_string()),
                ..Default::default()
            })
            .order(3)
    }

    pub fn password_confirm() -> Self {
        Self::new("password_confirm", "Confirm Password", RegistrationFieldType::Password)
            .required()
            .placeholder("Re-enter your password")
            .validation(FieldValidation {
                must_match: Some("password".to_string()),
                error_message: Some("Passwords do not match".to_string()),
                ..Default::default()
            })
            .order(4)
    }

    pub fn first_name() -> Self {
        Self::new("first_name", "First Name", RegistrationFieldType::Text)
            .placeholder("Your first name")
            .save_to_meta("first_name")
            .order(5)
    }

    pub fn last_name() -> Self {
        Self::new("last_name", "Last Name", RegistrationFieldType::Text)
            .placeholder("Your last name")
            .save_to_meta("last_name")
            .order(6)
    }

    pub fn display_name() -> Self {
        Self::new("display_name", "Display Name", RegistrationFieldType::Text)
            .placeholder("How you want to be known")
            .order(7)
    }

    pub fn website() -> Self {
        Self::new("website", "Website", RegistrationFieldType::Url)
            .placeholder("https://yoursite.com")
            .save_to_meta("url")
            .order(8)
    }

    pub fn bio() -> Self {
        Self::new("bio", "Bio", RegistrationFieldType::Textarea)
            .placeholder("Tell us about yourself")
            .validation(FieldValidation {
                max_length: Some(500),
                ..Default::default()
            })
            .save_to_meta("description")
            .order(9)
    }
}

/// Registration form configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationForm {
    /// Form ID
    pub id: String,

    /// Form name
    pub name: String,

    /// Form fields
    pub fields: Vec<RegistrationField>,

    /// Whether registration is enabled
    pub enabled: bool,

    /// Default role for new users
    pub default_role: String,

    /// Require email verification
    pub require_email_verification: bool,

    /// Require admin approval
    pub require_admin_approval: bool,

    /// Enable captcha
    pub enable_captcha: bool,

    /// Captcha type
    pub captcha_type: Option<CaptchaType>,

    /// Welcome email template
    pub welcome_email_template: Option<String>,

    /// Redirect URL after registration
    pub redirect_url: Option<String>,

    /// Terms and conditions URL
    pub terms_url: Option<String>,

    /// Privacy policy URL
    pub privacy_url: Option<String>,

    /// Registration limits
    pub limits: RegistrationLimits,
}

/// Captcha types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptchaType {
    ReCaptchaV2,
    ReCaptchaV3,
    HCaptcha,
    Turnstile,
    Simple,
}

/// Registration limits for spam prevention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationLimits {
    /// Max registrations per IP per day
    pub max_per_ip_daily: u32,

    /// Max registrations per email domain per day
    pub max_per_domain_daily: u32,

    /// Blocked email domains
    pub blocked_domains: Vec<String>,

    /// Blocked usernames
    pub blocked_usernames: Vec<String>,

    /// Minimum time between registrations from same IP (seconds)
    pub cooldown_seconds: u64,
}

impl Default for RegistrationLimits {
    fn default() -> Self {
        Self {
            max_per_ip_daily: 5,
            max_per_domain_daily: 10,
            blocked_domains: vec![
                "tempmail.com".to_string(),
                "throwaway.email".to_string(),
                "mailinator.com".to_string(),
            ],
            blocked_usernames: vec![
                "admin".to_string(),
                "administrator".to_string(),
                "root".to_string(),
                "system".to_string(),
            ],
            cooldown_seconds: 60,
        }
    }
}

impl Default for RegistrationForm {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Registration".to_string(),
            fields: vec![
                RegistrationField::username(),
                RegistrationField::email(),
                RegistrationField::password(),
                RegistrationField::password_confirm(),
            ],
            enabled: true,
            default_role: "subscriber".to_string(),
            require_email_verification: true,
            require_admin_approval: false,
            enable_captcha: false,
            captcha_type: None,
            welcome_email_template: None,
            redirect_url: None,
            terms_url: None,
            privacy_url: None,
            limits: RegistrationLimits::default(),
        }
    }
}

impl RegistrationForm {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn add_field(&mut self, field: RegistrationField) {
        self.fields.push(field);
        self.fields.sort_by_key(|f| f.order);
    }

    pub fn remove_field(&mut self, field_id: &str) {
        self.fields.retain(|f| f.id != field_id);
    }

    pub fn get_field(&self, field_id: &str) -> Option<&RegistrationField> {
        self.fields.iter().find(|f| f.id == field_id)
    }

    pub fn get_enabled_fields(&self) -> Vec<&RegistrationField> {
        self.fields.iter().filter(|f| f.enabled).collect()
    }

    pub fn get_required_fields(&self) -> Vec<&RegistrationField> {
        self.fields.iter().filter(|f| f.required && f.enabled).collect()
    }
}

/// Registration submission data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationSubmission {
    /// Submission ID
    pub id: Uuid,

    /// Form ID
    pub form_id: String,

    /// Field values
    pub values: HashMap<String, String>,

    /// Submission timestamp
    pub submitted_at: DateTime<Utc>,

    /// Client IP address
    pub ip_address: String,

    /// User agent
    pub user_agent: String,

    /// Referrer URL
    pub referrer: Option<String>,

    /// Submission status
    pub status: SubmissionStatus,

    /// Created user ID (if approved)
    pub user_id: Option<i64>,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Email verification token
    pub verification_token: Option<String>,

    /// Verification token expiry
    pub verification_expires: Option<DateTime<Utc>>,
}

/// Submission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionStatus {
    Pending,
    PendingVerification,
    PendingApproval,
    Approved,
    Rejected,
    Spam,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl RegistrationSubmission {
    pub fn new(form_id: &str, ip_address: &str, user_agent: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            form_id: form_id.to_string(),
            values: HashMap::new(),
            submitted_at: Utc::now(),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            referrer: None,
            status: SubmissionStatus::Pending,
            user_id: None,
            errors: Vec::new(),
            verification_token: None,
            verification_expires: None,
        }
    }

    pub fn set_value(&mut self, field: &str, value: &str) {
        self.values.insert(field.to_string(), value.to_string());
    }

    pub fn get_value(&self, field: &str) -> Option<&String> {
        self.values.get(field)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn add_error(&mut self, field: &str, message: &str, code: &str) {
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
            code: code.to_string(),
        });
    }
}

/// Registration validator
pub struct RegistrationValidator {
    form: RegistrationForm,
}

impl RegistrationValidator {
    pub fn new(form: RegistrationForm) -> Self {
        Self { form }
    }

    /// Validate a registration submission
    pub fn validate(&self, submission: &mut RegistrationSubmission) -> bool {
        submission.errors.clear();

        for field in self.form.get_enabled_fields() {
            // Clone value to avoid holding borrow on submission
            let value = submission.values.get(&field.id).cloned();

            // Check required
            if field.required {
                match &value {
                    None => {
                        submission.add_error(
                            &field.id,
                            &format!("{} is required", field.label),
                            "required",
                        );
                        continue;
                    }
                    Some(v) if v.is_empty() => {
                        submission.add_error(
                            &field.id,
                            &format!("{} is required", field.label),
                            "required",
                        );
                        continue;
                    }
                    _ => {}
                }
            }

            // Skip further validation if empty and not required
            let value = match value {
                Some(v) if !v.is_empty() => v,
                _ => continue,
            };

            // Validate based on field type
            self.validate_field_type(field, &value, submission);

            // Validate using validation rules
            self.validate_rules(field, &value, submission);
        }

        !submission.has_errors()
    }

    fn validate_field_type(
        &self,
        field: &RegistrationField,
        value: &str,
        submission: &mut RegistrationSubmission,
    ) {
        match &field.field_type {
            RegistrationFieldType::Email => {
                if !self.is_valid_email(value) {
                    submission.add_error(&field.id, "Invalid email address", "invalid_email");
                }
            }
            RegistrationFieldType::Url => {
                if !self.is_valid_url(value) {
                    submission.add_error(&field.id, "Invalid URL", "invalid_url");
                }
            }
            RegistrationFieldType::Phone => {
                if !self.is_valid_phone(value) {
                    submission.add_error(&field.id, "Invalid phone number", "invalid_phone");
                }
            }
            RegistrationFieldType::Number => {
                if value.parse::<f64>().is_err() {
                    submission.add_error(&field.id, "Must be a number", "invalid_number");
                }
            }
            RegistrationFieldType::Date => {
                if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
                    submission.add_error(&field.id, "Invalid date format", "invalid_date");
                }
            }
            _ => {}
        }
    }

    fn validate_rules(
        &self,
        field: &RegistrationField,
        value: &str,
        submission: &mut RegistrationSubmission,
    ) {
        let validation = &field.validation;

        // Min length
        if let Some(min) = validation.min_length {
            if value.len() < min {
                let msg = validation.error_message.clone()
                    .unwrap_or_else(|| format!("{} must be at least {} characters", field.label, min));
                submission.add_error(&field.id, &msg, "min_length");
            }
        }

        // Max length
        if let Some(max) = validation.max_length {
            if value.len() > max {
                let msg = validation.error_message.clone()
                    .unwrap_or_else(|| format!("{} must not exceed {} characters", field.label, max));
                submission.add_error(&field.id, &msg, "max_length");
            }
        }

        // Pattern
        if let Some(ref pattern) = validation.pattern {
            if let Ok(re) = regex::Regex::new(pattern) {
                if !re.is_match(value) {
                    let msg = validation.error_message.clone()
                        .unwrap_or_else(|| format!("{} has invalid format", field.label));
                    submission.add_error(&field.id, &msg, "pattern");
                }
            }
        }

        // Min value
        if let Some(min) = validation.min_value {
            if let Ok(num) = value.parse::<f64>() {
                if num < min {
                    let msg = format!("{} must be at least {}", field.label, min);
                    submission.add_error(&field.id, &msg, "min_value");
                }
            }
        }

        // Max value
        if let Some(max) = validation.max_value {
            if let Ok(num) = value.parse::<f64>() {
                if num > max {
                    let msg = format!("{} must not exceed {}", field.label, max);
                    submission.add_error(&field.id, &msg, "max_value");
                }
            }
        }

        // Must match
        if let Some(ref other_field) = validation.must_match {
            if let Some(other_value) = submission.values.get(other_field) {
                if value != other_value {
                    let msg = validation.error_message.clone()
                        .unwrap_or_else(|| format!("{} does not match", field.label));
                    submission.add_error(&field.id, &msg, "must_match");
                }
            }
        }
    }

    fn is_valid_email(&self, email: &str) -> bool {
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();
        email_regex.is_match(email)
    }

    fn is_valid_url(&self, url: &str) -> bool {
        url::Url::parse(url).is_ok()
    }

    fn is_valid_phone(&self, phone: &str) -> bool {
        let phone_regex = regex::Regex::new(r"^[\d\s\-\+\(\)]+$").unwrap();
        phone_regex.is_match(phone) && phone.chars().filter(|c| c.is_ascii_digit()).count() >= 7
    }
}

/// Registration manager
pub struct RegistrationManager {
    forms: HashMap<String, RegistrationForm>,
    submissions: HashMap<Uuid, RegistrationSubmission>,
    ip_counts: HashMap<String, (u32, DateTime<Utc>)>,
    domain_counts: HashMap<String, (u32, DateTime<Utc>)>,
}

impl Default for RegistrationManager {
    fn default() -> Self {
        let mut manager = Self {
            forms: HashMap::new(),
            submissions: HashMap::new(),
            ip_counts: HashMap::new(),
            domain_counts: HashMap::new(),
        };

        // Add default form
        manager.register_form(RegistrationForm::default());

        manager
    }
}

impl RegistrationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a form
    pub fn register_form(&mut self, form: RegistrationForm) {
        self.forms.insert(form.id.clone(), form);
    }

    /// Get a form by ID
    pub fn get_form(&self, id: &str) -> Option<&RegistrationForm> {
        self.forms.get(id)
    }

    /// Get default form
    pub fn get_default_form(&self) -> Option<&RegistrationForm> {
        self.forms.get("default")
    }

    /// Check registration limits
    pub fn check_limits(&mut self, ip: &str, email: &str, form: &RegistrationForm) -> Result<(), String> {
        let limits = &form.limits;
        let now = Utc::now();

        // Check IP limit
        if let Some((count, last_reset)) = self.ip_counts.get(ip) {
            if now.signed_duration_since(*last_reset).num_days() == 0 {
                if *count >= limits.max_per_ip_daily {
                    return Err("Too many registrations from your IP address today".to_string());
                }
            }
        }

        // Extract domain from email
        if let Some(domain) = email.split('@').nth(1) {
            // Check blocked domains
            if limits.blocked_domains.iter().any(|d| domain.to_lowercase() == d.to_lowercase()) {
                return Err("Registration from this email domain is not allowed".to_string());
            }

            // Check domain limit
            if let Some((count, last_reset)) = self.domain_counts.get(domain) {
                if now.signed_duration_since(*last_reset).num_days() == 0 {
                    if *count >= limits.max_per_domain_daily {
                        return Err("Too many registrations from this email domain today".to_string());
                    }
                }
            }
        }

        Ok(())
    }

    /// Record a registration attempt
    pub fn record_attempt(&mut self, ip: &str, email: &str) {
        let now = Utc::now();

        // Update IP count
        let (count, last_reset) = self.ip_counts.entry(ip.to_string())
            .or_insert((0, now));

        if now.signed_duration_since(*last_reset).num_days() > 0 {
            *count = 1;
            *last_reset = now;
        } else {
            *count += 1;
        }

        // Update domain count
        if let Some(domain) = email.split('@').nth(1) {
            let (count, last_reset) = self.domain_counts.entry(domain.to_string())
                .or_insert((0, now));

            if now.signed_duration_since(*last_reset).num_days() > 0 {
                *count = 1;
                *last_reset = now;
            } else {
                *count += 1;
            }
        }
    }

    /// Process a registration submission
    pub fn process_submission(&mut self, mut submission: RegistrationSubmission) -> Result<RegistrationSubmission, String> {
        let form = self.forms.get(&submission.form_id)
            .ok_or_else(|| "Form not found".to_string())?
            .clone();

        if !form.enabled {
            return Err("Registration is currently disabled".to_string());
        }

        // Check username against blocked list
        if let Some(username) = submission.values.get("username") {
            if form.limits.blocked_usernames.iter()
                .any(|u| username.to_lowercase() == u.to_lowercase())
            {
                return Err("This username is not available".to_string());
            }
        }

        // Check limits
        if let Some(email) = submission.values.get("email") {
            self.check_limits(&submission.ip_address, email, &form)?;
        }

        // Validate submission
        let validator = RegistrationValidator::new(form.clone());
        if !validator.validate(&mut submission) {
            submission.status = SubmissionStatus::Pending;
            return Ok(submission);
        }

        // Determine next status based on form settings
        if form.require_email_verification {
            submission.status = SubmissionStatus::PendingVerification;
            submission.verification_token = Some(Uuid::new_v4().to_string());
            submission.verification_expires = Some(Utc::now() + chrono::Duration::hours(24));
        } else if form.require_admin_approval {
            submission.status = SubmissionStatus::PendingApproval;
        } else {
            submission.status = SubmissionStatus::Approved;
        }

        // Record the attempt
        if let Some(email) = submission.values.get("email") {
            self.record_attempt(&submission.ip_address, email);
        }

        // Store submission
        self.submissions.insert(submission.id, submission.clone());

        Ok(submission)
    }

    /// Verify email
    pub fn verify_email(&mut self, token: &str) -> Result<&RegistrationSubmission, String> {
        let submission = self.submissions.values_mut()
            .find(|s| s.verification_token.as_deref() == Some(token))
            .ok_or_else(|| "Invalid verification token".to_string())?;

        if let Some(expires) = submission.verification_expires {
            if Utc::now() > expires {
                return Err("Verification token has expired".to_string());
            }
        }

        // Check if form requires admin approval
        let form = self.forms.get(&submission.form_id);
        if form.map(|f| f.require_admin_approval).unwrap_or(false) {
            submission.status = SubmissionStatus::PendingApproval;
        } else {
            submission.status = SubmissionStatus::Approved;
        }

        submission.verification_token = None;
        submission.verification_expires = None;

        Ok(submission)
    }

    /// Approve a registration
    pub fn approve(&mut self, submission_id: Uuid, user_id: i64) -> Result<(), String> {
        let submission = self.submissions.get_mut(&submission_id)
            .ok_or_else(|| "Submission not found".to_string())?;

        submission.status = SubmissionStatus::Approved;
        submission.user_id = Some(user_id);

        Ok(())
    }

    /// Reject a registration
    pub fn reject(&mut self, submission_id: Uuid) -> Result<(), String> {
        let submission = self.submissions.get_mut(&submission_id)
            .ok_or_else(|| "Submission not found".to_string())?;

        submission.status = SubmissionStatus::Rejected;

        Ok(())
    }

    /// Mark as spam
    pub fn mark_spam(&mut self, submission_id: Uuid) -> Result<(), String> {
        let submission = self.submissions.get_mut(&submission_id)
            .ok_or_else(|| "Submission not found".to_string())?;

        submission.status = SubmissionStatus::Spam;

        Ok(())
    }

    /// Get pending submissions
    pub fn get_pending(&self) -> Vec<&RegistrationSubmission> {
        self.submissions.values()
            .filter(|s| matches!(s.status, SubmissionStatus::PendingApproval))
            .collect()
    }

    /// Get submission by ID
    pub fn get_submission(&self, id: &Uuid) -> Option<&RegistrationSubmission> {
        self.submissions.get(id)
    }
}

/// Email verification token generator
pub struct VerificationToken;

impl VerificationToken {
    /// Generate a new verification token
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate verification URL
    pub fn url(base_url: &str, token: &str) -> String {
        format!("{}?action=verify_email&token={}", base_url, urlencoding::encode(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_field() {
        let field = RegistrationField::username();
        assert_eq!(field.id, "username");
        assert!(field.required);
    }

    #[test]
    fn test_registration_form() {
        let mut form = RegistrationForm::default();
        assert_eq!(form.fields.len(), 4);

        form.add_field(RegistrationField::first_name());
        assert_eq!(form.fields.len(), 5);
    }

    #[test]
    fn test_validation() {
        let form = RegistrationForm::default();
        let validator = RegistrationValidator::new(form);

        let mut submission = RegistrationSubmission::new("default", "127.0.0.1", "Test Agent");
        submission.set_value("username", "testuser");
        submission.set_value("email", "test@example.com");
        submission.set_value("password", "strongpassword123");
        submission.set_value("password_confirm", "strongpassword123");

        assert!(validator.validate(&mut submission));
        assert!(!submission.has_errors());
    }

    #[test]
    fn test_validation_errors() {
        let form = RegistrationForm::default();
        let validator = RegistrationValidator::new(form);

        let mut submission = RegistrationSubmission::new("default", "127.0.0.1", "Test Agent");
        submission.set_value("username", "ab"); // Too short
        submission.set_value("email", "invalid-email");
        submission.set_value("password", "short");
        submission.set_value("password_confirm", "different");

        assert!(!validator.validate(&mut submission));
        assert!(submission.has_errors());
        assert!(submission.errors.len() >= 3);
    }

    #[test]
    fn test_registration_manager() {
        let mut manager = RegistrationManager::new();

        let mut submission = RegistrationSubmission::new("default", "127.0.0.1", "Test Agent");
        submission.set_value("username", "newuser");
        submission.set_value("email", "new@example.com");
        submission.set_value("password", "securepass123");
        submission.set_value("password_confirm", "securepass123");

        let result = manager.process_submission(submission);
        assert!(result.is_ok());
    }
}
