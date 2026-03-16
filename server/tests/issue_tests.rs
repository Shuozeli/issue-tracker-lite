mod common;
use common::*;

// ── 8.2 Issue Lifecycle Tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_create_issue_minimal() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Minimal issue").await;

    assert_eq!(issue.status, Status::New as i32);
    assert_eq!(issue.title, "Minimal issue");
    assert!(issue.issue_id > 0);
}

#[tokio::test]
async fn test_create_issue_all_fields() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    let issue = issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Full issue".to_string(),
            description: "Detailed description".to_string(),
            priority: Priority::P0 as i32,
            r#type: IssueType::FeatureRequest as i32,
            severity: Some(Severity::S0 as i32),
            assignee: Some("dev@example.com".to_string()),
            reporter: Some("reporter@example.com".to_string()),
            verifier: Some("verifier@example.com".to_string()),
            found_in: Some("v1.0".to_string()),
            targeted_to: Some("v2.0".to_string()),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(issue.title, "Full issue");
    assert_eq!(issue.priority, Priority::P0 as i32);
    assert_eq!(issue.r#type, IssueType::FeatureRequest as i32);
    assert_eq!(issue.reporter, "reporter@example.com");
}

#[tokio::test]
async fn test_create_issue_invalid_component_fails() {
    let f = TestFixture::new().await;
    let mut issue_client = f.issue_client();

    let err = issue_client
        .create_issue(CreateIssueRequest {
            component_id: 999999,
            title: "Orphan issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            severity: None,
            assignee: None,
            reporter: Some("test@example.com".to_string()),
            verifier: None,
            found_in: None,
            targeted_to: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_update_issue_partial() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Original title").await;

    let updated = issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            title: Some("New title".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.title, "New title");
    assert_eq!(updated.priority, Priority::P2 as i32);
}

#[tokio::test]
async fn test_status_new_to_assigned_on_assignee_set() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test").await;
    assert_eq!(issue.status, Status::New as i32);

    let updated = issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            assignee: Some("dev@example.com".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.status, Status::Assigned as i32);
    assert_eq!(updated.assignee, "dev@example.com");
}

#[tokio::test]
async fn test_status_assigned_to_new_on_assignee_clear() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test").await;

    // Assign
    issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            assignee: Some("dev@example.com".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    // Clear assignee
    let updated = issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            assignee: Some(String::new()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.status, Status::New as i32);
    assert_eq!(updated.assignee, "");
}

#[tokio::test]
async fn test_list_issues_by_component() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp1 = create_component(&mut comp_client, &mut acl, "Comp1", None).await;
    let comp2 = create_component(&mut comp_client, &mut acl, "Comp2", None).await;

    create_issue(&mut issue_client, comp1, "Issue in comp1").await;
    create_issue(&mut issue_client, comp1, "Another in comp1").await;
    create_issue(&mut issue_client, comp2, "Issue in comp2").await;

    let resp1 = issue_client
        .list_issues(ListIssuesRequest {
            component_id: comp1,
            page_size: 50,
            page_token: String::new(),
            status_filter: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp1.issues.len(), 2);

    let resp2 = issue_client
        .list_issues(ListIssuesRequest {
            component_id: comp2,
            page_size: 50,
            page_token: String::new(),
            status_filter: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp2.issues.len(), 1);
}

// ── 8.4 Relationship Tests ────────────────────────────────────────────

#[tokio::test]
async fn test_parent_child_basic() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let parent = create_issue(&mut issue_client, comp_id, "Parent issue").await;
    let child = create_issue(&mut issue_client, comp_id, "Child issue").await;

    issue_client
        .add_parent(AddParentRequest {
            child_id: child.issue_id,
            parent_id: parent.issue_id,
        })
        .await
        .unwrap();

    let children = issue_client
        .list_children(ListRelatedIssuesRequest {
            issue_id: parent.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(children.issues.len(), 1);
    assert_eq!(children.issues[0].issue_id, child.issue_id);

    let parents = issue_client
        .list_parents(ListRelatedIssuesRequest {
            issue_id: child.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(parents.issues.len(), 1);
    assert_eq!(parents.issues[0].issue_id, parent.issue_id);
}

#[tokio::test]
async fn test_parent_child_cycle_rejected() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let a = create_issue(&mut issue_client, comp_id, "A").await;
    let b = create_issue(&mut issue_client, comp_id, "B").await;
    let c = create_issue(&mut issue_client, comp_id, "C").await;

    // A -> B -> C
    issue_client
        .add_parent(AddParentRequest {
            child_id: b.issue_id,
            parent_id: a.issue_id,
        })
        .await
        .unwrap();
    issue_client
        .add_parent(AddParentRequest {
            child_id: c.issue_id,
            parent_id: b.issue_id,
        })
        .await
        .unwrap();

    // C -> A would create cycle
    let err = issue_client
        .add_parent(AddParentRequest {
            child_id: a.issue_id,
            parent_id: c.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn test_parent_child_self_reference_rejected() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let a = create_issue(&mut issue_client, comp_id, "A").await;

    let err = issue_client
        .add_parent(AddParentRequest {
            child_id: a.issue_id,
            parent_id: a.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_blocking_basic() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let a = create_issue(&mut issue_client, comp_id, "Blocker").await;
    let b = create_issue(&mut issue_client, comp_id, "Blocked").await;

    issue_client
        .add_blocking(AddBlockingRequest {
            blocking_id: a.issue_id,
            blocked_id: b.issue_id,
        })
        .await
        .unwrap();

    // Verify by removing (if it exists, remove succeeds)
    issue_client
        .remove_blocking(RemoveBlockingRequest {
            blocking_id: a.issue_id,
            blocked_id: b.issue_id,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_duplicate_mark() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let canonical = create_issue(&mut issue_client, comp_id, "Canonical").await;
    let dup = create_issue(&mut issue_client, comp_id, "Duplicate").await;

    let marked = issue_client
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: dup.issue_id,
            canonical_id: canonical.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(marked.status, Status::Duplicate as i32);

    let can = issue_client
        .get_issue(GetIssueRequest {
            issue_id: canonical.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(can.duplicate_count, 1);
}

#[tokio::test]
async fn test_duplicate_unmark() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let canonical = create_issue(&mut issue_client, comp_id, "Canonical").await;
    let dup = create_issue(&mut issue_client, comp_id, "Duplicate").await;

    issue_client
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: dup.issue_id,
            canonical_id: canonical.issue_id,
        })
        .await
        .unwrap();

    let unmarked = issue_client
        .unmark_duplicate(UnmarkDuplicateRequest {
            issue_id: dup.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_ne!(unmarked.status, Status::Duplicate as i32);
}

#[tokio::test]
async fn test_duplicate_cascade_count() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let canonical = create_issue(&mut issue_client, comp_id, "Canonical").await;
    let dup1 = create_issue(&mut issue_client, comp_id, "Dup1").await;
    let dup2 = create_issue(&mut issue_client, comp_id, "Dup2").await;

    issue_client
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: dup1.issue_id,
            canonical_id: canonical.issue_id,
        })
        .await
        .unwrap();

    issue_client
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: dup2.issue_id,
            canonical_id: canonical.issue_id,
        })
        .await
        .unwrap();

    let can = issue_client
        .get_issue(GetIssueRequest {
            issue_id: canonical.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(can.duplicate_count, 2);
}

// ── Status Machine Transition Tests ─────────────────────────────────────

#[tokio::test]
async fn test_status_new_to_in_progress() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;
    assert_eq!(i.status, Status::New as i32);

    let updated = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::InProgress as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.status, Status::InProgress as i32);
}

#[tokio::test]
async fn test_status_in_progress_to_fixed() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    // NEW -> IN_PROGRESS -> FIXED
    issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::InProgress as i32),
            ..Default::default()
        })
        .await
        .unwrap();
    let fixed = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::Fixed as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fixed.status, Status::Fixed as i32);
    assert!(fixed.resolve_time.is_some());
}

#[tokio::test]
async fn test_status_fixed_to_verified() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    // NEW -> FIXED -> FIXED_VERIFIED
    issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::Fixed as i32),
            ..Default::default()
        })
        .await
        .unwrap();
    let verified = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::FixedVerified as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(verified.status, Status::FixedVerified as i32);
    assert!(verified.verify_time.is_some());
}

#[tokio::test]
async fn test_status_closed_to_new_reopen() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    // NEW -> FIXED -> NEW (reopen)
    issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::Fixed as i32),
            ..Default::default()
        })
        .await
        .unwrap();
    let reopened = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::New as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(reopened.status, Status::New as i32);
}

#[tokio::test]
async fn test_status_new_to_wont_fix() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    // Test all 4 WONT_FIX variants from NEW
    for wf_status in [
        Status::WontFixInfeasible,
        Status::WontFixNotReproducible,
        Status::WontFixObsolete,
        Status::WontFixIntendedBehavior,
    ] {
        let i = create_issue(&mut issue, cid, "Bug").await;
        let updated = issue
            .update_issue(UpdateIssueRequest {
                issue_id: i.issue_id,
                status: Some(wf_status as i32),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(updated.status, wf_status as i32);
        assert!(updated.resolve_time.is_some());
    }
}

#[tokio::test]
async fn test_status_invalid_transition_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    // NEW -> FIXED_VERIFIED is not allowed (must go through FIXED first)
    let err = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::FixedVerified as i32),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn test_status_in_progress_to_new_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::InProgress as i32),
            ..Default::default()
        })
        .await
        .unwrap();

    // IN_PROGRESS -> NEW is not allowed
    let err = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::New as i32),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

// ── List Issues Status Filters ──────────────────────────────────────────

#[tokio::test]
async fn test_list_issues_status_filter_closed() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    let open = create_issue(&mut issue, cid, "Open").await;
    let closed = create_issue(&mut issue, cid, "Closed").await;
    issue
        .update_issue(UpdateIssueRequest {
            issue_id: closed.issue_id,
            status: Some(Status::Fixed as i32),
            ..Default::default()
        })
        .await
        .unwrap();

    // Default (open) filter should return only the open issue
    let open_list = issue
        .list_issues(ListIssuesRequest {
            component_id: cid,
            status_filter: String::new(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(open_list.issues.len(), 1);
    assert_eq!(open_list.issues[0].issue_id, open.issue_id);

    // "closed" filter should return only the fixed issue
    let closed_list = issue
        .list_issues(ListIssuesRequest {
            component_id: cid,
            status_filter: "closed".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(closed_list.issues.len(), 1);
    assert_eq!(closed_list.issues[0].issue_id, closed.issue_id);

    // "all" filter should return both
    let all_list = issue
        .list_issues(ListIssuesRequest {
            component_id: cid,
            status_filter: "all".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(all_list.issues.len(), 2);
}

// ── Update Issue Multiple Fields ────────────────────────────────────────

#[tokio::test]
async fn test_update_issue_multiple_fields_at_once() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Original").await;

    let updated = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            title: Some("New Title".to_string()),
            priority: Some(Priority::P0 as i32),
            severity: Some(Severity::S0 as i32),
            assignee: Some("dev@test.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(updated.title, "New Title");
    assert_eq!(updated.priority, Priority::P0 as i32);
    assert_eq!(updated.severity, Severity::S0 as i32);
    assert_eq!(updated.assignee, "dev@test.com");
    // Should auto-transition to ASSIGNED since assignee was set
    assert_eq!(updated.status, Status::Assigned as i32);
}

// ── Blocking Relationship Edge Cases ────────────────────────────────────

#[tokio::test]
async fn test_blocking_self_reference_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let err = issue
        .add_blocking(AddBlockingRequest {
            blocking_id: i.issue_id,
            blocked_id: i.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_blocking_duplicate_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let a = create_issue(&mut issue, cid, "A").await;
    let b = create_issue(&mut issue, cid, "B").await;

    issue
        .add_blocking(AddBlockingRequest {
            blocking_id: a.issue_id,
            blocked_id: b.issue_id,
        })
        .await
        .unwrap();

    let err = issue
        .add_blocking(AddBlockingRequest {
            blocking_id: a.issue_id,
            blocked_id: b.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn test_blocking_nonexistent_issue() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let a = create_issue(&mut issue, cid, "A").await;

    let err = issue
        .add_blocking(AddBlockingRequest {
            blocking_id: a.issue_id,
            blocked_id: 99999,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_remove_blocking_nonexistent_relationship() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let a = create_issue(&mut issue, cid, "A").await;
    let b = create_issue(&mut issue, cid, "B").await;

    let err = issue
        .remove_blocking(RemoveBlockingRequest {
            blocking_id: a.issue_id,
            blocked_id: b.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

// ── Parent Relationship Edge Cases ──────────────────────────────────────

#[tokio::test]
async fn test_add_parent_duplicate_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let child = create_issue(&mut issue, cid, "Child").await;
    let parent = create_issue(&mut issue, cid, "Parent").await;

    issue
        .add_parent(AddParentRequest {
            child_id: child.issue_id,
            parent_id: parent.issue_id,
        })
        .await
        .unwrap();

    let err = issue
        .add_parent(AddParentRequest {
            child_id: child.issue_id,
            parent_id: parent.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn test_remove_parent_nonexistent_relationship() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let a = create_issue(&mut issue, cid, "A").await;
    let b = create_issue(&mut issue, cid, "B").await;

    let err = issue
        .remove_parent(RemoveParentRequest {
            child_id: a.issue_id,
            parent_id: b.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_multiple_parents() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let child = create_issue(&mut issue, cid, "Child").await;
    let p1 = create_issue(&mut issue, cid, "Parent1").await;
    let p2 = create_issue(&mut issue, cid, "Parent2").await;

    issue
        .add_parent(AddParentRequest {
            child_id: child.issue_id,
            parent_id: p1.issue_id,
        })
        .await
        .unwrap();
    issue
        .add_parent(AddParentRequest {
            child_id: child.issue_id,
            parent_id: p2.issue_id,
        })
        .await
        .unwrap();

    let parents = issue
        .list_parents(ListRelatedIssuesRequest {
            issue_id: child.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(parents.issues.len(), 2);
}

// ── Duplicate Edge Cases ────────────────────────────────────────────────

#[tokio::test]
async fn test_mark_duplicate_self_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let err = issue
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: i.issue_id,
            canonical_id: i.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_mark_duplicate_nonexistent_canonical() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let err = issue
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: i.issue_id,
            canonical_id: 99999,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_unmark_duplicate_non_duplicate_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let err = issue
        .unmark_duplicate(UnmarkDuplicateRequest {
            issue_id: i.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn test_mark_duplicate_sets_resolve_time() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let canonical = create_issue(&mut issue, cid, "Canonical").await;
    let dup = create_issue(&mut issue, cid, "Dup").await;
    assert!(dup.resolve_time.is_none());

    let marked = issue
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: dup.issue_id,
            canonical_id: canonical.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(marked.resolve_time.is_some());
}

#[tokio::test]
async fn test_unmark_duplicate_decrements_count() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let canonical = create_issue(&mut issue, cid, "Canonical").await;
    let dup = create_issue(&mut issue, cid, "Dup").await;

    issue
        .mark_duplicate(MarkDuplicateRequest {
            issue_id: dup.issue_id,
            canonical_id: canonical.issue_id,
        })
        .await
        .unwrap();

    let can = issue
        .get_issue(GetIssueRequest {
            issue_id: canonical.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(can.duplicate_count, 1);

    issue
        .unmark_duplicate(UnmarkDuplicateRequest {
            issue_id: dup.issue_id,
        })
        .await
        .unwrap();

    let can = issue
        .get_issue(GetIssueRequest {
            issue_id: canonical.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(can.duplicate_count, 0);
}

// ── List Issues Empty ───────────────────────────────────────────────────

#[tokio::test]
async fn test_list_issues_empty_component() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    let result = issue
        .list_issues(ListIssuesRequest {
            component_id: cid,
            status_filter: String::new(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(result.issues.is_empty());
    assert!(result.next_page_token.is_empty());
}

// ── Full Lifecycle ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_status_full_lifecycle() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Lifecycle Bug").await;
    assert_eq!(i.status, Status::New as i32);

    // NEW -> ASSIGNED (set assignee)
    let assigned = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            assignee: Some("dev@example.com".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(assigned.status, Status::Assigned as i32);

    // ASSIGNED -> IN_PROGRESS
    let in_progress = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::InProgress as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(in_progress.status, Status::InProgress as i32);
    assert!(in_progress.resolve_time.is_none());

    // IN_PROGRESS -> FIXED (resolve_time should be set)
    let fixed = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::Fixed as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fixed.status, Status::Fixed as i32);
    assert!(fixed.resolve_time.is_some());

    // FIXED -> FIXED_VERIFIED (verify_time should be set)
    let verified = issue
        .update_issue(UpdateIssueRequest {
            issue_id: i.issue_id,
            status: Some(Status::FixedVerified as i32),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(verified.status, Status::FixedVerified as i32);
    assert!(verified.verify_time.is_some());
}

// ── Concurrent Updates ──────────────────────────────────────────────────

#[tokio::test]
async fn test_concurrent_updates() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Concurrent Bug").await;
    let id = i.issue_id;

    // Two rapid updates to different fields
    issue
        .update_issue(UpdateIssueRequest {
            issue_id: id,
            title: Some("Update A".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    issue
        .update_issue(UpdateIssueRequest {
            issue_id: id,
            priority: Some(Priority::P0 as i32),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["priority".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    // Verify the issue reflects both updates (last-write-wins per field)
    let final_issue = issue
        .get_issue(GetIssueRequest { issue_id: id })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(final_issue.title, "Update A");
    assert_eq!(final_issue.priority, Priority::P0 as i32);
}
