use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a cryptographically secure invite token
/// Returns: (plaintext_token, token_hash)
///
/// Security: Uses 128-bit entropy (16 bytes) encoded as URL-safe base64.
/// Hash is SHA-256 for storage (never store plaintext).
pub fn generate_invite_token() -> (String, String) {
    // Generate 128-bit (16 bytes) of entropy
    let mut token_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut token_bytes);

    // Encode as URL-safe base64 for transmission
    let plaintext_token = URL_SAFE_NO_PAD.encode(token_bytes);

    // Hash for storage (never store plaintext)
    let mut hasher = Sha256::new();
    hasher.update(plaintext_token.as_bytes());
    let hash_bytes = hasher.finalize();
    let token_hash = hex::encode(hash_bytes);

    (plaintext_token, token_hash)
}

/// Hash an incoming token for lookup
///
/// Security: Uses SHA-256 to match stored hashes.
/// Timing-safe comparison should be used when comparing hashes.
pub fn hash_token(plaintext_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext_token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Validate invitation token format
///
/// Basic validation - tokens should be URL-safe base64.
pub fn validate_token_format(token: &str) -> bool {
    // Check length (128-bit = 16 bytes = ~22 chars in base64url)
    if token.len() < 20 || token.len() > 30 {
        return false;
    }

    // Check characters (URL-safe base64)
    token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite_token() {
        let (token, hash) = generate_invite_token();

        // Token should be URL-safe base64
        assert!(validate_token_format(&token));

        // Hash should be 64 characters (SHA-256 hex)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Hash of token should match
        assert_eq!(hash, hash_token(&token));
    }

    #[test]
    fn test_hash_token_consistency() {
        let token = "test_token_123";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_validate_token_format() {
        // Valid tokens (around 22 chars for 128-bit entropy)
        assert!(validate_token_format("ABCDEFGH123456789012"));
        assert!(validate_token_format("ABCDEF123456-789_ABC"));
        assert!(validate_token_format("ABCDEF123456_789-ABC"));

        // Invalid tokens
        assert!(!validate_token_format("")); // Too short
        assert!(!validate_token_format("ABC")); // Too short
        assert!(!validate_token_format("ABCDEF123456789012345678901234567890")); // Too long
        assert!(!validate_token_format("ABCDEF123456 789 ABC")); // Space
        assert!(!validate_token_format("ABCDEF123456+789_ABC")); // Invalid char
        assert!(!validate_token_format("ABCDEF123456/789_ABC")); // Invalid char
    }

    #[test]
    fn test_token_entropy() {
        // Generate multiple tokens - should be unique
        let mut tokens = std::collections::HashSet::new();
        for _ in 0..100 {
            let (token, _) = generate_invite_token();
            assert!(tokens.insert(token), "Generated duplicate token");
        }
    }
}
