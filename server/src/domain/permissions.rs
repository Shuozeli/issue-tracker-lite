use quiver_driver_core::{Connection, Value};
use quiver_query::{Filter, Query};
use tonic::Request;

use crate::db::row_mapping::{Component, ComponentAcl, Issue};
use crate::domain::types::DomainError;

/// Component-level permissions.
/// Each permission implies the permissions below it in the hierarchy
/// (except CREATE_ISSUES which is isolated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentPermission {
    ViewIssues,
    CommentOnIssues,
    EditIssues,
    AdminIssues,
    CreateIssues,
    ViewComponents,
    AdminComponents,
    ViewRestricted,
    ViewRestrictedPlus,
}

impl ComponentPermission {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        match s {
            "VIEW_ISSUES" => Ok(Self::ViewIssues),
            "COMMENT_ON_ISSUES" => Ok(Self::CommentOnIssues),
            "EDIT_ISSUES" => Ok(Self::EditIssues),
            "ADMIN_ISSUES" => Ok(Self::AdminIssues),
            "CREATE_ISSUES" => Ok(Self::CreateIssues),
            "VIEW_COMPONENTS" => Ok(Self::ViewComponents),
            "ADMIN_COMPONENTS" => Ok(Self::AdminComponents),
            "VIEW_RESTRICTED" => Ok(Self::ViewRestricted),
            "VIEW_RESTRICTED_PLUS" => Ok(Self::ViewRestrictedPlus),
            _ => Err(DomainError::InvalidArgument(format!(
                "unknown permission: {s}"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ViewIssues => "VIEW_ISSUES",
            Self::CommentOnIssues => "COMMENT_ON_ISSUES",
            Self::EditIssues => "EDIT_ISSUES",
            Self::AdminIssues => "ADMIN_ISSUES",
            Self::CreateIssues => "CREATE_ISSUES",
            Self::ViewComponents => "VIEW_COMPONENTS",
            Self::AdminComponents => "ADMIN_COMPONENTS",
            Self::ViewRestricted => "VIEW_RESTRICTED",
            Self::ViewRestrictedPlus => "VIEW_RESTRICTED_PLUS",
        }
    }

    pub fn from_proto(val: i32) -> Result<Self, DomainError> {
        match val {
            1 => Ok(Self::ViewIssues),
            2 => Ok(Self::CommentOnIssues),
            3 => Ok(Self::EditIssues),
            4 => Ok(Self::AdminIssues),
            5 => Ok(Self::CreateIssues),
            6 => Ok(Self::ViewComponents),
            7 => Ok(Self::AdminComponents),
            8 => Ok(Self::ViewRestricted),
            9 => Ok(Self::ViewRestrictedPlus),
            _ => Err(DomainError::InvalidArgument(format!(
                "unknown permission value: {val}"
            ))),
        }
    }

    pub fn to_proto(&self) -> i32 {
        match self {
            Self::ViewIssues => 1,
            Self::CommentOnIssues => 2,
            Self::EditIssues => 3,
            Self::AdminIssues => 4,
            Self::CreateIssues => 5,
            Self::ViewComponents => 6,
            Self::AdminComponents => 7,
            Self::ViewRestricted => 8,
            Self::ViewRestrictedPlus => 9,
        }
    }
}

/// Expand a set of permissions to include all implied permissions.
///
/// Implication graph:
///   ADMIN_COMPONENTS -> ADMIN_ISSUES -> EDIT_ISSUES -> COMMENT_ON_ISSUES -> VIEW_ISSUES
///   ADMIN_COMPONENTS -> VIEW_COMPONENTS
///   VIEW_RESTRICTED_PLUS -> VIEW_RESTRICTED
///   CREATE_ISSUES is isolated (implies nothing, not implied by anything except ADMIN)
pub fn expand_permissions(
    perms: &[ComponentPermission],
) -> std::collections::HashSet<ComponentPermission> {
    use ComponentPermission::*;
    let mut expanded = std::collections::HashSet::new();

    for &perm in perms {
        expanded.insert(perm);
        match perm {
            AdminComponents => {
                expanded.insert(AdminIssues);
                expanded.insert(EditIssues);
                expanded.insert(CommentOnIssues);
                expanded.insert(ViewIssues);
                expanded.insert(ViewComponents);
                expanded.insert(CreateIssues);
            }
            AdminIssues => {
                expanded.insert(EditIssues);
                expanded.insert(CommentOnIssues);
                expanded.insert(ViewIssues);
            }
            EditIssues => {
                expanded.insert(CommentOnIssues);
                expanded.insert(ViewIssues);
            }
            CommentOnIssues => {
                expanded.insert(ViewIssues);
            }
            ViewRestrictedPlus => {
                expanded.insert(ViewRestricted);
            }
            _ => {}
        }
    }

    expanded
}

/// Determines the permissions granted via expanded access based on the user's role on an issue.
/// Returns the single permission level granted, which will be expanded by the caller.
pub fn expanded_access_permission(
    user_id: &str,
    assignee: &str,
    verifier: &str,
    reporter: &str,
) -> Option<ComponentPermission> {
    // Assignee or Verifier -> EDIT_ISSUES
    if !assignee.is_empty() && assignee == user_id {
        return Some(ComponentPermission::EditIssues);
    }
    if !verifier.is_empty() && verifier == user_id {
        return Some(ComponentPermission::EditIssues);
    }
    // Reporter -> COMMENT_ON_ISSUES
    if !reporter.is_empty() && reporter == user_id {
        return Some(ComponentPermission::CommentOnIssues);
    }
    None
}

/// Validate identity type string.
pub fn validate_identity_type(s: &str) -> Result<(), DomainError> {
    match s {
        "USER" | "GROUP" | "PUBLIC" => Ok(()),
        _ => Err(DomainError::InvalidArgument(format!(
            "invalid identity type: {s}, expected USER, GROUP, or PUBLIC"
        ))),
    }
}

pub fn identity_type_from_proto(val: i32) -> Result<String, DomainError> {
    match val {
        1 => Ok("USER".to_string()),
        2 => Ok("GROUP".to_string()),
        3 => Ok("PUBLIC".to_string()),
        _ => Err(DomainError::InvalidArgument(format!(
            "unknown identity type value: {val}"
        ))),
    }
}

pub fn identity_type_to_proto(s: &str) -> i32 {
    match s {
        "USER" => 1,
        "GROUP" => 2,
        "PUBLIC" => 3,
        _ => 0,
    }
}

/// Hotlist permission helpers.
pub fn hotlist_permission_from_proto(val: i32) -> Result<String, DomainError> {
    match val {
        1 => Ok("HOTLIST_VIEW".to_string()),
        2 => Ok("HOTLIST_VIEW_APPEND".to_string()),
        3 => Ok("HOTLIST_ADMIN".to_string()),
        _ => Err(DomainError::InvalidArgument(format!(
            "unknown hotlist permission value: {val}"
        ))),
    }
}

pub fn hotlist_permission_to_proto(s: &str) -> i32 {
    match s {
        "HOTLIST_VIEW" => 1,
        "HOTLIST_VIEW_APPEND" => 2,
        "HOTLIST_ADMIN" => 3,
        _ => 0,
    }
}

/// Check if a hotlist permission implies another.
pub fn hotlist_permission_implies(held: &str, required: &str) -> bool {
    match (held, required) {
        ("HOTLIST_ADMIN", _) => true,
        ("HOTLIST_VIEW_APPEND", "HOTLIST_VIEW") => true,
        ("HOTLIST_VIEW_APPEND", "HOTLIST_VIEW_APPEND") => true,
        (a, b) => a == b,
    }
}

/// Maximum allowed length for x-user-id header values.
const MAX_USER_ID_LEN: usize = 256;

/// Extract user_id from gRPC request metadata.
/// Returns None if the `x-user-id` header is not present (unauthenticated / anonymous).
/// Validates format: max 256 chars, alphanumeric plus @, -, _, . only.
pub fn extract_user_id<T>(request: &Request<T>) -> Option<String> {
    request
        .metadata()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .filter(|s| s.len() <= MAX_USER_ID_LEN)
        .filter(|s| {
            s.chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '@' | '-' | '_' | '.' | '+'))
        })
        .map(|s| s.to_string())
}

/// Check that the caller has the required component permission.
/// Same logic as check_component_permission but uses quiver Connection trait.
/// `user_groups` contains pre-resolved group names the user belongs to.
pub async fn check_component_permission_quiver<C: Connection>(
    conn: &C,
    component_id: i64,
    user_id: Option<&str>,
    required: ComponentPermission,
    issue_id: Option<i64>,
    user_groups: &[String],
) -> Result<(), DomainError> {
    let user_id = match user_id {
        Some(uid) if !uid.is_empty() => uid,
        _ => {
            return Err(DomainError::PermissionDenied(
                "authentication required".to_string(),
            ))
        }
    };

    // Step 1: Check component ACL for direct match
    let acl_q = Query::table("ComponentAcl")
        .find_many()
        .filter(Filter::eq("componentId", Value::Int(component_id)))
        .build();
    let acl_rows = conn
        .query(&acl_q)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    for row in &acl_rows {
        let acl = ComponentAcl::try_from(row)?;
        let matches = match acl.identity_type.as_str() {
            "USER" => acl.identity_value == user_id,
            "GROUP" => user_groups.contains(&acl.identity_value),
            "PUBLIC" => true,
            _ => false,
        };
        if matches {
            let perm_strings: Vec<String> =
                serde_json::from_str(&acl.permissions).unwrap_or_default();
            let perms: Vec<ComponentPermission> = perm_strings
                .iter()
                .filter_map(|s| ComponentPermission::parse(s).ok())
                .collect();
            let expanded = expand_permissions(&perms);
            if expanded.contains(&required) {
                return Ok(());
            }
        }
    }

    // Step 2: Check expanded access if an issue_id is provided
    if let Some(iid) = issue_id {
        let comp_q = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(component_id)))
            .build();
        let comp_row = conn
            .query_optional(&comp_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Some(comp_row) = comp_row {
            let comp = Component::try_from(&comp_row)?;
            if comp.expanded_access_enabled {
                let issue_q = Query::table("Issue")
                    .find_first()
                    .filter(Filter::eq("id", Value::Int(iid)))
                    .build();
                let issue_row = conn
                    .query_optional(&issue_q)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;

                if let Some(issue_row) = issue_row {
                    let issue = Issue::try_from(&issue_row)?;
                    if let Some(base_perm) = expanded_access_permission(
                        user_id,
                        &issue.assignee,
                        &issue.verifier,
                        &issue.reporter,
                    ) {
                        let expanded = expand_permissions(&[base_perm]);
                        if expanded.contains(&required) {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    Err(DomainError::PermissionDenied("access denied".to_string()))
}

/// Quiver-based hotlist permission check.
/// `user_groups` contains pre-resolved group names the user belongs to.
pub async fn check_hotlist_permission_quiver<C: Connection>(
    conn: &C,
    hotlist_id: i64,
    user_id: Option<&str>,
    required: &str,
    user_groups: &[String],
) -> Result<(), DomainError> {
    let user_id = match user_id {
        Some(uid) if !uid.is_empty() => uid,
        _ => {
            return Err(DomainError::PermissionDenied(
                "authentication required".to_string(),
            ))
        }
    };

    let acl_q = Query::table("HotlistAcl")
        .find_many()
        .filter(Filter::eq("hotlistId", Value::Int(hotlist_id)))
        .build();
    let acl_rows = conn
        .query(&acl_q)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    for row in &acl_rows {
        let acl = crate::db::row_mapping::HotlistAcl::try_from(row)?;
        let matches = match acl.identity_type.as_str() {
            "USER" => acl.identity_value == user_id,
            "GROUP" => user_groups.contains(&acl.identity_value),
            "PUBLIC" => true,
            _ => false,
        };
        if matches && hotlist_permission_implies(&acl.permission, required) {
            return Ok(());
        }
    }

    Err(DomainError::PermissionDenied("access denied".to_string()))
}

/// Return the set of component IDs the user has at least `required` permission on.
/// Scans all ComponentAcl entries and returns matching component IDs.
/// If user_id is None, returns an empty set (unauthenticated).
pub async fn get_accessible_component_ids<C: Connection>(
    conn: &C,
    user_id: Option<&str>,
    required: ComponentPermission,
    user_groups: &[String],
) -> Result<Vec<i64>, DomainError> {
    let user_id = match user_id {
        Some(uid) if !uid.is_empty() => uid,
        _ => return Ok(vec![]),
    };

    // Fetch all ACL entries
    let acl_q = Query::table("ComponentAcl").find_many().build();
    let acl_rows = conn
        .query(&acl_q)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    let mut accessible = std::collections::HashSet::new();

    for row in &acl_rows {
        let acl = ComponentAcl::try_from(row)?;
        let matches = match acl.identity_type.as_str() {
            "USER" => acl.identity_value == user_id,
            "GROUP" => user_groups.contains(&acl.identity_value),
            "PUBLIC" => true,
            _ => false,
        };
        if matches {
            let perm_strings: Vec<String> =
                serde_json::from_str(&acl.permissions).unwrap_or_default();
            let perms: Vec<ComponentPermission> = perm_strings
                .iter()
                .filter_map(|s| ComponentPermission::parse(s).ok())
                .collect();
            let expanded = expand_permissions(&perms);
            if expanded.contains(&required) {
                accessible.insert(acl.component_id as i64);
            }
        }
    }

    Ok(accessible.into_iter().collect())
}

/// Return the set of hotlist IDs the user has at least `required` permission on.
pub async fn get_accessible_hotlist_ids<C: Connection>(
    conn: &C,
    user_id: Option<&str>,
    required: &str,
    user_groups: &[String],
) -> Result<Vec<i64>, DomainError> {
    let user_id = match user_id {
        Some(uid) if !uid.is_empty() => uid,
        _ => return Ok(vec![]),
    };

    let acl_q = Query::table("HotlistAcl").find_many().build();
    let acl_rows = conn
        .query(&acl_q)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    let mut accessible = std::collections::HashSet::new();

    for row in &acl_rows {
        let acl = crate::db::row_mapping::HotlistAcl::try_from(row)?;
        let matches = match acl.identity_type.as_str() {
            "USER" => acl.identity_value == user_id,
            "GROUP" => user_groups.contains(&acl.identity_value),
            "PUBLIC" => true,
            _ => false,
        };
        if matches && hotlist_permission_implies(&acl.permission, required) {
            accessible.insert(acl.hotlist_id as i64);
        }
    }

    Ok(accessible.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_admin_components() {
        let expanded = expand_permissions(&[ComponentPermission::AdminComponents]);
        assert!(expanded.contains(&ComponentPermission::AdminIssues));
        assert!(expanded.contains(&ComponentPermission::EditIssues));
        assert!(expanded.contains(&ComponentPermission::CommentOnIssues));
        assert!(expanded.contains(&ComponentPermission::ViewIssues));
        assert!(expanded.contains(&ComponentPermission::ViewComponents));
        assert!(expanded.contains(&ComponentPermission::CreateIssues));
    }

    #[test]
    fn test_expand_edit_issues() {
        let expanded = expand_permissions(&[ComponentPermission::EditIssues]);
        assert!(expanded.contains(&ComponentPermission::CommentOnIssues));
        assert!(expanded.contains(&ComponentPermission::ViewIssues));
        assert!(!expanded.contains(&ComponentPermission::AdminIssues));
        assert!(!expanded.contains(&ComponentPermission::CreateIssues));
    }

    #[test]
    fn test_create_issues_isolated() {
        let expanded = expand_permissions(&[ComponentPermission::CreateIssues]);
        assert!(expanded.contains(&ComponentPermission::CreateIssues));
        assert!(!expanded.contains(&ComponentPermission::ViewIssues));
        assert!(!expanded.contains(&ComponentPermission::CommentOnIssues));
    }

    #[test]
    fn test_expanded_access_assignee() {
        let perm = expanded_access_permission("user@test.com", "user@test.com", "", "");
        assert_eq!(perm, Some(ComponentPermission::EditIssues));
    }

    #[test]
    fn test_expanded_access_reporter() {
        let perm = expanded_access_permission("user@test.com", "", "", "user@test.com");
        assert_eq!(perm, Some(ComponentPermission::CommentOnIssues));
    }

    #[test]
    fn test_expanded_access_none() {
        let perm = expanded_access_permission("user@test.com", "other@test.com", "", "");
        assert_eq!(perm, None);
    }

    #[test]
    fn test_hotlist_permission_implies() {
        assert!(hotlist_permission_implies("HOTLIST_ADMIN", "HOTLIST_VIEW"));
        assert!(hotlist_permission_implies(
            "HOTLIST_ADMIN",
            "HOTLIST_VIEW_APPEND"
        ));
        assert!(hotlist_permission_implies(
            "HOTLIST_VIEW_APPEND",
            "HOTLIST_VIEW"
        ));
        assert!(!hotlist_permission_implies(
            "HOTLIST_VIEW",
            "HOTLIST_VIEW_APPEND"
        ));
        assert!(!hotlist_permission_implies("HOTLIST_VIEW", "HOTLIST_ADMIN"));
    }

    #[test]
    fn test_permission_roundtrip() {
        for val in 1..=9 {
            let perm = ComponentPermission::from_proto(val).unwrap();
            assert_eq!(perm.to_proto(), val);
            let s = perm.as_str();
            let perm2 = ComponentPermission::parse(s).unwrap();
            assert_eq!(perm, perm2);
        }
    }
}
