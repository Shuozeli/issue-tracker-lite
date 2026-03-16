use test_utils::*;

// ── 8.6 Search Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_search_by_status() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut search_client = f.search_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Will be fixed").await;

    // Assign then mark fixed
    issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            assignee: Some("dev@example.com".to_string()),
            status: Some(Status::Fixed as i32),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["assignee".to_string(), "status".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    create_issue(&mut issue_client, comp_id, "Still open").await;

    let resp = search_client
        .search_issues(SearchIssuesRequest {
            query: "status:open".to_string(),
            page_size: 50,
            page_token: String::new(),
            order_by: String::new(),
            order_direction: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].title, "Still open");
}

#[tokio::test]
async fn test_search_by_priority() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut search_client = f.search_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "P0 issue".to_string(),
            description: String::new(),
            priority: Priority::P0 as i32,
            r#type: IssueType::Bug as i32,
            severity: None,
            assignee: None,
            reporter: Some("test@example.com".to_string()),
            verifier: None,
            found_in: None,
            targeted_to: None,
        })
        .await
        .unwrap();

    issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "P2 issue".to_string(),
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
        .unwrap();

    let resp = search_client
        .search_issues(SearchIssuesRequest {
            query: "priority:P0".to_string(),
            page_size: 50,
            page_token: String::new(),
            order_by: String::new(),
            order_direction: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].title, "P0 issue");
}

#[tokio::test]
async fn test_search_keyword() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut search_client = f.search_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Memory leak in parser".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "UI button broken".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    let resp = search_client
        .search_issues(SearchIssuesRequest {
            query: "memory".to_string(),
            page_size: 50,
            page_token: String::new(),
            order_by: String::new(),
            order_direction: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].title, "Memory leak in parser");
}

#[tokio::test]
async fn test_search_combined() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut search_client = f.search_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Memory leak critical".to_string(),
            description: String::new(),
            priority: Priority::P0 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "Memory leak minor".to_string(),
            description: String::new(),
            priority: Priority::P3 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    let resp = search_client
        .search_issues(SearchIssuesRequest {
            query: "memory priority:P0".to_string(),
            page_size: 50,
            page_token: String::new(),
            order_by: String::new(),
            order_direction: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].title, "Memory leak critical");
}

#[tokio::test]
async fn test_search_pagination() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut search_client = f.search_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    for i in 0..25 {
        issue_client
            .create_issue(CreateIssueRequest {
                component_id: comp_id,
                title: format!("Issue {}", i),
                description: String::new(),
                priority: Priority::P2 as i32,
                r#type: IssueType::Bug as i32,
                reporter: Some("test@example.com".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
    }

    let mut all_ids = Vec::new();
    let mut page_token = String::new();

    loop {
        let resp = search_client
            .search_issues(SearchIssuesRequest {
                query: "status:open".to_string(),
                page_size: 10,
                page_token: page_token.clone(),
                order_by: String::new(),
                order_direction: String::new(),
            })
            .await
            .unwrap()
            .into_inner();

        for issue in &resp.issues {
            all_ids.push(issue.issue_id);
        }

        if resp.next_page_token.is_empty() {
            break;
        }
        page_token = resp.next_page_token;
    }

    assert_eq!(all_ids.len(), 25);
    let mut sorted = all_ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 25);
}

// ── Search Edge Cases ───────────────────────────────────────────────────

#[tokio::test]
async fn test_search_empty_result() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    create_issue(&mut issue, cid, "Bug").await;

    let result = search
        .search_issues(SearchIssuesRequest {
            query: "xyznonexistent123".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(result.issues.is_empty());
}

#[tokio::test]
async fn test_search_by_type() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "Bug issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap();
    issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "Feature request".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::FeatureRequest as i32,
            ..Default::default()
        })
        .await
        .unwrap();

    let result = search
        .search_issues(SearchIssuesRequest {
            query: "type:FEATURE_REQUEST".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].r#type, IssueType::FeatureRequest as i32);
}

#[tokio::test]
async fn test_search_by_assignee() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "Assigned issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            assignee: Some("user@example.com".to_string()),
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "Unassigned issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    let resp = search
        .search_issues(SearchIssuesRequest {
            query: "assignee:user@example.com".to_string(),
            page_size: 50,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].title, "Assigned issue");
}

#[tokio::test]
async fn test_search_by_component_recursive() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();

    let parent = create_component(&mut comp, &mut acl, "Parent", None).await;
    let child = create_component(&mut comp, &mut acl, "Child", Some(parent)).await;
    let grandchild = create_component(&mut comp, &mut acl, "Grandchild", Some(child)).await;

    // Create issues in parent, child, and grandchild
    issue
        .create_issue(CreateIssueRequest {
            component_id: parent,
            title: "Parent issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    issue
        .create_issue(CreateIssueRequest {
            component_id: child,
            title: "Child issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    issue
        .create_issue(CreateIssueRequest {
            component_id: grandchild,
            title: "Grandchild issue".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    // componentid:<id>+ means recursive (include descendants)
    let resp = search
        .search_issues(SearchIssuesRequest {
            query: format!("componentid:{}+", parent),
            page_size: 50,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    // Should find all 3 issues (parent + child + grandchild)
    assert_eq!(resp.issues.len(), 3);
}

#[tokio::test]
async fn test_search_not_operator() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    // Create open issue
    create_issue(&mut issue, cid, "Open issue").await;

    // Create closed issue
    let closed = create_issue(&mut issue, cid, "Closed issue").await;
    issue
        .update_issue(UpdateIssueRequest {
            issue_id: closed.issue_id,
            status: Some(Status::Fixed as i32),
            ..Default::default()
        })
        .await
        .unwrap();

    let resp = search
        .search_issues(SearchIssuesRequest {
            query: "-status:closed".to_string(),
            page_size: 50,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].title, "Open issue");
}

#[tokio::test]
async fn test_search_special_values() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    // Create assigned issue
    issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "Assigned".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            assignee: Some("dev@example.com".to_string()),
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    // Create unassigned issue
    create_issue(&mut issue, cid, "Unassigned").await;

    // assignee:none returns unassigned
    let none_resp = search
        .search_issues(SearchIssuesRequest {
            query: "assignee:none".to_string(),
            page_size: 50,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(none_resp.issues.len(), 1);
    assert_eq!(none_resp.issues[0].title, "Unassigned");

    // assignee:any returns assigned
    let any_resp = search
        .search_issues(SearchIssuesRequest {
            query: "assignee:any".to_string(),
            page_size: 50,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(any_resp.issues.len(), 1);
    assert_eq!(any_resp.issues[0].title, "Assigned");
}

#[tokio::test]
async fn test_search_order_by() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    let i1 = create_issue(&mut issue, cid, "First").await;
    let i2 = create_issue(&mut issue, cid, "Second").await;
    let i3 = create_issue(&mut issue, cid, "Third").await;

    // Order by created desc: should get Third, Second, First
    let resp = search
        .search_issues(SearchIssuesRequest {
            query: String::new(),
            page_size: 50,
            order_by: "created".to_string(),
            order_direction: "desc".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.issues.len(), 3);
    assert_eq!(resp.issues[0].issue_id, i3.issue_id);
    assert_eq!(resp.issues[1].issue_id, i2.issue_id);
    assert_eq!(resp.issues[2].issue_id, i1.issue_id);
}

#[tokio::test]
async fn test_search_by_hotlist() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut hotlist = f.hotlist_client();
    let mut search = f.search_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    let i1 = create_issue(&mut issue, cid, "In hotlist").await;
    let _i2 = create_issue(&mut issue, cid, "Not in hotlist").await;

    let hl = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "Search HL".to_string(),
            description: String::new(),
            owner: "owner@test.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    grant_hotlist_admin(&mut acl, hl.hotlist_id).await;

    hotlist
        .add_issue(AddIssueToHotlistRequest {
            hotlist_id: hl.hotlist_id,
            issue_id: i1.issue_id,
            added_by: "user@test.com".to_string(),
        })
        .await
        .unwrap();

    let resp = search
        .search_issues(SearchIssuesRequest {
            query: format!("hotlistid:{}", hl.hotlist_id),
            page_size: 50,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.issues.len(), 1);
    assert_eq!(resp.issues[0].issue_id, i1.issue_id);
}
