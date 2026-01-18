//! SMTP Email Sender implementation using lettre crate
//!
//! Provides async email sending functionality with TLS support.

use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use shared_error::AppError;
use std::sync::Arc;

/// SMTP configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

impl SmtpConfig {
    /// Check if SMTP is properly configured
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty()
    }
}

/// Email content with both HTML and plain text versions
#[derive(Debug, Clone)]
pub struct EmailContent {
    pub to: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

/// Trait for email sending abstraction
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Send an email
    async fn send(&self, content: EmailContent) -> Result<(), AppError>;

    /// Check if email sending is available
    fn is_available(&self) -> bool;
}

/// SMTP Email Sender implementation
pub struct SmtpEmailSender {
    config: SmtpConfig,
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl SmtpEmailSender {
    /// Create a new SMTP email sender
    pub fn new(config: SmtpConfig) -> Result<Self, AppError> {
        if !config.is_configured() {
            tracing::warn!("SMTP not configured - email sending will be disabled");
            return Ok(Self {
                config,
                transport: None,
            });
        }

        let transport = Self::build_transport(&config)?;

        tracing::info!(
            host = %config.host,
            port = %config.port,
            tls = %config.use_tls,
            "SMTP email sender initialized"
        );

        Ok(Self {
            config,
            transport: Some(transport),
        })
    }

    fn build_transport(
        config: &SmtpConfig,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, AppError> {
        let mut builder = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host).map_err(|e| {
                AppError::InternalError(format!("Failed to create SMTP transport: {}", e))
            })?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        };

        builder = builder.port(config.port);

        // Add credentials if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            let creds = Credentials::new(username.clone(), password.clone());
            builder = builder.credentials(creds);
        }

        Ok(builder.build())
    }

    fn build_message(&self, content: &EmailContent) -> Result<Message, AppError> {
        let from_mailbox: Mailbox =
            format!("{} <{}>", self.config.from_name, self.config.from_email)
                .parse()
                .map_err(|e| AppError::ValidationError(format!("Invalid from address: {}", e)))?;

        let to_mailbox: Mailbox = content
            .to
            .parse()
            .map_err(|e| AppError::ValidationError(format!("Invalid to address: {}", e)))?;

        let message = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(&content.subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(content.text_body.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(content.html_body.clone()),
                    ),
            )
            .map_err(|e| AppError::InternalError(format!("Failed to build email: {}", e)))?;

        Ok(message)
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send(&self, content: EmailContent) -> Result<(), AppError> {
        let transport = match &self.transport {
            Some(t) => t,
            None => {
                tracing::info!(
                    to = %content.to,
                    subject = %content.subject,
                    "[DEV] Email would be sent (SMTP not configured)"
                );
                return Ok(());
            },
        };

        let message = self.build_message(&content)?;

        match transport.send(message).await {
            Ok(response) => {
                tracing::info!(
                    to = %content.to,
                    subject = %content.subject,
                    code = ?response.code(),
                    "Email sent successfully"
                );
                Ok(())
            },
            Err(e) => {
                tracing::error!(
                    to = %content.to,
                    subject = %content.subject,
                    error = %e,
                    "Failed to send email"
                );
                Err(AppError::InternalError(format!("Failed to send email: {}", e)))
            },
        }
    }

    fn is_available(&self) -> bool {
        self.transport.is_some()
    }
}

/// Arc wrapper for thread-safe sharing
pub type SharedEmailSender = Arc<dyn EmailSender>;

/// Email templates for verification emails
pub mod templates {
    /// Generate HTML email template for email verification
    pub fn verification_email_html(verification_url: &str, expiry_hours: i64) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verify Your Email</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            text-align: center;
            padding: 20px 0;
            border-bottom: 1px solid #eee;
        }}
        .content {{
            padding: 30px 0;
        }}
        .button {{
            display: inline-block;
            background-color: #4F46E5;
            color: white !important;
            text-decoration: none;
            padding: 12px 30px;
            border-radius: 6px;
            font-weight: 600;
            margin: 20px 0;
        }}
        .button:hover {{
            background-color: #4338CA;
        }}
        .footer {{
            text-align: center;
            padding: 20px 0;
            border-top: 1px solid #eee;
            color: #666;
            font-size: 14px;
        }}
        .link {{
            word-break: break-all;
            color: #4F46E5;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Anthill</h1>
    </div>
    <div class="content">
        <h2>Verify Your Email Address</h2>
        <p>Thank you for registering! Please click the button below to verify your email address:</p>
        <p style="text-align: center;">
            <a href="{verification_url}" class="button">Verify Email</a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p class="link">{verification_url}</p>
        <p><strong>This link will expire in {expiry_hours} hours.</strong></p>
        <p>If you did not create an account, please ignore this email.</p>
    </div>
    <div class="footer">
        <p>&copy; 2026 Anthill. All rights reserved.</p>
        <p>This is an automated message, please do not reply.</p>
    </div>
</body>
</html>"#,
            verification_url = verification_url,
            expiry_hours = expiry_hours
        )
    }

    /// Generate plain text email template for email verification
    pub fn verification_email_text(verification_url: &str, expiry_hours: i64) -> String {
        format!(
            r#"Verify Your Email Address

Thank you for registering with Anthill!

Please click the link below to verify your email address:

{verification_url}

This link will expire in {expiry_hours} hours.

If you did not create an account, please ignore this email.

---
Anthill
This is an automated message, please do not reply."#,
            verification_url = verification_url,
            expiry_hours = expiry_hours
        )
    }

    /// Generate HTML email template for password reset
    pub fn password_reset_email_html(reset_url: &str, expiry_minutes: i64) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Your Password</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            text-align: center;
            padding: 20px 0;
            border-bottom: 1px solid #eee;
        }}
        .content {{
            padding: 30px 0;
        }}
        .button {{
            display: inline-block;
            background-color: #DC2626;
            color: white !important;
            text-decoration: none;
            padding: 12px 30px;
            border-radius: 6px;
            font-weight: 600;
            margin: 20px 0;
        }}
        .button:hover {{
            background-color: #B91C1C;
        }}
        .footer {{
            text-align: center;
            padding: 20px 0;
            border-top: 1px solid #eee;
            color: #666;
            font-size: 14px;
        }}
        .link {{
            word-break: break-all;
            color: #DC2626;
        }}
        .warning {{
            background-color: #FEF2F2;
            border: 1px solid #FECACA;
            border-radius: 6px;
            padding: 15px;
            margin: 20px 0;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Anthill</h1>
    </div>
    <div class="content">
        <h2>Reset Your Password</h2>
        <p>We received a request to reset your password. Click the button below to create a new password:</p>
        <p style="text-align: center;">
            <a href="{reset_url}" class="button">Reset Password</a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p class="link">{reset_url}</p>
        <p><strong>This link will expire in {expiry_minutes} minutes.</strong></p>
        <div class="warning">
            <strong>Security Notice:</strong> If you did not request a password reset, please ignore this email. Your password will remain unchanged.
        </div>
    </div>
    <div class="footer">
        <p>&copy; 2026 Anthill. All rights reserved.</p>
        <p>This is an automated message, please do not reply.</p>
    </div>
</body>
</html>"#,
            reset_url = reset_url,
            expiry_minutes = expiry_minutes
        )
    }

    /// Generate plain text email template for password reset
    pub fn password_reset_email_text(reset_url: &str, expiry_minutes: i64) -> String {
        format!(
            r#"Reset Your Password

We received a request to reset your password for your Anthill account.

Click the link below to create a new password:

{reset_url}

This link will expire in {expiry_minutes} minutes.

SECURITY NOTICE: If you did not request a password reset, please ignore this email. Your password will remain unchanged.

---
Anthill
This is an automated message, please do not reply."#,
            reset_url = reset_url,
            expiry_minutes = expiry_minutes
        )
    }
}
