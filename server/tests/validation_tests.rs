mod common;
use common::*;

// ── 8.8 Cross-Cutting Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_unicode_fields() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    let issue = issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: "CJK: \u{4F60}\u{597D}\u{4E16}\u{754C}".to_string(),
            description: "Description with special chars: <>&\"'".to_string(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    let got = issue_client
        .get_issue(GetIssueRequest {
            issue_id: issue.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(got.title, "CJK: \u{4F60}\u{597D}\u{4E16}\u{754C}");
}

#[tokio::test]
async fn test_large_payload() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;

    let long_title = "A".repeat(10_000);
    let long_desc = "B".repeat(100_000);

    let issue = issue_client
        .create_issue(CreateIssueRequest {
            component_id: comp_id,
            title: long_title.clone(),
            description: long_desc.clone(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    let got = issue_client
        .get_issue(GetIssueRequest {
            issue_id: issue.issue_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(got.title.len(), 10_000);
    assert_eq!(got.description.len(), 100_000);
}

#[tokio::test]
async fn test_health_ping() {
    let f = TestFixture::new().await;
    let mut client = f.health_client();

    let resp = client.ping(PingRequest {}).await.unwrap().into_inner();
    assert_eq!(resp.message, "pong");
}

// ── Validation Edge Cases ───────────────────────────────────────────────

#[tokio::test]
async fn test_create_issue_empty_title_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    let err = issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "   ".to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_create_component_empty_name_rejected() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();

    let err = client
        .create_component(CreateComponentRequest {
            name: "  ".to_string(),
            description: String::new(),
            parent_id: None,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_create_comment_empty_body_rejected() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut comment = f.comment_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let err = comment
        .create_comment(CreateCommentRequest {
            issue_id: i.issue_id,
            author: "user@test.com".to_string(),
            body: "  ".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_create_comment_on_nonexistent_issue() {
    let f = TestFixture::new().await;
    let mut comment = f.comment_client();

    let err = comment
        .create_comment(CreateCommentRequest {
            issue_id: 99999,
            author: "user@test.com".to_string(),
            body: "Hello".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_create_hotlist_empty_name_rejected() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();

    let err = hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "  ".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_update_component_empty_name_rejected() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut client, &mut acl, "Valid", None).await;

    let err = client
        .update_component(UpdateComponentRequest {
            component_id: cid,
            name: Some("  ".to_string()),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_update_component_nonexistent() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();

    let err = client
        .update_component(UpdateComponentRequest {
            component_id: 99999,
            name: Some("NewName".to_string()),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_update_component_self_parent_rejected() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut client, &mut acl, "Comp", None).await;

    let err = client
        .update_component(UpdateComponentRequest {
            component_id: cid,
            parent_id: Some(cid),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}
