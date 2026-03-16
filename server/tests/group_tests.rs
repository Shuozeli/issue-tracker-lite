#[allow(dead_code, unused_imports)]
mod common;
use common::*;

use issuetracker_server::proto::comment_service_client::CommentServiceClient;
use issuetracker_server::proto::component_service_client::ComponentServiceClient;
use issuetracker_server::proto::hotlist_service_client::HotlistServiceClient;
use issuetracker_server::proto::issue_service_client::IssueServiceClient;
use issuetracker_server::proto::search_service_client::SearchServiceClient;

// ── Group Service Tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_group_crud() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    // Create
    let created = group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: "Engineering Team".to_string(),
            description: "All engineers".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.name, "eng-team");
    assert_eq!(created.display_name, "Engineering Team");
    assert_eq!(created.creator, TEST_ADMIN_USER);

    // Get
    let fetched = group
        .get_group(identity_proto::GetGroupRequest {
            name: "eng-team".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fetched.name, "eng-team");
    assert_eq!(fetched.description, "All engineers");

    // Update
    let updated = group
        .update_group(identity_proto::UpdateGroupRequest {
            name: "eng-team".to_string(),
            display_name: Some("Eng Team".to_string()),
            description: None,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.display_name, "Eng Team");
    assert_eq!(updated.description, "All engineers"); // unchanged

    // List
    let list_resp = group
        .list_groups(identity_proto::ListGroupsRequest {
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list_resp.groups.len(), 1);
    assert_eq!(list_resp.groups[0].name, "eng-team");

    // Delete
    group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "eng-team".to_string(),
        })
        .await
        .unwrap();

    // Get after delete -> not found
    let err = group
        .get_group(identity_proto::GetGroupRequest {
            name: "eng-team".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_group_duplicate_name_rejected() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "frontend".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let err = group
        .create_group(identity_proto::CreateGroupRequest {
            name: "frontend".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn test_group_name_validation() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    // Too short
    let err = group
        .create_group(identity_proto::CreateGroupRequest {
            name: "ab".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);

    // Leading hyphen
    let err = group
        .create_group(identity_proto::CreateGroupRequest {
            name: "-bad-name".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);

    // Uppercase
    let err = group
        .create_group(identity_proto::CreateGroupRequest {
            name: "BadName".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_group_member_management() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    // Add user member
    let member = group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1, // USER
            member_value: "alice@test.com".to_string(),
            role: 1, // MEMBER
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(member.member_value, "alice@test.com");
    assert_eq!(member.role, 1); // MEMBER

    // Add another user
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "bob@test.com".to_string(),
            role: 2, // MANAGER
        })
        .await
        .unwrap();

    // List members
    let members = group
        .list_members(identity_proto::ListMembersRequest {
            group_name: "eng-team".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(members.members.len(), 2);

    // Update member role
    let updated = group
        .update_member_role(identity_proto::UpdateMemberRoleRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 3, // OWNER
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.role, 3);

    // Remove member
    group
        .remove_member(identity_proto::RemoveMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "bob@test.com".to_string(),
        })
        .await
        .unwrap();

    let members = group
        .list_members(identity_proto::ListMembersRequest {
            group_name: "eng-team".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(members.members.len(), 1);
    assert_eq!(members.members[0].member_value, "alice@test.com");
}

#[tokio::test]
async fn test_group_duplicate_member_rejected() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let err = group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn test_group_nested_membership() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    // Create groups: frontend -> engineering -> all-staff
    for name in &["frontend", "engineering", "all-staff"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    // alice -> frontend
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "frontend".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    // frontend -> engineering (group member)
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "engineering".to_string(),
            member_type: 2,
            member_value: "frontend".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    // engineering -> all-staff (group member)
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "all-staff".to_string(),
            member_type: 2,
            member_value: "engineering".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    // Resolve alice's groups: should include frontend, engineering, all-staff
    let resolved = group
        .resolve_user_groups(identity_proto::ResolveUserGroupsRequest {
            user_id: "alice@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resolved.groups.len(), 3);
    assert!(resolved.groups.contains(&"frontend".to_string()));
    assert!(resolved.groups.contains(&"engineering".to_string()));
    assert!(resolved.groups.contains(&"all-staff".to_string()));

    // is_member checks
    let is_member = group
        .is_member(identity_proto::IsMemberRequest {
            user_id: "alice@test.com".to_string(),
            group_name: "all-staff".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(is_member.is_member);

    let not_member = group
        .is_member(identity_proto::IsMemberRequest {
            user_id: "bob@test.com".to_string(),
            group_name: "all-staff".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!not_member.is_member);
}

#[tokio::test]
async fn test_group_nesting_depth_limit() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    // Create 5 levels: grp-1 -> grp-2 -> grp-3 -> grp-4 -> grp-5
    for i in 1..=5 {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: format!("grp-{i}"),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    // user -> grp-1
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "grp-1".to_string(),
            member_type: 1,
            member_value: "deep@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    // grp-1 -> grp-2, grp-2 -> grp-3, grp-3 -> grp-4, grp-4 -> grp-5
    for i in 1..5 {
        group
            .add_member(identity_proto::AddMemberRequest {
                group_name: format!("grp-{}", i + 1),
                member_type: 2,
                member_value: format!("grp-{i}"),
                role: 1,
            })
            .await
            .unwrap();
    }

    // Resolve: max depth is 3
    let resolved = group
        .resolve_user_groups(identity_proto::ResolveUserGroupsRequest {
            user_id: "deep@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(resolved.groups.contains(&"grp-1".to_string()));
    assert!(resolved.groups.contains(&"grp-2".to_string()));
    assert!(resolved.groups.contains(&"grp-3".to_string()));
    // Should not get all 5
    assert!(resolved.groups.len() <= 4);
}

#[tokio::test]
async fn test_group_cycle_detection() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    for name in &["group-a", "group-b"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    // a -> b
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "group-b".to_string(),
            member_type: 2,
            member_value: "group-a".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    // Try b -> a (would create cycle)
    let err = group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "group-a".to_string(),
            member_type: 2,
            member_value: "group-b".to_string(),
            role: 1,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("cycle"));
}

#[tokio::test]
async fn test_group_delete_preconditions() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    // Set up a component ACL referencing the group
    let comp_id = create_component(&mut comp, &mut acl, "test-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    // Delete should fail: referenced in ComponentAcl
    let err = group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "eng-team".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);

    // Remove the ACL reference, then delete should succeed
    acl.remove_component_acl(RemoveComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
    })
    .await
    .unwrap();

    group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "eng-team".to_string(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_group_delete_member_of_another_group() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    for name in &["parent-group", "child-group"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    // child-group is member of parent-group
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "parent-group".to_string(),
            member_type: 2,
            member_value: "child-group".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    // Deleting child-group should fail
    let err = group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "child-group".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn test_group_batch_add_members() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let result = group
        .batch_add_members(identity_proto::BatchAddMembersRequest {
            group_name: "eng-team".to_string(),
            members: vec![
                identity_proto::BatchMemberEntry {
                    member_type: 1,
                    member_value: "alice@test.com".to_string(),
                    role: 1,
                },
                identity_proto::BatchMemberEntry {
                    member_type: 1,
                    member_value: "bob@test.com".to_string(),
                    role: 2,
                },
            ],
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(result.members.len(), 2);

    let members = group
        .list_members(identity_proto::ListMembersRequest {
            group_name: "eng-team".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(members.members.len(), 2);
}

// ── GROUP ACL Permission Integration Tests ──────────────────────────────

#[tokio::test]
async fn test_group_acl_grants_access() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "groupuser@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "group-test-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    let mut issue = f.issue_client();
    let created_issue = create_issue(&mut issue, comp_id, "Group Test Issue").await;

    let mut group_user_issue_client = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("groupuser@test.com".to_string()),
    );
    let result = group_user_issue_client
        .get_issue(GetIssueRequest {
            issue_id: created_issue.issue_id,
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().into_inner().title, "Group Test Issue");
}

#[tokio::test]
async fn test_group_acl_no_access_for_non_member() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "restricted-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
        permissions: vec![7],
    })
    .await
    .unwrap();

    let mut outsider_comp_client = ComponentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("outsider@test.com".to_string()),
    );
    let err = outsider_comp_client
        .get_component(GetComponentRequest {
            component_id: comp_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_group_nested_acl_grants_access() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    for name in &["frontend", "engineering"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "frontend".to_string(),
            member_type: 1,
            member_value: "nested-user@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "engineering".to_string(),
            member_type: 2,
            member_value: "frontend".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "nested-acl-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "engineering".to_string(),
        permissions: vec![ComponentPermission::ViewIssues as i32],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Nested ACL Issue").await;

    let mut user_issue_client = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("nested-user@test.com".to_string()),
    );
    let result = user_issue_client
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().into_inner().title, "Nested ACL Issue");
}

#[tokio::test]
async fn test_group_hotlist_acl_grants_access() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "hl-group".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "hl-group".to_string(),
            member_type: 1,
            member_value: "hl-member@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Group Hotlist".to_string(),
            description: String::new(),
            owner: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    grant_hotlist_admin(&mut acl, hl.hotlist_id).await;

    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: 2,
        identity_value: "hl-group".to_string(),
        permission: HotlistPermission::HotlistView as i32,
    })
    .await
    .unwrap();

    let mut member_hotlist_client = HotlistServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("hl-member@test.com".to_string()),
    );
    let result = member_hotlist_client
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().into_inner().issues.len(), 0);
}

#[tokio::test]
async fn test_group_event_log_entries() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut event_client = f.event_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "event-log-grp".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "event-log-grp".to_string(),
            member_type: 1,
            member_value: "eventuser@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let resp = event_client
        .list_events(ListEventsRequest {
            entity_type: "Group".to_string(),
            entity_id: 0,
            event_type: String::new(),
            actor: String::new(),
            since: None,
            until: None,
            page_size: 50,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(!resp.events.is_empty());
    assert!(resp.events.iter().any(|e| e.event_type == "GROUP_CREATED"));
    assert!(resp
        .events
        .iter()
        .any(|e| e.event_type == "GROUP_MEMBER_ADDED"));
}

#[tokio::test]
async fn test_group_list_pagination() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    for name in &["grp-aaa", "grp-bbb", "grp-ccc", "grp-ddd", "grp-eee"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    let page1 = group
        .list_groups(identity_proto::ListGroupsRequest {
            page_size: 2,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page1.groups.len(), 2);
    assert!(!page1.next_page_token.is_empty());

    let page2 = group
        .list_groups(identity_proto::ListGroupsRequest {
            page_size: 2,
            page_token: page1.next_page_token.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page2.groups.len(), 2);
    assert!(!page2.next_page_token.is_empty());

    let page3 = group
        .list_groups(identity_proto::ListGroupsRequest {
            page_size: 2,
            page_token: page2.next_page_token.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page3.groups.len(), 1);
    assert!(page3.next_page_token.is_empty());
}

#[tokio::test]
async fn test_group_remove_nonexistent_member() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "rm-test-grp".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let err = group
        .remove_member(identity_proto::RemoveMemberRequest {
            group_name: "rm-test-grp".to_string(),
            member_type: 1,
            member_value: "nonexistent@test.com".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_group_update_nonexistent() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    let err = group
        .update_group(identity_proto::UpdateGroupRequest {
            name: "does-not-exist".to_string(),
            display_name: Some("Ghost".to_string()),
            description: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_group_check_component_permission_with_groups() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "perm-check-grp".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "perm-check-grp".to_string(),
            member_type: 1,
            member_value: "perm-user@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "perm-check-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "perm-check-grp".to_string(),
        permissions: vec![ComponentPermission::EditIssues as i32],
    })
    .await
    .unwrap();

    let resp = acl
        .check_component_permission(CheckComponentPermissionRequest {
            component_id: comp_id,
            user_id: "perm-user@test.com".to_string(),
            issue_id: None,
        })
        .await
        .unwrap()
        .into_inner();

    assert!(!resp.permissions.is_empty());
    assert_eq!(resp.grant_source, "ACL");
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::EditIssues as i32)));
    assert!(resp
        .permissions
        .contains(&(ComponentPermission::ViewIssues as i32)));
}

#[tokio::test]
async fn test_group_batch_add_idempotent() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "batch-idem-grp".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "batch-idem-grp".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let result = group
        .batch_add_members(identity_proto::BatchAddMembersRequest {
            group_name: "batch-idem-grp".to_string(),
            members: vec![
                identity_proto::BatchMemberEntry {
                    member_type: 1,
                    member_value: "alice@test.com".to_string(),
                    role: 1,
                },
                identity_proto::BatchMemberEntry {
                    member_type: 1,
                    member_value: "bob@test.com".to_string(),
                    role: 1,
                },
            ],
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(result.members.len(), 2);

    let members = group
        .list_members(identity_proto::ListMembersRequest {
            group_name: "batch-idem-grp".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(members.members.len(), 2);
}

#[tokio::test]
async fn test_group_unauthenticated_rejected() {
    let f = TestFixture::new().await;
    let mut raw_client = f.unauthenticated_group_client();

    let err = raw_client
        .create_group(identity_proto::CreateGroupRequest {
            name: "unauth-grp".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_group_nested_acl_with_edit_permissions() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "dev-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "dev-team".to_string(),
            member_type: 1,
            member_value: "dev@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "edit-perm-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "dev-team".to_string(),
        permissions: vec![ComponentPermission::EditIssues as i32],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Edit Perm Issue").await;

    let mut user_issue_client = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("dev@test.com".to_string()),
    );
    let updated = user_issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            title: Some("Updated By Group Member".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.title, "Updated By Group Member");
}

#[tokio::test]
async fn test_group_acl_revocation_removes_access() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "revoke-test", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Visible Issue").await;

    let mut alice = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    alice
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await
        .unwrap();

    acl.remove_component_acl(RemoveComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
    })
    .await
    .unwrap();

    let err = alice
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_group_member_removal_revokes_access() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "eng-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "member-revoke", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "eng-team".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Test Issue").await;

    let mut alice = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    alice
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await
        .unwrap();

    group
        .remove_member(identity_proto::RemoveMemberRequest {
            group_name: "eng-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
        })
        .await
        .unwrap();

    let err = alice
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_group_multiple_acls_union_permissions() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    for name in &["viewers", "editors"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
        group
            .add_member(identity_proto::AddMemberRequest {
                group_name: name.to_string(),
                member_type: 1,
                member_value: "alice@test.com".to_string(),
                role: 1,
            })
            .await
            .unwrap();
    }

    let comp_id = create_component(&mut comp, &mut acl, "multi-acl-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "viewers".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "editors".to_string(),
        permissions: vec![3],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Multi ACL Issue").await;

    let mut alice = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let updated = alice
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            title: Some("Edited By Alice".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.title, "Edited By Alice");
}

#[tokio::test]
async fn test_group_and_user_acl_coexist() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "team-a".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "team-a".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "coexist-comp", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "team-a".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 1,
        identity_value: "alice@test.com".to_string(),
        permissions: vec![3],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Coexist Issue").await;

    let mut alice = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let updated = alice
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            title: Some("Edited".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.title, "Edited");

    acl.remove_component_acl(RemoveComponentAclRequest {
        component_id: comp_id,
        identity_type: 1,
        identity_value: "alice@test.com".to_string(),
    })
    .await
    .unwrap();

    alice
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await
        .unwrap();

    let err = alice
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            title: Some("Should Fail".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_group_acl_create_issues() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "creators".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "creators".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "create-test", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "creators".to_string(),
        permissions: vec![5, 1],
    })
    .await
    .unwrap();

    let mut alice = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let created = alice
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Created By Group Member".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("alice@test.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.title, "Created By Group Member");
}

#[tokio::test]
async fn test_group_acl_comment_on_issues() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "commenters".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "commenters".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "comment-test", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "commenters".to_string(),
        permissions: vec![2],
    })
    .await
    .unwrap();

    let created = create_issue(&mut issue, comp_id, "Comment Test Issue").await;

    let mut alice = CommentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let comment = alice
        .create_comment(CreateCommentRequest {
            issue_id: created.issue_id,
            body: "Comment from group member".to_string(),
            author: "alice@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(comment.body, "Comment from group member");
    assert_eq!(comment.author, "alice@test.com");
}

#[tokio::test]
async fn test_group_acl_admin_components() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "comp-admins".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "comp-admins".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "admin-test", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "comp-admins".to_string(),
        permissions: vec![7],
    })
    .await
    .unwrap();

    let mut alice = ComponentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let updated = alice
        .update_component(UpdateComponentRequest {
            component_id: comp_id,
            name: Some("renamed-by-group-admin".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.name, "renamed-by-group-admin");
}

#[tokio::test]
async fn test_group_self_cycle_rejected() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "self-ref".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let err = group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "self-ref".to_string(),
            member_type: 2,
            member_value: "self-ref".to_string(),
            role: 1,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("cycle"));
}

#[tokio::test]
async fn test_group_three_level_cycle_detection() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    for name in &["cycle-a", "cycle-b", "cycle-c"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "cycle-b".to_string(),
            member_type: 2,
            member_value: "cycle-a".to_string(),
            role: 1,
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "cycle-c".to_string(),
            member_type: 2,
            member_value: "cycle-b".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let err = group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "cycle-a".to_string(),
            member_type: 2,
            member_value: "cycle-c".to_string(),
            role: 1,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("cycle"));
}

#[tokio::test]
async fn test_group_hotlist_acl_view_append() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();
    let mut hotlist = f.hotlist_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "hotlist-editors".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "hotlist-editors".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "hl-test-comp", None).await;
    let created_issue = create_issue(&mut issue, comp_id, "HL Test Issue").await;

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Priority Queue".to_string(),
            description: String::new(),
            owner: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    grant_hotlist_admin(&mut acl, hl.hotlist_id).await;

    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: 2,
        identity_value: "hotlist-editors".to_string(),
        permission: 2,
    })
    .await
    .unwrap();

    let mut alice_hotlist = HotlistServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    alice_hotlist
        .add_issue(AddIssueToHotlistRequest {
            hotlist_id: hl.hotlist_id,
            issue_id: created_issue.issue_id,
            added_by: "alice@test.com".to_string(),
        })
        .await
        .unwrap();

    let issues = alice_hotlist
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(issues.issues.len(), 1);
    assert_eq!(issues.issues[0].issue_id, created_issue.issue_id);
}

#[tokio::test]
async fn test_group_resolve_empty() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    let resolved = group
        .resolve_user_groups(identity_proto::ResolveUserGroupsRequest {
            user_id: "nobody@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(resolved.groups.is_empty());
}

#[tokio::test]
async fn test_group_resolve_multiple_direct_memberships() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    for name in &["team-red", "team-blue", "team-green"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
        group
            .add_member(identity_proto::AddMemberRequest {
                group_name: name.to_string(),
                member_type: 1,
                member_value: "multi@test.com".to_string(),
                role: 1,
            })
            .await
            .unwrap();
    }

    let resolved = group
        .resolve_user_groups(identity_proto::ResolveUserGroupsRequest {
            user_id: "multi@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resolved.groups.len(), 3);
    assert!(resolved.groups.contains(&"team-red".to_string()));
    assert!(resolved.groups.contains(&"team-blue".to_string()));
    assert!(resolved.groups.contains(&"team-green".to_string()));
}

#[tokio::test]
async fn test_group_delete_and_recreate() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "ephemeral".to_string(),
            display_name: "V1".to_string(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "ephemeral".to_string(),
            member_type: 1,
            member_value: "user@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "ephemeral".to_string(),
        })
        .await
        .unwrap();

    let recreated = group
        .create_group(identity_proto::CreateGroupRequest {
            name: "ephemeral".to_string(),
            display_name: "V2".to_string(),
            description: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(recreated.display_name, "V2");

    let members = group
        .list_members(identity_proto::ListMembersRequest {
            group_name: "ephemeral".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(members.members.is_empty());
}

#[tokio::test]
async fn test_group_batch_add_with_cycle_rejected() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    for name in &["batch-a", "batch-b"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }

    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "batch-b".to_string(),
            member_type: 2,
            member_value: "batch-a".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let err = group
        .batch_add_members(identity_proto::BatchAddMembersRequest {
            group_name: "batch-a".to_string(),
            members: vec![
                identity_proto::BatchMemberEntry {
                    member_type: 1,
                    member_value: "user@test.com".to_string(),
                    role: 1,
                },
                identity_proto::BatchMemberEntry {
                    member_type: 2,
                    member_value: "batch-b".to_string(),
                    role: 1,
                },
            ],
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("cycle"));
}

#[tokio::test]
async fn test_group_nested_hotlist_acl() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut acl = f.acl_client();
    let mut hotlist = f.hotlist_client();

    for name in &["inner-team", "outer-team"] {
        group
            .create_group(identity_proto::CreateGroupRequest {
                name: name.to_string(),
                display_name: String::new(),
                description: String::new(),
            })
            .await
            .unwrap();
    }
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "inner-team".to_string(),
            member_type: 1,
            member_value: "nested@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "outer-team".to_string(),
            member_type: 2,
            member_value: "inner-team".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Nested HL".to_string(),
            description: String::new(),
            owner: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    grant_hotlist_admin(&mut acl, hl.hotlist_id).await;

    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: 2,
        identity_value: "outer-team".to_string(),
        permission: 1,
    })
    .await
    .unwrap();

    let mut nested = HotlistServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("nested@test.com".to_string()),
    );
    let issues = nested
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(issues.issues.is_empty());

    let mut outsider = HotlistServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("outsider@test.com".to_string()),
    );
    let err = outsider
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: hl.hotlist_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_group_delete_blocked_by_hotlist_acl() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut acl = f.acl_client();
    let mut hotlist = f.hotlist_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "hl-group".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Blocked HL".to_string(),
            description: String::new(),
            owner: TEST_ADMIN_USER.to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    grant_hotlist_admin(&mut acl, hl.hotlist_id).await;

    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: 2,
        identity_value: "hl-group".to_string(),
        permission: 1,
    })
    .await
    .unwrap();

    let err = group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "hl-group".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);

    acl.remove_hotlist_acl(RemoveHotlistAclRequest {
        hotlist_id: hl.hotlist_id,
        identity_type: 2,
        identity_value: "hl-group".to_string(),
    })
    .await
    .unwrap();
    group
        .delete_group(identity_proto::DeleteGroupRequest {
            name: "hl-group".to_string(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_group_add_nonexistent_group_member() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "real-group".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();

    let err = group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "real-group".to_string(),
            member_type: 2,
            member_value: "ghost-group".to_string(),
            role: 1,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_group_full_org_scenario() {
    let f = TestFixture::new().await;
    let mut group = f.group_client();
    let mut comp = f.component_client();
    let mut acl = f.acl_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "frontend".to_string(),
            display_name: "Frontend Team".to_string(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "engineering".to_string(),
            display_name: "Engineering".to_string(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "engineering".to_string(),
            member_type: 2,
            member_value: "frontend".to_string(),
            role: 1,
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "frontend".to_string(),
            member_type: 1,
            member_value: "dev@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let comp_id = create_component(&mut comp, &mut acl, "web-app", None).await;
    acl.set_component_acl(SetComponentAclRequest {
        component_id: comp_id,
        identity_type: 2,
        identity_value: "engineering".to_string(),
        permissions: vec![5, 3],
    })
    .await
    .unwrap();

    let mut dev = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("dev@test.com".to_string()),
    );
    let created = dev
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Button misaligned".to_string(),
            description: "On mobile view".to_string(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("dev@test.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    let updated = dev
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            assignee: Some("dev@test.com".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.assignee, "dev@test.com");

    let mut dev_comment = CommentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("dev@test.com".to_string()),
    );
    dev_comment
        .create_comment(CreateCommentRequest {
            issue_id: created.issue_id,
            body: "Working on this now".to_string(),
            author: "dev@test.com".to_string(),
        })
        .await
        .unwrap();

    let mut outsider = IssueServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("outsider@test.com".to_string()),
    );
    let err = outsider
        .get_issue(GetIssueRequest {
            issue_id: created.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);

    let mut admin_issue = f.issue_client();
    admin_issue
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            assignee: Some(String::new()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    group
        .remove_member(identity_proto::RemoveMemberRequest {
            group_name: "frontend".to_string(),
            member_type: 1,
            member_value: "dev@test.com".to_string(),
        })
        .await
        .unwrap();

    let err = dev
        .update_issue(UpdateIssueRequest {
            issue_id: created.issue_id,
            title: Some("Should Fail".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_list_components_filters_by_group_permission() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "dev-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "dev-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let c1 = create_component(&mut comp, &mut acl, "group-visible", None).await;
    let _c2 = create_component(&mut comp, &mut acl, "group-hidden", None).await;

    acl.set_component_acl(SetComponentAclRequest {
        component_id: c1,
        identity_type: 2,
        identity_value: "dev-team".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    let mut alice = ComponentServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let list = alice
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list.components.len(), 1);
    assert_eq!(list.components[0].name, "group-visible");
}

#[tokio::test]
async fn test_search_with_group_acl_filtering() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut acl = f.acl_client();
    let mut issue = f.issue_client();
    let mut group = f.group_client();

    group
        .create_group(identity_proto::CreateGroupRequest {
            name: "search-team".to_string(),
            display_name: String::new(),
            description: String::new(),
        })
        .await
        .unwrap();
    group
        .add_member(identity_proto::AddMemberRequest {
            group_name: "search-team".to_string(),
            member_type: 1,
            member_value: "alice@test.com".to_string(),
            role: 1,
        })
        .await
        .unwrap();

    let c1 = create_component(&mut comp, &mut acl, "team-comp", None).await;
    let c2 = create_component(&mut comp, &mut acl, "other-comp", None).await;

    create_issue(&mut issue, c1, "Team Issue").await;
    create_issue(&mut issue, c2, "Other Issue").await;

    acl.set_component_acl(SetComponentAclRequest {
        component_id: c1,
        identity_type: 2,
        identity_value: "search-team".to_string(),
        permissions: vec![1],
    })
    .await
    .unwrap();

    let mut alice = SearchServiceClient::with_interceptor(
        f.channel.clone(),
        UserInterceptor("alice@test.com".to_string()),
    );
    let results = alice
        .search_issues(SearchIssuesRequest {
            query: "Issue".to_string(),
            page_size: 10,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(results.issues.len(), 1);
    assert_eq!(results.issues[0].title, "Team Issue");
}
