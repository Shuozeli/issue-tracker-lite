#[allow(dead_code, unused_imports)]
mod common;
use common::*;

// ── 8.7 Event Log Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_event_log_issue_created() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut event_client = f.event_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test").await;

    let resp = event_client
        .list_events(ListEventsRequest {
            entity_type: "Issue".to_string(),
            entity_id: issue.issue_id,
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
    assert!(resp.events.iter().any(|e| e.event_type == "ISSUE_CREATED"));
}

#[tokio::test]
async fn test_event_log_issue_updated() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut event_client = f.event_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test").await;

    issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            title: Some("Updated title".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    let resp = event_client
        .list_events(ListEventsRequest {
            entity_type: "Issue".to_string(),
            entity_id: issue.issue_id,
            event_type: "ISSUE_UPDATED".to_string(),
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
}

#[tokio::test]
async fn test_event_log_filter_by_type() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut comment_client = f.comment_client();
    let mut event_client = f.event_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    let issue = create_issue(&mut issue_client, comp_id, "Test").await;

    issue_client
        .update_issue(UpdateIssueRequest {
            issue_id: issue.issue_id,
            title: Some("Updated".to_string()),
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["title".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    comment_client
        .create_comment(CreateCommentRequest {
            issue_id: issue.issue_id,
            body: "A comment".to_string(),
            author: "user@example.com".to_string(),
        })
        .await
        .unwrap();

    let resp = event_client
        .list_events(ListEventsRequest {
            entity_type: String::new(),
            entity_id: 0,
            event_type: "ISSUE_CREATED".to_string(),
            actor: String::new(),
            since: None,
            until: None,
            page_size: 50,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.events.len(), 1);
    assert_eq!(resp.events[0].event_type, "ISSUE_CREATED");
}

// ── Event Log Coverage ──────────────────────────────────────────────────

#[tokio::test]
async fn test_event_log_comment_created() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut comment = f.comment_client();
    let mut events = f.event_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i = create_issue(&mut issue, cid, "Bug").await;

    comment
        .create_comment(CreateCommentRequest {
            issue_id: i.issue_id,
            author: "user@test.com".to_string(),
            body: "Test comment".to_string(),
        })
        .await
        .unwrap();

    let log = events
        .list_events(ListEventsRequest {
            event_type: "COMMENT_ADDED".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!log.events.is_empty());
    assert_eq!(log.events[0].entity_type, "Comment");
}

#[tokio::test]
async fn test_event_log_hotlist_created() {
    let f = TestFixture::new().await;
    let mut hotlist = f.hotlist_client();
    let mut events = f.event_client();

    hotlist
        .create_hotlist(CreateHotlistRequest {
            name: "HL".to_string(),
            description: String::new(),
            owner: "user@test.com".to_string(),
        })
        .await
        .unwrap();

    let log = events
        .list_events(ListEventsRequest {
            event_type: "HOTLIST_CREATED".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!log.events.is_empty());
    assert_eq!(log.events[0].entity_type, "Hotlist");
}

#[tokio::test]
async fn test_event_log_filter_by_entity_id() {
    let f = TestFixture::new().await;
    let mut comp = f.component_client();
    let mut issue = f.issue_client();
    let mut events = f.event_client();
    let mut acl = f.acl_client();
    let cid = create_component(&mut comp, &mut acl, "C", None).await;
    let i1 = create_issue(&mut issue, cid, "Bug1").await;
    let _i2 = create_issue(&mut issue, cid, "Bug2").await;

    let log = events
        .list_events(ListEventsRequest {
            entity_type: "Issue".to_string(),
            entity_id: i1.issue_id,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    for evt in &log.events {
        assert_eq!(evt.entity_id, i1.issue_id);
    }
}
