use crate::error::IdentityError;

/// Maximum nesting depth for group membership resolution.
pub const MAX_NESTING_DEPTH: usize = 3;

/// Validate a group name.
/// Format: `[a-z0-9][a-z0-9-]{1,62}[a-z0-9]`
/// - 3 to 64 characters
/// - lowercase alphanumeric and hyphens only
/// - no leading/trailing hyphens
/// - no consecutive hyphens
pub fn validate_group_name(name: &str) -> Result<(), IdentityError> {
    let len = name.len();

    if !(3..=64).contains(&len) {
        return Err(IdentityError::InvalidArgument(
            "group name must be 3-64 characters".to_string(),
        ));
    }

    let bytes = name.as_bytes();

    // No leading/trailing hyphens
    if bytes[0] == b'-' || bytes[len - 1] == b'-' {
        return Err(IdentityError::InvalidArgument(
            "group name must not start or end with a hyphen".to_string(),
        ));
    }

    let mut prev_hyphen = false;
    for &b in bytes {
        match b {
            b'a'..=b'z' | b'0'..=b'9' => {
                prev_hyphen = false;
            }
            b'-' => {
                if prev_hyphen {
                    return Err(IdentityError::InvalidArgument(
                        "group name must not contain consecutive hyphens".to_string(),
                    ));
                }
                prev_hyphen = true;
            }
            _ => {
                return Err(IdentityError::InvalidArgument(
                    "group name must contain only lowercase letters, digits, and hyphens"
                        .to_string(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_group_name("eng-team").is_ok());
        assert!(validate_group_name("frontend").is_ok());
        assert!(validate_group_name("all-staff").is_ok());
        assert!(validate_group_name("team123").is_ok());
        assert!(validate_group_name("abc").is_ok());
    }

    #[test]
    fn test_invalid_names() {
        // Too short
        assert!(validate_group_name("ab").is_err());
        // Too long
        assert!(validate_group_name(&"a".repeat(65)).is_err());
        // Leading hyphen
        assert!(validate_group_name("-team").is_err());
        // Trailing hyphen
        assert!(validate_group_name("team-").is_err());
        // Consecutive hyphens
        assert!(validate_group_name("team--name").is_err());
        // Uppercase
        assert!(validate_group_name("Team").is_err());
        // Spaces
        assert!(validate_group_name("my team").is_err());
        // Underscores
        assert!(validate_group_name("my_team").is_err());
    }
}
