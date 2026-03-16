use test_utils::*;

use issuetracker_server::proto::acl_service_client::AclServiceClient;
use issuetracker_server::proto::component_service_client::ComponentServiceClient;
use issuetracker_server::proto::hotlist_service_client::HotlistServiceClient;
use issuetracker_server::proto::search_service_client::SearchServiceClient;

// ── ACL Service Tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_set_and_get_component_acl() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "ACL Test Component", None).await;

    // Set ACL for a user
    let entry = acl
        .set_component_acl(SetComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "alice@test.com".to_string(),
            permissions: vec![
                ComponentPermission::ViewIssues as i32,
                ComponentPermission::CommentOnIssues as i32,
            ],
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(entry.component_id, comp_id);
    assert_eq!(entry.identity_type, IdentityType::User as i32);
    assert_eq!(entry.identity_value, "alice@test.com");
    assert_eq!(entry.permissions.len(), 2);

    // Get ACL entries
    let resp = acl
        .get_component_acl(GetComponentAclRequest {
            component_id: comp_id,
        })
        .await
        .unwrap()
        .into_inner();

    // 2 entries: admin@test.com (from create_component helper) + alice@test.com
    assert_eq!(resp.entries.len(), 2);
    assert!(resp
        .entries
        .iter()
        .any(|e| e.identity_value == "alice@test.com"));
}

#[tokio::test]
async fn test_set_component_acl_upsert() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Upsert ACL Component", None).await;

    // Set initial ACL
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "bob@test.com".to_string(),
        permissions: vec![ComponentPermission::ViewIssues as i32],
    })
    .await
    .unwrap();

    // Upsert with different permissions
    let updated = acl
        .set_component_acl(SetComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "bob@test.com".to_string(),
            permissions: vec![
                ComponentPermission::ViewIssues as i32,
                ComponentPermission::EditIssues as i32,
            ],
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(updated.permissions.len(), 2);

    // Two entries: admin@test.com (from create_component helper) + bob@test.com (upserted)
    let resp = acl
        .get_component_acl(GetComponentAclRequest {
            component_id: comp_id,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.entries.len(), 2);
}

#[tokio::test]
async fn test_remove_component_acl() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Remove ACL Component", None).await;

    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "charlie@test.com".to_string(),
        permissions: vec![ComponentPermission::ViewIssues as i32],
    })
    .await
    .unwrap();

    // Remove it
    acl.remove_component_acl(RemoveComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "charlie@test.com".to_string(),
    })
    .await
    .unwrap();

    // Verify charlie removed; admin@test.com (from create_component helper) remains
    let resp = acl
        .get_component_acl(GetComponentAclRequest {
            component_id: comp_id,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.entries.len(), 1);
    assert_eq!(resp.entries[0].identity_value, TEST_ADMIN_USER);
}

#[tokio::test]
async fn test_remove_component_acl_not_found() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Remove NF Component", None).await;

    let err = acl
        .remove_component_acl(RemoveComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "nobody@test.com".to_string(),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_set_component_acl_nonexistent_component() {
    let fixture = TestFixture::new().await;
    let mut acl = fixture.acl_client();

    let err = acl
        .set_component_acl(SetComponentAclRequest {
            component_id: 99999,
            identity_type: IdentityType::User as i32,
            identity_value: "alice@test.com".to_string(),
            permissions: vec![ComponentPermission::ViewIssues as i32],
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_set_component_acl_empty_identity_value() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Empty Identity Component", None).await;

    let err = acl
        .set_component_acl(SetComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "".to_string(),
            permissions: vec![ComponentPermission::ViewIssues as i32],
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_set_component_acl_empty_permissions() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Empty Perm Component", None).await;

    let err = acl
        .set_component_acl(SetComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "alice@test.com".to_string(),
            permissions: vec![],
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_check_component_permission_acl_match() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Check Perm Component", None).await;

    // Grant ADMIN_ISSUES + ADMIN_COMPONENTS to user (ADMIN_COMPONENTS required to call check)
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "admin@test.com".to_string(),
        permissions: vec![
            ComponentPermission::AdminIssues as i32,
            ComponentPermission::AdminComponents as i32,
        ],
    })
    .await
    .unwrap();

    // Check permissions
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "admin@test.com".to_string(),
            issue_id: None,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "ACL");
    // ADMIN_ISSUES implies EDIT_ISSUES, COMMENT_ON_ISSUES, VIEW_ISSUES
    assert!(resp.permissions.len() >= 4);
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::AdminIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::EditIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::CommentOnIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::ViewIssues as i32)));
}

#[tokio::test]
async fn test_check_component_permission_public_acl() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Public ACL Component", None).await;

    // Grant PUBLIC VIEW_ISSUES
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::Public as i32,
        identity_value: "*".to_string(),
        permissions: vec![ComponentPermission::ViewIssues as i32],
    })
    .await
    .unwrap();

    // Any user should get VIEW_ISSUES
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "random@test.com".to_string(),
            issue_id: None,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "ACL");
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::ViewIssues as i32)));
}

#[tokio::test]
async fn test_check_component_permission_denied() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Denied Component", None).await;

    // No ACL set, check should return DENIED
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "nobody@test.com".to_string(),
            issue_id: None,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "DENIED");
    assert!(resp.permissions.is_empty());
}

#[tokio::test]
async fn test_check_component_permission_expanded_access() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    // Create component with expanded access enabled
    let comp_id = create_component(&mut comp, &mut acl, "Expanded Access Component", None).await;

    // Enable expanded access on the component
    comp.update_component(UpdateComponentRequest {
        component_id: comp_id,
        expanded_access_enabled: Some(true),
        ..Default::default()
    })
    .await
    .unwrap();

    // Create an issue with assignee
    let issue_resp = issue
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Expanded Access Issue".to_string(),
            assignee: Some("assignee@test.com".to_string()),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Check permissions for assignee with expanded access
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "assignee@test.com".to_string(),
            issue_id: Some(issue_resp.issue_id),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "EXPANDED_ACCESS");
    // Assignee gets EDIT_ISSUES (which implies COMMENT_ON_ISSUES and VIEW_ISSUES)
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::EditIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::CommentOnIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::ViewIssues as i32)));
}

#[tokio::test]
async fn test_check_component_permission_expanded_access_reporter() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Reporter Access Component", None).await;

    comp.update_component(UpdateComponentRequest {
        component_id: comp_id,
        expanded_access_enabled: Some(true),
        ..Default::default()
    })
    .await
    .unwrap();

    let issue_resp = issue
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Reporter Access Issue".to_string(),
            reporter: Some("reporter@test.com".to_string()),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Reporter gets COMMENT_ON_ISSUES
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "reporter@test.com".to_string(),
            issue_id: Some(issue_resp.issue_id),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "EXPANDED_ACCESS");
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::CommentOnIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::ViewIssues as i32)));
}

#[tokio::test]
async fn test_check_component_permission_expanded_access_disabled() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    // Component with expanded access explicitly disabled
    let comp_id = create_component(&mut comp, &mut acl, "No Expanded Component", None).await;

    comp.update_component(UpdateComponentRequest {
        component_id: comp_id,
        expanded_access_enabled: Some(false),
        ..Default::default()
    })
    .await
    .unwrap();

    let issue_resp = issue
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "No Expanded Issue".to_string(),
            assignee: Some("assignee@test.com".to_string()),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Even though user is assignee, no expanded access since it's disabled
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "assignee@test.com".to_string(),
            issue_id: Some(issue_resp.issue_id),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "DENIED");
    assert!(resp.permissions.is_empty());
}

// ── Hotlist ACL Tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_set_and_get_hotlist_acl() {
    let fixture = TestFixture::new().await;
    let mut hotlist = fixture.hotlist_client();
    let mut acl = fixture.acl_client();

    // Create a hotlist
    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "ACL Test Hotlist".to_string(),
            description: String::new(),
            owner: "owner@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    let entry = acl
        .set_hotlist_acl(SetHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
            identity_type: IdentityType::User as i32,
            identity_value: "alice@test.com".to_string(),
            permission: HotlistPermission::HotlistViewAppend as i32,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(entry.hotlist_id, hl.hotlist_id);
    assert_eq!(
        entry.permission,
        HotlistPermission::HotlistViewAppend as i32
    );

    let resp = acl
        .get_hotlist_acl(GetHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();

    // 2 entries: auto-granted admin@test.com + alice@test.com
    assert_eq!(resp.entries.len(), 2);
}

#[tokio::test]
async fn test_remove_hotlist_acl() {
    let fixture = TestFixture::new().await;
    let mut hotlist = fixture.hotlist_client();
    let mut acl = fixture.acl_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Remove HL ACL".to_string(),
            description: String::new(),
            owner: "owner@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: IdentityType::User as i32,
        identity_value: "bob@test.com".to_string(),
        permission: HotlistPermission::HotlistAdmin as i32,
    })
    .await
    .unwrap();

    acl.remove_hotlist_acl(RemoveHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: IdentityType::User as i32,
        identity_value: "bob@test.com".to_string(),
    })
    .await
    .unwrap();

    let resp = acl
        .get_hotlist_acl(GetHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();

    // 1 entry remaining: auto-granted admin@test.com (bob was removed)
    assert_eq!(resp.entries.len(), 1);
}

#[tokio::test]
async fn test_set_hotlist_acl_nonexistent_hotlist() {
    let fixture = TestFixture::new().await;
    let mut acl = fixture.acl_client();

    let err = acl
        .set_hotlist_acl(SetHotlistAclRequest {
            hotlist_id: 99999,
            identity_type: IdentityType::User as i32,
            identity_value: "alice@test.com".to_string(),
            permission: HotlistPermission::HotlistView as i32,
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_multiple_component_acl_entries() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Multi ACL Component", None).await;

    // Set ACL for two different users
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "user1@test.com".to_string(),
        permissions: vec![ComponentPermission::ViewIssues as i32],
    })
    .await
    .unwrap();

    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "user2@test.com".to_string(),
        permissions: vec![ComponentPermission::AdminIssues as i32],
    })
    .await
    .unwrap();

    let resp = acl
        .get_component_acl(GetComponentAclRequest {
            component_id: comp_id,
        })
        .await
        .unwrap()
        .into_inner();

    // 3 entries: admin@test.com (from create_component helper) + user1@test.com + user2@test.com
    assert_eq!(resp.entries.len(), 3);
}

#[tokio::test]
async fn test_check_component_permission_acl_takes_priority_over_expanded() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Priority Check Component", None).await;

    // Enable expanded access
    comp.update_component(UpdateComponentRequest {
        component_id: comp_id,
        expanded_access_enabled: Some(true),
        ..Default::default()
    })
    .await
    .unwrap();

    // Create issue with assignee
    let issue_resp = issue
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Priority Issue".to_string(),
            assignee: Some("admin@test.com".to_string()),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Also grant ADMIN_COMPONENTS via ACL
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "admin@test.com".to_string(),
        permissions: vec![ComponentPermission::AdminComponents as i32],
    })
    .await
    .unwrap();

    // ACL should take priority over expanded access
    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "admin@test.com".to_string(),
            issue_id: Some(issue_resp.issue_id),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.grant_source, "ACL");
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::AdminComponents as i32)));
}

// ── Permission Enforcement Tests ───────────────────────────────────────────

#[tokio::test]
async fn test_permission_denied_get_component() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    // Create component (admin interceptor provides auth)
    let comp_id = create_component(&mut comp, &mut acl, "AuthZ Component", None).await;

    // Get with authenticated user who has no ACL -> PERMISSION_DENIED
    let err = comp
        .get_component(with_user(
            "nobody@test.com",
            GetComponentRequest {
                component_id: comp_id,
            },
        ))
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_permission_allowed_get_component_with_acl() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "AuthZ Component 2", None).await;

    // Grant VIEW_COMPONENTS to alice
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "alice@test.com".to_string(),
        permissions: vec![ComponentPermission::ViewComponents as i32],
    })
    .await
    .unwrap();

    // alice can get the component
    let resp = comp
        .get_component(with_user(
            "alice@test.com",
            GetComponentRequest {
                component_id: comp_id,
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.component_id, comp_id);
}

#[tokio::test]
async fn test_permission_denied_create_issue() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Issue AuthZ Component", None).await;

    // Try to create issue with auth header but no CREATE_ISSUES permission
    let err = issue
        .create_issue(with_user(
            "nobody@test.com",
            CreateIssueRequest {
                component_id: comp_id,
                title: "Unauthorized Issue".to_string(),
                priority: Priority::P2 as i32,
                r#type: IssueType::Bug as i32,
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_permission_allowed_create_issue_with_acl() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Issue AuthZ Component 2", None).await;

    // Grant CREATE_ISSUES to bob
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "bob@test.com".to_string(),
        permissions: vec![ComponentPermission::CreateIssues as i32],
    })
    .await
    .unwrap();

    // bob can create an issue
    let resp = issue
        .create_issue(with_user(
            "bob@test.com",
            CreateIssueRequest {
                component_id: comp_id,
                title: "Authorized Issue".to_string(),
                priority: Priority::P2 as i32,
                r#type: IssueType::Bug as i32,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert!(resp.issue_id > 0);
}

#[tokio::test]
async fn test_permission_denied_update_issue() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Update AuthZ Component", None).await;

    // Create issue without auth header (allowed)
    let created = issue
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "To Update".to_string(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Try to update with auth but no EDIT_ISSUES permission
    let err = issue
        .update_issue(with_user(
            "nobody@test.com",
            UpdateIssueRequest {
                issue_id: created.issue_id,
                title: Some("Hacked".to_string()),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_permission_expanded_access_allows_edit() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut issue = fixture.issue_client();
    let mut acl = fixture.acl_client();

    let comp_id = create_component(&mut comp, &mut acl, "Expanded Edit Component", None).await;

    // Component has expanded access enabled by default

    // Create issue with assignee (no auth header)
    let created = issue
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Assignee Edit Test".to_string(),
            assignee: Some("dev@test.com".to_string()),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Assignee can update via expanded access
    let resp = issue
        .update_issue(with_user(
            "dev@test.com",
            UpdateIssueRequest {
                issue_id: created.issue_id,
                title: Some("Updated by assignee".to_string()),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.title, "Updated by assignee");
}

#[tokio::test]
async fn test_permission_no_header_denies_access() {
    let fixture = TestFixture::new().await;
    let mut comp = fixture.component_client();
    let mut unauthed_comp = fixture.unauthenticated_component_client();
    let mut acl = fixture.acl_client();

    // Create component with authenticated admin client
    let comp_id = create_component(&mut comp, &mut acl, "No Auth Component", None).await;

    // Without x-user-id header, permission-checked operations are denied
    let err = unauthed_comp
        .get_component(GetComponentRequest {
            component_id: comp_id,
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_permission_denied_hotlist_get() {
    let fixture = TestFixture::new().await;
    let mut hotlist = fixture.hotlist_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Private Hotlist".to_string(),
            description: String::new(),
            owner: "owner@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    // Authenticated user with no ACL -> denied
    let err = hotlist
        .get_hotlist(with_user(
            "nobody@test.com",
            GetHotlistRequest {
                hotlist_id: hl.hotlist_id,
            },
        ))
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_permission_allowed_hotlist_with_acl() {
    let fixture = TestFixture::new().await;
    let mut hotlist = fixture.hotlist_client();
    let mut acl = fixture.acl_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Shared Hotlist".to_string(),
            description: String::new(),
            owner: "owner@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    // Grant HOTLIST_VIEW to viewer
    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: IdentityType::User as i32,
        identity_value: "viewer@test.com".to_string(),
        permission: HotlistPermission::HotlistView as i32,
    })
    .await
    .unwrap();

    // viewer can get the hotlist
    let resp = hotlist
        .get_hotlist(with_user(
            "viewer@test.com",
            GetHotlistRequest {
                hotlist_id: hl.hotlist_id,
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.name, "Shared Hotlist");
}

// ── List filtering by permissions ───────────────────────────────────────

#[tokio::test]
async fn test_list_components_filters_by_permission() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    // Create 3 components as admin
    let c1 = create_component(&mut comp, &mut acl, "visible-comp", None).await;
    let _c2 = create_component(&mut comp, &mut acl, "hidden-comp", None).await;
    let c3 = create_component(&mut comp, &mut acl, "also-visible", None).await;

    // Grant alice VIEW on c1 and c3 only
    acl.set_component_acl(SetComponentAclRequest {
        component_id: c1,
        identity_type: 1,
        identity_value: "alice@test.com".to_string(),
        permissions: vec![1], // VIEW_ISSUES
    })
    .await
    .unwrap();
    acl.set_component_acl(SetComponentAclRequest {
        component_id: c3,
        identity_type: 1,
        identity_value: "alice@test.com".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    // Admin sees all 3
    let admin_list = comp
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(admin_list.components.len(), 3);

    // Alice sees only 2
    let mut alice_comp = ComponentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let alice_list = alice_comp
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(alice_list.components.len(), 2);
    let names: Vec<&str> = alice_list
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(names.contains(&"visible-comp"));
    assert!(names.contains(&"also-visible"));
    assert!(!names.contains(&"hidden-comp"));

    // Outsider sees none
    let mut outsider = ComponentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("outsider@test.com".to_string()),
    );
    let outsider_list = outsider
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(outsider_list.components.len(), 0);
}

#[tokio::test]
async fn test_list_hotlists_filters_by_permission() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();

    // Create 2 hotlists
    let h1 = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Visible HL".to_string(),
            description: String::new(),
            owner: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    let h2 = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Hidden HL".to_string(),
            description: String::new(),
            owner: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, h1.hotlist_id).await;
    grant_hotlist_admin(&mut acl, h2.hotlist_id).await;

    // Grant alice VIEW on h1 only
    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: h1.hotlist_id,
        identity_type: 1,
        identity_value: "alice@test.com".to_string(),
        permission: 1,
    })
    .await
    .unwrap();

    // Admin sees both
    let admin_list = hotlist
        .list_hotlists(ListHotlistsRequest {
            page_size: 10,
            page_token: String::new(),
            filter: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(admin_list.hotlists.len(), 2);

    // Alice sees only 1
    let mut alice = HotlistServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let alice_list = alice
        .list_hotlists(ListHotlistsRequest {
            page_size: 10,
            page_token: String::new(),
            filter: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(alice_list.hotlists.len(), 1);
    assert_eq!(alice_list.hotlists[0].name, "Visible HL");

    // Outsider sees none
    let mut outsider = HotlistServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("outsider@test.com".to_string()),
    );
    let outsider_list = outsider
        .list_hotlists(ListHotlistsRequest {
            page_size: 10,
            page_token: String::new(),
            filter: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(outsider_list.hotlists.len(), 0);
}

#[tokio::test]
async fn test_search_filters_by_permission() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    // Create 2 components
    let c1 = create_component(&mut comp, &mut acl, "search-visible", None).await;
    let c2 = create_component(&mut comp, &mut acl, "search-hidden", None).await;

    // Create issues in both
    create_issue(&mut issue, c1, "Visible Bug").await;
    create_issue(&mut issue, c2, "Hidden Bug").await;

    // Grant alice VIEW on c1 only
    acl.set_component_acl(SetComponentAclRequest {
        component_id: c1,
        identity_type: 1,
        identity_value: "alice@test.com".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    // Admin search returns both
    let mut admin_search = f.search_client();
    let admin_results = admin_search
        .search_issues(SearchIssuesRequest {
            query: "Bug".to_string(),
            page_size: 10,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(admin_results.issues.len(), 2);

    // Alice search returns only the visible one
    let mut alice_search = SearchServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let alice_results = alice_search
        .search_issues(SearchIssuesRequest {
            query: "Bug".to_string(),
            page_size: 10,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(alice_results.issues.len(), 1);
    assert_eq!(alice_results.issues[0].title, "Visible Bug");

    // Outsider search returns nothing
    let mut outsider = SearchServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("outsider@test.com".to_string()),
    );
    let outsider_results = outsider
        .search_issues(SearchIssuesRequest {
            query: "Bug".to_string(),
            page_size: 10,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(outsider_results.issues.len(), 0);
}

#[tokio::test]
async fn test_unauthenticated_list_returns_empty() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    create_component(&mut comp, &mut acl, "some-comp", None).await;

    // Unauthenticated user sees nothing
    let mut unauth = f.unauthenticated_component_client();
    let list = unauth
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list.components.len(), 0);
}

// ── ACL Authorization Enforcement Tests ──────────────────────────────────────

#[tokio::test]
async fn test_set_component_acl_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let comp_id = create_component(&mut comp, &mut acl, "Auth Test", None).await;

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .set_component_acl(SetComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "evil@test.com".to_string(),
            permissions: vec![ComponentPermission::AdminComponents as i32],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_get_component_acl_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let comp_id = create_component(&mut comp, &mut acl, "Auth Test", None).await;

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .get_component_acl(GetComponentAclRequest {
            component_id: comp_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_remove_component_acl_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let comp_id = create_component(&mut comp, &mut acl, "Auth Test", None).await;

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .remove_component_acl(RemoveComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_check_component_permission_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let comp_id = create_component(&mut comp, &mut acl, "Auth Test", None).await;

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "someone@test.com".to_string(),
            issue_id: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_set_component_acl_non_admin_denied() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let comp_id = create_component(&mut comp, &mut acl, "Auth Test", None).await;

    // Give alice only VIEW permission
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: IdentityType::User as i32,
        identity_value: "alice@test.com".to_string(),
        permissions: vec![ComponentPermission::ViewIssues as i32],
    })
    .await
    .unwrap();

    // alice tries to modify ACL -- should be denied
    let mut alice_acl = AclServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let err = alice_acl
        .set_component_acl(SetComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "alice@test.com".to_string(),
            permissions: vec![ComponentPermission::AdminComponents as i32],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_set_hotlist_acl_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Auth Test Hotlist".to_string(),
            description: String::new(),
            owner: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .set_hotlist_acl(SetHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
            identity_type: IdentityType::User as i32,
            identity_value: "evil@test.com".to_string(),
            permission: HotlistPermission::HotlistAdmin as i32,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_get_hotlist_acl_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Auth Test Hotlist".to_string(),
            description: String::new(),
            owner: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .get_hotlist_acl(GetHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_remove_hotlist_acl_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Auth Test Hotlist".to_string(),
            description: String::new(),
            owner: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut unauth_acl = f.unauthenticated_acl_client();
    let err = unauth_acl
        .remove_hotlist_acl(RemoveHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
            identity_type: IdentityType::User as i32,
            identity_value: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_create_hotlist_unauthenticated_denied() {
    let f = TestFixture::new().await;
    let mut unauth_hotlist = f.unauthenticated_hotlist_client();

    let err = unauth_hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Evil Hotlist".to_string(),
            description: String::new(),
            owner: "attacker@test.com".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_create_hotlist_owner_set_from_auth() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();

    // Try to set owner to someone else -- should be overridden to admin@test.com
    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Owner Override Test".to_string(),
            description: String::new(),
            owner: "someone_else@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(hl.owner, TEST_ADMIN_USER);
}

#[tokio::test]
async fn test_create_hotlist_auto_grants_admin() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Auto Admin Test".to_string(),
            description: String::new(),
            owner: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    // Creator should be able to get ACL (requires HOTLIST_ADMIN)
    let acl_resp = acl
        .get_hotlist_acl(GetHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();

    assert!(acl_resp
        .entries
        .iter()
        .any(|e| e.identity_value == TEST_ADMIN_USER
            && e.permission == HotlistPermission::HotlistAdmin as i32));
}

#[tokio::test]
async fn test_remove_component_acl_error_redacted() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let comp_id = create_component(&mut comp, &mut acl, "Redact Test", None).await;

    let err = acl
        .remove_component_acl(RemoveComponentAclRequest {
            component_id: comp_id,
            identity_type: IdentityType::User as i32,
            identity_value: "nonexistent@secret.com".to_string(),
        })
        .await
        .unwrap_err();

    // Error message should NOT contain the identity value or component ID
    let msg = err.message().to_string();
    assert!(
        !msg.contains("nonexistent@secret.com"),
        "error leaked identity_value"
    );
    assert!(
        !msg.contains(&comp_id.to_string()),
        "error leaked component_id"
    );
}

#[tokio::test]
async fn test_remove_hotlist_acl_error_redacted() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Redact Hotlist Test".to_string(),
            description: String::new(),
            owner: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    let err = acl
        .remove_hotlist_acl(RemoveHotlistAclRequest {
            hotlist_id: hl.hotlist_id,
            identity_type: IdentityType::User as i32,
            identity_value: "nonexistent@secret.com".to_string(),
        })
        .await
        .unwrap_err();

    let msg = err.message().to_string();
    assert!(
        !msg.contains("nonexistent@secret.com"),
        "error leaked identity_value"
    );
    assert!(
        !msg.contains(&hl.hotlist_id.to_string()),
        "error leaked hotlist_id"
    );
}
