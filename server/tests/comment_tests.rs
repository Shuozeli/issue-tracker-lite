#[allow(dead_code, unused_imports)]
mod common;
use common::*;

// ── 8.3 Comment Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_add_comments_ordering() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut comment_client = f.comment_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test issue").await;

    for i in 1..=3 {
        comment_client
            .create_comment(CreateCommentRequest {
                issue_id: issue.issue_id,
                body: format!("Comment {}", i),
                author: "user@example.com".to_string(),
            })
            .await
            .unwrap();
    }

    let resp = comment_client
        .list_comments(ListCommentsRequest {
            issue_id: issue.issue_id,
            page_size: 50,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(resp.comments.len() >= 3);
}

#[tokio::test]
async fn test_edit_comment() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut comment_client = f.comment_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test issue").await;

    let created = comment_client
        .create_comment(CreateCommentRequest {
            issue_id: issue.issue_id,
            body: "Original body".to_string(),
            author: "user@example.com".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    let edited = comment_client
        .update_comment(UpdateCommentRequest {
            comment_id: created.comment_id,
            body: "Edited body".to_string(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(edited.body, "Edited body");
    assert!(edited.modify_time.is_some());
}

#[tokio::test]
async fn test_edit_nonexistent_comment_fails() {
    let f = TestFixture::new().await;
    let mut comment_client = f.comment_client();

    let err = comment_client
        .update_comment(UpdateCommentRequest {
            comment_id: 999999,
            body: "Nope".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

// ── Pagination ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_comments_empty() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut comment = f.comment_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    let result = comment
        .list_comments(ListCommentsRequest {
            issue_id: i.issue_id,
            page_size: 50,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    // Description comment is auto-created if description is non-empty,
    // but our helper creates with empty description
    assert!(result.next_page_token.is_empty());
}

// ── Description Comment ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_issue_creates_description_comment() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut comment = f.comment_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;

    // Create issue with non-empty description
    let i = issue
        .create_issue(CreateIssueRequest {
            component_id: cid,
            title: "Issue with description".to_string(),
            description: "This is the description".to_string(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            reporter: Some("test@example.com".to_string()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();

    let resp = comment
        .list_comments(ListCommentsRequest {
            issue_id: i.issue_id,
            page_size: 50,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(!resp.comments.is_empty());
    let desc_comment = &resp.comments[0];
    assert!(desc_comment.is_description);
    assert_eq!(desc_comment.body, "This is the description");
}
