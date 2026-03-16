use crate::domain::types::DomainError;

const OPEN_STATUSES: &[&str] = &["NEW", "ASSIGNED", "IN_PROGRESS", "INACTIVE"];
const CLOSED_STATUSES: &[&str] = &[
    "FIXED",
    "FIXED_VERIFIED",
    "WONT_FIX_INFEASIBLE",
    "WONT_FIX_NOT_REPRODUCIBLE",
    "WONT_FIX_OBSOLETE",
    "WONT_FIX_INTENDED_BEHAVIOR",
    "DUPLICATE",
];

pub fn is_open(status: &str) -> bool {
    OPEN_STATUSES.contains(&status)
}

pub fn is_closed(status: &str) -> bool {
    CLOSED_STATUSES.contains(&status)
}

pub fn is_valid_status(status: &str) -> bool {
    is_open(status) || is_closed(status)
}

/// Validates that a status transition from `from` to `to` is allowed.
/// Returns Ok(()) if valid, or an error describing why the transition is invalid.
pub fn validate_transition(from: &str, to: &str) -> Result<(), DomainError> {
    if from == to {
        return Ok(());
    }

    let valid = match from {
        "NEW" => matches!(
            to,
            "ASSIGNED"
                | "IN_PROGRESS"
                | "FIXED"
                | "WONT_FIX_INFEASIBLE"
                | "WONT_FIX_NOT_REPRODUCIBLE"
                | "WONT_FIX_OBSOLETE"
                | "WONT_FIX_INTENDED_BEHAVIOR"
                | "DUPLICATE"
        ),
        "ASSIGNED" => matches!(
            to,
            "NEW"
                | "IN_PROGRESS"
                | "FIXED"
                | "WONT_FIX_INFEASIBLE"
                | "WONT_FIX_NOT_REPRODUCIBLE"
                | "WONT_FIX_OBSOLETE"
                | "WONT_FIX_INTENDED_BEHAVIOR"
                | "DUPLICATE"
        ),
        "IN_PROGRESS" => matches!(
            to,
            "ASSIGNED"
                | "FIXED"
                | "WONT_FIX_INFEASIBLE"
                | "WONT_FIX_NOT_REPRODUCIBLE"
                | "WONT_FIX_OBSOLETE"
                | "WONT_FIX_INTENDED_BEHAVIOR"
                | "DUPLICATE"
        ),
        "INACTIVE" => is_open(to),
        "FIXED" => matches!(to, "FIXED_VERIFIED" | "NEW" | "ASSIGNED"),
        _ if is_closed(from) => matches!(to, "NEW" | "ASSIGNED"),
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(DomainError::FailedPrecondition(format!(
            "invalid status transition: {from} -> {to}"
        )))
    }
}

/// Computes automatic status transitions based on field changes.
/// Returns the new status if a transition should occur, or None if no auto-transition.
pub fn auto_transition(
    current_status: &str,
    assignee_changed: bool,
    new_assignee: &str,
) -> Option<&'static str> {
    match (current_status, assignee_changed) {
        ("NEW", true) if !new_assignee.is_empty() => Some("ASSIGNED"),
        ("ASSIGNED", true) if new_assignee.is_empty() => Some("NEW"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(validate_transition("NEW", "ASSIGNED").is_ok());
        assert!(validate_transition("NEW", "IN_PROGRESS").is_ok());
        assert!(validate_transition("NEW", "FIXED").is_ok());
        assert!(validate_transition("ASSIGNED", "IN_PROGRESS").is_ok());
        assert!(validate_transition("ASSIGNED", "NEW").is_ok());
        assert!(validate_transition("IN_PROGRESS", "FIXED").is_ok());
        assert!(validate_transition("FIXED", "FIXED_VERIFIED").is_ok());
        assert!(validate_transition("FIXED", "NEW").is_ok());
        assert!(validate_transition("FIXED", "ASSIGNED").is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(validate_transition("NEW", "FIXED_VERIFIED").is_err());
        assert!(validate_transition("IN_PROGRESS", "NEW").is_err());
        assert!(validate_transition("FIXED_VERIFIED", "IN_PROGRESS").is_err());
    }

    #[test]
    fn test_same_status_is_valid() {
        assert!(validate_transition("NEW", "NEW").is_ok());
        assert!(validate_transition("FIXED", "FIXED").is_ok());
    }

    #[test]
    fn test_auto_transition_assign() {
        assert_eq!(
            auto_transition("NEW", true, "user@example.com"),
            Some("ASSIGNED")
        );
        assert_eq!(auto_transition("NEW", true, ""), None);
        assert_eq!(auto_transition("NEW", false, "user@example.com"), None);
    }

    #[test]
    fn test_auto_transition_unassign() {
        assert_eq!(auto_transition("ASSIGNED", true, ""), Some("NEW"));
        assert_eq!(auto_transition("ASSIGNED", true, "other@example.com"), None);
        assert_eq!(auto_transition("ASSIGNED", false, ""), None);
    }

    #[test]
    fn test_auto_transition_no_change() {
        assert_eq!(
            auto_transition("IN_PROGRESS", true, "user@example.com"),
            None
        );
        assert_eq!(auto_transition("FIXED", true, ""), None);
    }

    #[test]
    fn test_is_open_closed() {
        assert!(is_open("NEW"));
        assert!(is_open("ASSIGNED"));
        assert!(is_open("IN_PROGRESS"));
        assert!(is_open("INACTIVE"));
        assert!(!is_open("FIXED"));

        assert!(is_closed("FIXED"));
        assert!(is_closed("DUPLICATE"));
        assert!(!is_closed("NEW"));
    }
}
