//! Kanidm OAuth2/OIDC Client Library for Anthill
//!
//! This crate provides a client for authenticating users with Kanidm Identity Provider
//! using OAuth2 Authorization Code Flow with PKCE (Proof Key for Code Exchange).
//!
//! # Features
//!
//! - OAuth2 Authorization Code Flow with PKCE
//! - JWT token validation with JWKS
//! - User info retrieval
//! - Token refresh
//! - Group extraction for multi-tenancy
//!
//! # Example
//!
//! ```no_run
//! use shared_kanidm_client::{KanidmClient, KanidmConfig, KanidmOAuth2Client, PkceState};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment
//!     let config = KanidmConfig::from_env()?;
//!
//!     // Create client
//!     let client = KanidmClient::new(config)?;
//!
//!     // Generate PKCE state
//!     let pkce = PkceState::generate();
//!
//!     // Get authorization URL
//!     let auth_url = client.authorization_url(&pkce)?;
//!     println!("Redirect user to: {}", auth_url);
//!
//!     // After user authorizes, exchange code for tokens
//!     let code = "authorization_code_from_callback";
//!     let tokens = client.exchange_code(code, &pkce).await?;
//!
//!     // Validate and extract claims
//!     let claims = client.validate_token(&tokens.access_token).await?;
//!     println!("User: {:?}", claims.preferred_username);
//!     println!("Groups: {:?}", claims.groups);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod types;

// Re-exports for convenience
pub use client::{KanidmClient, KanidmOAuth2Client, PkceState};
pub use config::KanidmConfig;
pub use error::{KanidmError, Result};
pub use types::{KanidmClaims, TokenResponse, UserInfo};
