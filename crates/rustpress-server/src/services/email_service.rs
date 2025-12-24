//! Email service for sending transactional emails.
//!
//! Supports SMTP transport and templated emails for:
//! - Password reset
//! - Email verification
//! - Welcome emails
//! - Notification emails

use handlebars::Handlebars;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Email service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP host
    pub smtp_host: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: Option<String>,
    /// SMTP password
    pub smtp_password: Option<String>,
    /// Use TLS
    pub smtp_tls: bool,
    /// Default from address
    pub from_email: String,
    /// Default from name
    pub from_name: String,
    /// Site name (for templates)
    pub site_name: String,
    /// Site URL (for templates)
    pub site_url: String,
    /// Whether email is enabled
    pub enabled: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: None,
            smtp_password: None,
            smtp_tls: true,
            from_email: "noreply@localhost".to_string(),
            from_name: "RustPress".to_string(),
            site_name: "RustPress".to_string(),
            site_url: "http://localhost".to_string(),
            enabled: false,
        }
    }
}

/// Email template types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmailTemplate {
    PasswordReset,
    EmailVerification,
    Welcome,
    NewComment,
    CommentApproved,
    PostPublished,
    AccountDeactivated,
    SecurityAlert,
}

impl EmailTemplate {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::PasswordReset => "Reset Your Password",
            Self::EmailVerification => "Verify Your Email Address",
            Self::Welcome => "Welcome to {{site_name}}",
            Self::NewComment => "New Comment on Your Post",
            Self::CommentApproved => "Your Comment Has Been Approved",
            Self::PostPublished => "Your Post Has Been Published",
            Self::AccountDeactivated => "Your Account Has Been Deactivated",
            Self::SecurityAlert => "Security Alert for Your Account",
        }
    }

    pub fn template_html(&self) -> &'static str {
        match self {
            Self::PasswordReset => include_str!("../templates/email/password_reset.html"),
            Self::EmailVerification => include_str!("../templates/email/email_verification.html"),
            Self::Welcome => include_str!("../templates/email/welcome.html"),
            Self::NewComment => include_str!("../templates/email/new_comment.html"),
            Self::CommentApproved => include_str!("../templates/email/comment_approved.html"),
            Self::PostPublished => include_str!("../templates/email/post_published.html"),
            Self::AccountDeactivated => include_str!("../templates/email/account_deactivated.html"),
            Self::SecurityAlert => include_str!("../templates/email/security_alert.html"),
        }
    }
}

/// Email send result
#[derive(Debug)]
pub struct EmailResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

/// Email service error
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Email service not configured")]
    NotConfigured,
    #[error("Email service disabled")]
    Disabled,
    #[error("SMTP error: {0}")]
    SmtpError(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),
}

/// Email service for sending transactional emails
pub struct EmailService {
    config: Arc<RwLock<EmailConfig>>,
    templates: Arc<RwLock<Handlebars<'static>>>,
    transport: Arc<RwLock<Option<AsyncSmtpTransport<Tokio1Executor>>>>,
}

impl EmailService {
    /// Create a new email service
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        Self {
            config: Arc::new(RwLock::new(EmailConfig::default())),
            templates: Arc::new(RwLock::new(handlebars)),
            transport: Arc::new(RwLock::new(None)),
        }
    }

    /// Configure the email service
    pub async fn configure(&self, config: EmailConfig) -> Result<(), EmailError> {
        // Build SMTP transport
        let transport = if config.enabled {
            let builder = if config.smtp_tls {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
                    .map_err(|e| EmailError::SmtpError(e.to_string()))?
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
            };

            let mut builder = builder.port(config.smtp_port);

            if let (Some(username), Some(password)) = (&config.smtp_username, &config.smtp_password) {
                builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
            }

            Some(builder.build())
        } else {
            None
        };

        // Register default templates
        {
            let mut templates = self.templates.write().await;
            for template in [
                EmailTemplate::PasswordReset,
                EmailTemplate::EmailVerification,
                EmailTemplate::Welcome,
                EmailTemplate::NewComment,
                EmailTemplate::CommentApproved,
                EmailTemplate::PostPublished,
                EmailTemplate::AccountDeactivated,
                EmailTemplate::SecurityAlert,
            ] {
                let name = format!("{:?}", template);
                if let Err(e) = templates.register_template_string(&name, template.template_html()) {
                    tracing::warn!("Failed to register template {}: {}", name, e);
                }
            }
        }

        *self.config.write().await = config;
        *self.transport.write().await = transport;

        Ok(())
    }

    /// Check if email service is configured and enabled
    pub async fn is_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.enabled && self.transport.read().await.is_some()
    }

    /// Send an email using a template
    pub async fn send_template(
        &self,
        template: EmailTemplate,
        to_email: &str,
        to_name: Option<&str>,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<EmailResult, EmailError> {
        let config = self.config.read().await;

        if !config.enabled {
            return Err(EmailError::Disabled);
        }

        // Build template data with site info
        let mut template_data = data;
        template_data.insert("site_name".to_string(), serde_json::json!(config.site_name));
        template_data.insert("site_url".to_string(), serde_json::json!(config.site_url));
        template_data.insert("current_year".to_string(), serde_json::json!(chrono::Utc::now().format("%Y").to_string()));

        // Render subject
        let subject = template.subject().replace("{{site_name}}", &config.site_name);

        // Render body
        let template_name = format!("{:?}", template);
        let body = {
            let templates = self.templates.read().await;
            templates.render(&template_name, &template_data)
                .map_err(|e| EmailError::TemplateError(e.to_string()))?
        };

        drop(config);

        // Send email
        self.send_raw(to_email, to_name, &subject, &body).await
    }

    /// Send a raw email
    pub async fn send_raw(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        html_body: &str,
    ) -> Result<EmailResult, EmailError> {
        let config = self.config.read().await;

        if !config.enabled {
            return Err(EmailError::Disabled);
        }

        let transport = self.transport.read().await;
        let transport = transport.as_ref().ok_or(EmailError::NotConfigured)?;

        // Parse addresses
        let from_mailbox: Mailbox = format!("{} <{}>", config.from_name, config.from_email)
            .parse()
            .map_err(|e| EmailError::InvalidEmail(format!("Invalid from address: {}", e)))?;

        let to_mailbox: Mailbox = if let Some(name) = to_name {
            format!("{} <{}>", name, to_email)
        } else {
            to_email.to_string()
        }
        .parse()
        .map_err(|e| EmailError::InvalidEmail(format!("Invalid to address: {}", e)))?;

        // Build message
        let message = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| EmailError::SmtpError(format!("Failed to build message: {}", e)))?;

        drop(config);

        // Send
        match transport.send(message).await {
            Ok(response) => {
                tracing::info!("Email sent to {}: {:?}", to_email, response);
                let message_id = response.message().collect::<Vec<_>>().join(" ");
                Ok(EmailResult {
                    success: true,
                    message_id: Some(message_id),
                    error: None,
                })
            }
            Err(e) => {
                tracing::error!("Failed to send email to {}: {}", to_email, e);
                Ok(EmailResult {
                    success: false,
                    message_id: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Send password reset email
    pub async fn send_password_reset(
        &self,
        email: &str,
        name: Option<&str>,
        reset_token: &str,
    ) -> Result<EmailResult, EmailError> {
        let config = self.config.read().await;
        let reset_url = format!("{}/reset-password?token={}", config.site_url, reset_token);
        drop(config);

        let mut data = HashMap::new();
        data.insert("name".to_string(), serde_json::json!(name.unwrap_or("User")));
        data.insert("reset_url".to_string(), serde_json::json!(reset_url));
        data.insert("reset_token".to_string(), serde_json::json!(reset_token));
        data.insert("expires_hours".to_string(), serde_json::json!(24));

        self.send_template(EmailTemplate::PasswordReset, email, name, data).await
    }

    /// Send email verification email
    pub async fn send_email_verification(
        &self,
        email: &str,
        name: Option<&str>,
        verification_token: &str,
    ) -> Result<EmailResult, EmailError> {
        let config = self.config.read().await;
        let verify_url = format!("{}/verify-email?token={}", config.site_url, verification_token);
        drop(config);

        let mut data = HashMap::new();
        data.insert("name".to_string(), serde_json::json!(name.unwrap_or("User")));
        data.insert("verify_url".to_string(), serde_json::json!(verify_url));
        data.insert("verification_token".to_string(), serde_json::json!(verification_token));

        self.send_template(EmailTemplate::EmailVerification, email, name, data).await
    }

    /// Send welcome email
    pub async fn send_welcome(
        &self,
        email: &str,
        name: Option<&str>,
    ) -> Result<EmailResult, EmailError> {
        let config = self.config.read().await;
        let login_url = format!("{}/login", config.site_url);
        drop(config);

        let mut data = HashMap::new();
        data.insert("name".to_string(), serde_json::json!(name.unwrap_or("User")));
        data.insert("login_url".to_string(), serde_json::json!(login_url));

        self.send_template(EmailTemplate::Welcome, email, name, data).await
    }

    /// Test the email configuration by sending a test email
    pub async fn send_test(&self, to_email: &str) -> Result<EmailResult, EmailError> {
        let config = self.config.read().await;
        let site_name = config.site_name.clone();
        drop(config);

        self.send_raw(
            to_email,
            None,
            &format!("Test Email from {}", site_name),
            &format!(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; padding: 20px;">
<h1>Test Email</h1>
<p>This is a test email from <strong>{}</strong>.</p>
<p>If you received this email, your email configuration is working correctly.</p>
<p>Sent at: {}</p>
</body>
</html>"#,
                site_name,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ),
        ).await
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EmailService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            templates: self.templates.clone(),
            transport: self.transport.clone(),
        }
    }
}
