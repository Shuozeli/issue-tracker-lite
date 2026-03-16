use test_utils::*;

// ── 8.5 Hotlist Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_hotlist_crud() {
    let f = TestFixture::new().await;
    let mut client = f.hotlist_client();
    let mut acl = f.acl_client();

    let created = client
        .create_hotlist(CreateHotlistRequest {
            name: "My Hotlist".to_string(),
            description: "A test hotlist".to_string(),
            owner: "owner@example.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.name, "My Hotlist");

    grant_hotlist_admin(&mut acl, created.hotlist_id).await;

    let got = client
        .get_hotlist(GetHotlistRequest {
            hotlist_id: created.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(got.name, "My Hotlist");

    let updated = client
        .update_hotlist(UpdateHotlistRequest {
            hotlist_id: created.hotlist_id,
            name: Some("Renamed".to_string()),
            description: Some("Updated desc".to_string()),
            archived: None,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.description, "Updated desc");
}

#[tokio::test]
async fn test_hotlist_add_remove_issues() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut hotlist_client = f.hotlist_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue1 = create_issue(&mut issue_client, comp_id, "Issue 1").await;
    let issue2 = create_issue(&mut issue_client, comp_id, "Issue 2").await;
    let issue3 = create_issue(&mut issue_client, comp_id, "Issue 3").await;

    let hotlist = hotlist_client
        .create_hotlist(CreateHotlistRequest {
            name: "Hotlist".to_string(),
            description: String::new(),
            owner: "owner@example.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, hotlist.hotlist_id).await;

    for issue_id in [issue1.issue_id, issue2.issue_id, issue3.issue_id] {
        hotlist_client
            .add_issue(AddIssueToHotlistRequest {
                hotlist_id: hotlist.hotlist_id,
                issue_id,
                added_by: "user@example.com".to_string(),
            })
            .await
            .unwrap();
    }

    let issues = hotlist_client
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: hotlist.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(issues.issues.len(), 3);

    hotlist_client
        .remove_issue(RemoveIssueFromHotlistRequest {
            hotlist_id: hotlist.hotlist_id,
            issue_id: issue2.issue_id,
        })
        .await
        .unwrap();

    let issues = hotlist_client
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: hotlist.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(issues.issues.len(), 2);
}

#[tokio::test]
async fn test_hotlist_add_duplicate_issue_fails() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut hotlist_client = f.hotlist_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Issue").await;

    let hotlist = hotlist_client
        .create_hotlist(CreateHotlistRequest {
            name: "Hotlist".to_string(),
            description: String::new(),
            owner: "owner@example.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, hotlist.hotlist_id).await;

    hotlist_client
        .add_issue(AddIssueToHotlistRequest {
            hotlist_id: hotlist.hotlist_id,
            issue_id: issue.issue_id,
            added_by: "user@example.com".to_string(),
        })
        .await
        .unwrap();

    let err = hotlist_client
        .add_issue(AddIssueToHotlistRequest {
            hotlist_id: hotlist.hotlist_id,
            issue_id: issue.issue_id,
            added_by: "user@example.com".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

// ── Hotlist Reorder & Filtering ─────────────────────────────────────────

#[tokio::test]
async fn test_hotlist_reorder() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i1 = create_issue(&mut issue, cid, "I1").await;
    let i2 = create_issue(&mut issue, cid, "I2").await;
    let i3 = create_issue(&mut issue, cid, "I3").await;

    let h = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "HL".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, h.hotlist_id).await;

    for i in [&i1, &i2, &i3] {
        hotlist
            .add_issue(AddIssueToHotlistRequest {
                hotlist_id: h.hotlist_id,
                issue_id: i.issue_id,
                added_by: "user@test.com".to_string(),
            })
            .await
            .unwrap();
    }

    // Verify initial order: i1(0), i2(1), i3(2)
    let before = hotlist
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: h.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(before.issues[0].issue_id, i1.issue_id);
    assert_eq!(before.issues[1].issue_id, i2.issue_id);
    assert_eq!(before.issues[2].issue_id, i3.issue_id);

    // Reorder: i3, i1, i2
    hotlist
        .reorder_issues(ReorderHotlistIssuesRequest {
            hotlist_id: h.hotlist_id,
            issue_ids: vec![i3.issue_id, i1.issue_id, i2.issue_id],
        })
        .await
        .unwrap();

    let after = hotlist
        .list_issues(ListHotlistIssuesRequest {
            hotlist_id: h.hotlist_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(after.issues[0].issue_id, i3.issue_id);
    assert_eq!(after.issues[1].issue_id, i1.issue_id);
    assert_eq!(after.issues[2].issue_id, i2.issue_id);
}

#[tokio::test]
async fn test_hotlist_reorder_nonexistent_hotlist() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut _acl = f.acl_client();

    let err = hotlist
        .reorder_issues(ReorderHotlistIssuesRequest {
            hotlist_id: 99999,
            issue_ids: vec![1, 2, 3],
        })
        .await
        .unwrap_err();
    // Permission check happens before existence check
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_hotlist_archive_and_filter() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();

    let active = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Active".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, active.hotlist_id).await;

    let archived = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Archived".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, archived.hotlist_id).await;

    hotlist
        .update_hotlist(UpdateHotlistRequest {
            hotlist_id: archived.hotlist_id,
            archived: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();

    // Default filter returns only active
    let active_list = hotlist
        .list_hotlists(ListHotlistsRequest {
            filter: String::new(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(active_list.hotlists.len(), 1);
    assert_eq!(active_list.hotlists[0].hotlist_id, active.hotlist_id);

    // "archived" filter returns only archived
    let archived_list = hotlist
        .list_hotlists(ListHotlistsRequest {
            filter: "archived".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(archived_list.hotlists.len(), 1);
    assert_eq!(archived_list.hotlists[0].hotlist_id, archived.hotlist_id);

    // "all" filter returns both
    let all_list = hotlist
        .list_hotlists(ListHotlistsRequest {
            filter: "all".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(all_list.hotlists.len(), 2);
}

#[tokio::test]
async fn test_hotlist_add_to_nonexistent_hotlist() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let err = hotlist
        .add_issue(AddIssueToHotlistRequest {
            hotlist_id: 99999,
            issue_id: i.issue_id,
            added_by: "user@test.com".to_string(),
        })
        .await
        .unwrap_err();
    // Permission check happens before existence check
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_hotlist_add_nonexistent_issue() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();

    let h = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "HL".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, h.hotlist_id).await;

    let err = hotlist
        .add_issue(AddIssueToHotlistRequest {
            hotlist_id: h.hotlist_id,
            issue_id: 99999,
            added_by: "user@test.com".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_hotlist_remove_not_member() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut hotlist = f.hotlist_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let h = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "HL".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, h.hotlist_id).await;

    let err = hotlist
        .remove_issue(RemoveIssueFromHotlistRequest {
            hotlist_id: h.hotlist_id,
            issue_id: i.issue_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}
