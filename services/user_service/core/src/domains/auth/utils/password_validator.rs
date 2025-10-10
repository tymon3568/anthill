use zxcvbn::{zxcvbn, Score};

/// Password strength requirements
pub struct PasswordStrength {
    /// Minimum password length
    pub min_length: usize,
    /// Minimum zxcvbn score (0-4, recommend 3+)
    pub min_score: Score,
}

impl Default for PasswordStrength {
    fn default() -> Self {
        Self {
            min_length: 8,
            min_score: Score::Three, // Strong password
        }
    }
}

/// Validation result with detailed feedback
#[derive(Debug)]
pub struct PasswordValidationResult {
    pub is_valid: bool,
    pub score: Score,
    pub feedback: Vec<String>,
    pub estimated_crack_time: String,
}

/// Validate password strength using zxcvbn
/// 
/// # Arguments
/// * `password` - The password to validate
/// * `user_inputs` - Additional context (email, name, etc.) to check for common patterns
/// 
/// # Returns
/// Validation result with score and feedback
pub fn validate_password_strength(
    password: &str,
    user_inputs: &[&str],
) -> PasswordValidationResult {
    let requirements = PasswordStrength::default();
    
    let mut feedback = Vec::new();
    
    // Check minimum length
    if password.len() < requirements.min_length {
        feedback.push(format!(
            "Password must be at least {} characters long",
            requirements.min_length
        ));
        return PasswordValidationResult {
            is_valid: false,
            score: Score::Zero,
            feedback,
            estimated_crack_time: "Instant".to_string(),
        };
    }
    
    // Run zxcvbn analysis
    let entropy = zxcvbn(password, user_inputs);
    let score = entropy.score();
    
    // Check if password meets minimum score
    let is_valid = score >= requirements.min_score;
    
    // Add suggestions from zxcvbn
    if let Some(zxcvbn_feedback) = entropy.feedback() {
        if let Some(warning) = zxcvbn_feedback.warning() {
            feedback.push(warning.to_string());
        }
        
        for suggestion in zxcvbn_feedback.suggestions() {
            feedback.push(suggestion.to_string());
        }
    }
    
    // Add score-based feedback
    match score {
        Score::Zero => feedback.push("Password is extremely weak".to_string()),
        Score::One => feedback.push("Password is very weak".to_string()),
        Score::Two => feedback.push("Password is weak".to_string()),
        Score::Three => {
            if !is_valid {
                feedback.push("Password is acceptable but could be stronger".to_string());
            }
        }
        Score::Four => {} // Strong, no additional feedback needed
        _ => {} // Future-proof for any additional scores
    }
    
    // Get crack time estimate
    let times = entropy.crack_times();
    let crack_time = format!(
        "Estimated crack time (offline slow hash): {}",
        times.offline_slow_hashing_1e4_per_second()
    );
    
    PasswordValidationResult {
        is_valid,
        score,
        feedback,
        estimated_crack_time: crack_time,
    }
}

/// Quick validation - returns error message if invalid
pub fn validate_password_quick(password: &str, user_inputs: &[&str]) -> Result<(), String> {
    let result = validate_password_strength(password, user_inputs);
    
    if !result.is_valid {
        if result.feedback.is_empty() {
            return Err("Password is too weak".to_string());
        }
        return Err(result.feedback.join(". "));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_too_short() {
        let result = validate_password_strength("abc", &[]);
        assert!(!result.is_valid);
        assert!(result.feedback.iter().any(|f| f.contains("at least")));
    }
    
    #[test]
    fn test_weak_password() {
        let result = validate_password_strength("password123", &[]);
        assert!(!result.is_valid);
        assert!(result.score < Score::Three);
    }
    
    #[test]
    fn test_password_with_user_info() {
        let result = validate_password_strength("john.doe@example.com", &["john", "doe", "john.doe@example.com"]);
        assert!(!result.is_valid);
        // Should detect that password contains user information
    }
    
    #[test]
    fn test_strong_password() {
        let result = validate_password_strength("Tr0ub4dor&3xKcd!", &[]);
        assert!(result.is_valid);
        assert!(result.score >= Score::Three);
    }
    
    #[test]
    fn test_very_strong_password() {
        let result = validate_password_strength("correcthorsebatterystaple", &[]);
        // Long random words should be strong
        assert!(result.is_valid);
    }
    
    #[test]
    fn test_quick_validation_ok() {
        let result = validate_password_quick("MyS3cure!P@ssw0rd", &[]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_quick_validation_error() {
        let result = validate_password_quick("123456", &[]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("weak") || err_msg.contains("at least"));
    }
    
    #[test]
    fn test_common_password_patterns() {
        // These should all fail
        let weak_passwords = vec![
            "password",
            "123456789",
            "qwerty",
            "abc123",
            "letmein",
        ];
        
        for pwd in weak_passwords {
            let result = validate_password_strength(pwd, &[]);
            assert!(!result.is_valid, "Password '{}' should be rejected", pwd);
        }
    }
    
    #[test]
    fn test_passphrase_style() {
        // Long passphrases should be accepted
        let result = validate_password_strength("correct horse battery staple", &[]);
        assert!(result.is_valid);
    }
}
