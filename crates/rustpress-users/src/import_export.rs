//! # User Import/Export
//!
//! User data import and export functionality.
//!
//! Features:
//! - CSV import/export
//! - JSON export
//! - Field mapping
//! - Validation during import
//! - Duplicate handling

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Json,
    Xml,
}

impl ExportFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Xml => "xml",
        }
    }

    pub fn content_type(&self) -> &str {
        match self {
            Self::Csv => "text/csv",
            Self::Json => "application/json",
            Self::Xml => "application/xml",
        }
    }
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,

    /// Fields to export
    pub fields: Vec<String>,

    /// Include user meta
    pub include_meta: bool,

    /// Meta keys to include (empty = all)
    pub meta_keys: Vec<String>,

    /// Filter by role
    pub role_filter: Option<String>,

    /// Filter by user IDs
    pub user_ids: Vec<i64>,

    /// Include header row (CSV)
    pub include_header: bool,

    /// Date format
    pub date_format: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Csv,
            fields: vec![
                "id".to_string(),
                "username".to_string(),
                "email".to_string(),
                "display_name".to_string(),
                "first_name".to_string(),
                "last_name".to_string(),
                "role".to_string(),
                "registered".to_string(),
            ],
            include_meta: false,
            meta_keys: Vec::new(),
            role_filter: None,
            user_ids: Vec::new(),
            include_header: true,
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
        }
    }
}

impl ExportConfig {
    pub fn new(format: ExportFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = fields;
        self
    }

    pub fn with_meta(mut self, keys: Vec<String>) -> Self {
        self.include_meta = true;
        self.meta_keys = keys;
        self
    }

    pub fn filter_role(mut self, role: &str) -> Self {
        self.role_filter = Some(role.to_string());
        self
    }

    pub fn filter_users(mut self, ids: Vec<i64>) -> Self {
        self.user_ids = ids;
        self
    }
}

/// Exportable user data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: String,
    pub url: String,
    pub description: String,
    pub roles: Vec<String>,
    pub registered: DateTime<Utc>,
    pub status: String,
    pub meta: HashMap<String, serde_json::Value>,
}

/// User exporter
pub struct UserExporter;

impl UserExporter {
    /// Export users to CSV format
    pub fn to_csv(users: &[ExportableUser], config: &ExportConfig) -> String {
        let mut output = String::new();

        // Header
        if config.include_header {
            let headers: Vec<&str> = config.fields.iter()
                .map(|f| f.as_str())
                .collect();
            output.push_str(&headers.join(","));
            output.push('\n');
        }

        // Data rows
        for user in users {
            let values: Vec<String> = config.fields.iter()
                .map(|field| Self::get_field_value(user, field, config))
                .map(|v| Self::escape_csv(&v))
                .collect();
            output.push_str(&values.join(","));
            output.push('\n');
        }

        output
    }

    /// Export users to JSON format
    pub fn to_json(users: &[ExportableUser], config: &ExportConfig) -> String {
        let filtered: Vec<serde_json::Value> = users.iter()
            .map(|user| {
                let mut obj = serde_json::Map::new();
                for field in &config.fields {
                    let value = Self::get_field_value(user, field, config);
                    obj.insert(field.clone(), serde_json::Value::String(value));
                }
                if config.include_meta {
                    let meta_value = if config.meta_keys.is_empty() {
                        serde_json::to_value(&user.meta).unwrap_or_default()
                    } else {
                        let filtered_meta: HashMap<_, _> = user.meta.iter()
                            .filter(|(k, _)| config.meta_keys.contains(k))
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        serde_json::to_value(&filtered_meta).unwrap_or_default()
                    };
                    obj.insert("meta".to_string(), meta_value);
                }
                serde_json::Value::Object(obj)
            })
            .collect();

        serde_json::to_string_pretty(&filtered).unwrap_or_default()
    }

    /// Get field value from user
    fn get_field_value(user: &ExportableUser, field: &str, config: &ExportConfig) -> String {
        match field {
            "id" => user.id.to_string(),
            "username" => user.username.clone(),
            "email" => user.email.clone(),
            "display_name" => user.display_name.clone(),
            "first_name" => user.first_name.clone(),
            "last_name" => user.last_name.clone(),
            "nickname" => user.nickname.clone(),
            "url" => user.url.clone(),
            "description" => user.description.clone(),
            "role" | "roles" => user.roles.join(", "),
            "registered" => user.registered.format(&config.date_format).to_string(),
            "status" => user.status.clone(),
            _ => {
                // Check meta
                user.meta.get(field)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default()
            }
        }
    }

    /// Escape CSV value
    fn escape_csv(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
}

// ============================================================================
// Import
// ============================================================================

/// Import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Field mapping (file column -> user field)
    pub field_mapping: HashMap<String, String>,

    /// Default role for imported users
    pub default_role: String,

    /// Send welcome email
    pub send_welcome_email: bool,

    /// Generate passwords
    pub generate_passwords: bool,

    /// How to handle duplicates
    pub duplicate_handling: DuplicateHandling,

    /// Skip invalid rows
    pub skip_invalid: bool,

    /// Dry run (validate only)
    pub dry_run: bool,

    /// Update existing users
    pub update_existing: bool,

    /// Required fields
    pub required_fields: Vec<String>,
}

/// How to handle duplicate users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateHandling {
    Skip,
    Update,
    Error,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            field_mapping: HashMap::new(),
            default_role: "subscriber".to_string(),
            send_welcome_email: true,
            generate_passwords: true,
            duplicate_handling: DuplicateHandling::Skip,
            skip_invalid: true,
            dry_run: false,
            update_existing: false,
            required_fields: vec!["username".to_string(), "email".to_string()],
        }
    }
}

impl ImportConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map_field(&mut self, source: &str, target: &str) {
        self.field_mapping.insert(source.to_string(), target.to_string());
    }

    pub fn with_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.field_mapping = mapping;
        self
    }
}

/// Importable user data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportableUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub role: Option<String>,
    pub meta: HashMap<String, String>,
}

impl ImportableUser {
    pub fn new() -> Self {
        Self {
            username: None,
            email: None,
            password: None,
            display_name: None,
            first_name: None,
            last_name: None,
            nickname: None,
            url: None,
            description: None,
            role: None,
            meta: HashMap::new(),
        }
    }

    pub fn set_field(&mut self, field: &str, value: &str) {
        let value = value.trim().to_string();
        if value.is_empty() {
            return;
        }

        match field {
            "username" => self.username = Some(value),
            "email" => self.email = Some(value),
            "password" => self.password = Some(value),
            "display_name" => self.display_name = Some(value),
            "first_name" => self.first_name = Some(value),
            "last_name" => self.last_name = Some(value),
            "nickname" => self.nickname = Some(value),
            "url" => self.url = Some(value),
            "description" => self.description = Some(value),
            "role" => self.role = Some(value),
            _ => { self.meta.insert(field.to_string(), value); }
        }
    }
}

impl Default for ImportableUser {
    fn default() -> Self {
        Self::new()
    }
}

/// Import validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportValidationError {
    pub row: usize,
    pub field: String,
    pub message: String,
    pub value: Option<String>,
}

/// Import result for a single user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportUserResult {
    Created { user_id: i64 },
    Updated { user_id: i64 },
    Skipped { reason: String },
    Error { errors: Vec<ImportValidationError> },
}

/// Overall import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub total_rows: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: usize,
    pub results: Vec<(usize, ImportUserResult)>,
    pub validation_errors: Vec<ImportValidationError>,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            total_rows: 0,
            created: 0,
            updated: 0,
            skipped: 0,
            errors: 0,
            results: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    pub fn add_result(&mut self, row: usize, result: ImportUserResult) {
        match &result {
            ImportUserResult::Created { .. } => self.created += 1,
            ImportUserResult::Updated { .. } => self.updated += 1,
            ImportUserResult::Skipped { .. } => self.skipped += 1,
            ImportUserResult::Error { errors } => {
                self.errors += 1;
                for error in errors {
                    self.validation_errors.push(error.clone());
                }
            }
        }
        self.results.push((row, result));
    }

    pub fn is_success(&self) -> bool {
        self.errors == 0
    }
}

impl Default for ImportResult {
    fn default() -> Self {
        Self::new()
    }
}

/// User importer
pub struct UserImporter {
    config: ImportConfig,
    existing_usernames: Vec<String>,
    existing_emails: Vec<String>,
}

impl UserImporter {
    pub fn new(config: ImportConfig) -> Self {
        Self {
            config,
            existing_usernames: Vec::new(),
            existing_emails: Vec::new(),
        }
    }

    /// Set existing users for duplicate checking
    pub fn set_existing_users(&mut self, usernames: Vec<String>, emails: Vec<String>) {
        self.existing_usernames = usernames.into_iter().map(|s| s.to_lowercase()).collect();
        self.existing_emails = emails.into_iter().map(|s| s.to_lowercase()).collect();
    }

    /// Parse CSV content
    pub fn parse_csv(&self, content: &str) -> Result<Vec<ImportableUser>, String> {
        let mut users = Vec::new();
        let mut lines = content.lines();

        // Parse header
        let header = lines.next()
            .ok_or_else(|| "Empty CSV file".to_string())?;
        let columns: Vec<&str> = Self::parse_csv_line(header);

        // Parse data rows
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let values = Self::parse_csv_line(line);
            let mut user = ImportableUser::new();

            for (i, value) in values.iter().enumerate() {
                if let Some(column) = columns.get(i) {
                    // Apply field mapping
                    let target_field = self.config.field_mapping
                        .get(*column)
                        .map(|s| s.as_str())
                        .unwrap_or(*column);

                    user.set_field(target_field, value);
                }
            }

            users.push(user);
        }

        Ok(users)
    }

    /// Parse a single CSV line
    fn parse_csv_line(line: &str) -> Vec<&str> {
        let mut fields = Vec::new();
        let mut current_start = 0;
        let mut in_quotes = false;
        let chars: Vec<char> = line.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    let field = &line[current_start..i];
                    fields.push(Self::unescape_csv(field));
                    current_start = i + 1;
                }
                _ => {}
            }
        }

        // Add last field
        let field = &line[current_start..];
        fields.push(Self::unescape_csv(field));

        fields
    }

    /// Unescape CSV value
    fn unescape_csv(value: &str) -> &str {
        let trimmed = value.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            &trimmed[1..trimmed.len()-1]
        } else {
            trimmed
        }
    }

    /// Validate a user before import
    pub fn validate(&self, user: &ImportableUser, row: usize) -> Vec<ImportValidationError> {
        let mut errors = Vec::new();

        // Check required fields
        for field in &self.config.required_fields {
            let value = match field.as_str() {
                "username" => &user.username,
                "email" => &user.email,
                _ => &None,
            };

            if value.is_none() || value.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                errors.push(ImportValidationError {
                    row,
                    field: field.clone(),
                    message: format!("{} is required", field),
                    value: None,
                });
            }
        }

        // Validate email format
        if let Some(ref email) = user.email {
            if !Self::is_valid_email(email) {
                errors.push(ImportValidationError {
                    row,
                    field: "email".to_string(),
                    message: "Invalid email format".to_string(),
                    value: Some(email.clone()),
                });
            }
        }

        // Validate username format
        if let Some(ref username) = user.username {
            if !Self::is_valid_username(username) {
                errors.push(ImportValidationError {
                    row,
                    field: "username".to_string(),
                    message: "Invalid username format".to_string(),
                    value: Some(username.clone()),
                });
            }
        }

        // Check duplicates
        if let Some(ref username) = user.username {
            if self.existing_usernames.contains(&username.to_lowercase()) {
                match self.config.duplicate_handling {
                    DuplicateHandling::Error => {
                        errors.push(ImportValidationError {
                            row,
                            field: "username".to_string(),
                            message: "Username already exists".to_string(),
                            value: Some(username.clone()),
                        });
                    }
                    _ => {}
                }
            }
        }

        if let Some(ref email) = user.email {
            if self.existing_emails.contains(&email.to_lowercase()) {
                match self.config.duplicate_handling {
                    DuplicateHandling::Error => {
                        errors.push(ImportValidationError {
                            row,
                            field: "email".to_string(),
                            message: "Email already exists".to_string(),
                            value: Some(email.clone()),
                        });
                    }
                    _ => {}
                }
            }
        }

        errors
    }

    fn is_valid_email(email: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        re.is_match(email)
    }

    fn is_valid_username(username: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z0-9_-]{3,60}$").unwrap();
        re.is_match(username)
    }

    /// Check if user is duplicate
    pub fn is_duplicate(&self, user: &ImportableUser) -> bool {
        if let Some(ref username) = user.username {
            if self.existing_usernames.contains(&username.to_lowercase()) {
                return true;
            }
        }
        if let Some(ref email) = user.email {
            if self.existing_emails.contains(&email.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

/// Generate secure random password
pub fn generate_password(length: usize) -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_export() {
        let users = vec![
            ExportableUser {
                id: 1,
                username: "john".to_string(),
                email: "john@example.com".to_string(),
                display_name: "John Doe".to_string(),
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                nickname: "johnny".to_string(),
                url: "".to_string(),
                description: "".to_string(),
                roles: vec!["editor".to_string()],
                registered: Utc::now(),
                status: "active".to_string(),
                meta: HashMap::new(),
            }
        ];

        let config = ExportConfig::new(ExportFormat::Csv)
            .with_fields(vec!["username".to_string(), "email".to_string()]);

        let csv = UserExporter::to_csv(&users, &config);
        assert!(csv.contains("username,email"));
        assert!(csv.contains("john,john@example.com"));
    }

    #[test]
    fn test_csv_parse() {
        let importer = UserImporter::new(ImportConfig::default());
        let csv = "username,email\njohn,john@example.com";

        let users = importer.parse_csv(csv).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, Some("john".to_string()));
    }

    #[test]
    fn test_validation() {
        let importer = UserImporter::new(ImportConfig::default());

        let valid_user = ImportableUser {
            username: Some("validuser".to_string()),
            email: Some("valid@example.com".to_string()),
            ..Default::default()
        };

        let errors = importer.validate(&valid_user, 1);
        assert!(errors.is_empty());

        let invalid_user = ImportableUser {
            username: Some("ab".to_string()), // Too short
            email: Some("invalid-email".to_string()),
            ..Default::default()
        };

        let errors = importer.validate(&invalid_user, 1);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_duplicate_detection() {
        let mut importer = UserImporter::new(ImportConfig::default());
        importer.set_existing_users(
            vec!["existing_user".to_string()],
            vec!["existing@example.com".to_string()]
        );

        let user = ImportableUser {
            username: Some("existing_user".to_string()),
            email: Some("new@example.com".to_string()),
            ..Default::default()
        };

        assert!(importer.is_duplicate(&user));
    }

    #[test]
    fn test_password_generation() {
        let password = generate_password(16);
        assert_eq!(password.len(), 16);
    }
}
