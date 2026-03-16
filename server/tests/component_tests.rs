#[allow(dead_code, unused_imports)]
mod common;
use common::*;

// ── 8.1 Component Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_get_component() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let created = client
        .create_component(CreateComponentRequest {
            name: "TestComp".to_string(),
            description: "A test component".to_string(),
            parent_id: None,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(created.component_id > 0);
    assert_eq!(created.name, "TestComp");
    assert_eq!(created.description, "A test component");

    grant_admin(&mut acl, created.component_id).await;

    let got = client
        .get_component(GetComponentRequest {
            component_id: created.component_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(got.component_id, created.component_id);
    assert_eq!(got.name, "TestComp");
}

#[tokio::test]
async fn test_create_child_component() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let parent_id = create_component(&mut client, &mut acl, "Parent", None).await;
    let child_id = create_component(&mut client, &mut acl, "Child", Some(parent_id)).await;

    let child = client
        .get_component(GetComponentRequest {
            component_id: child_id,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(child.parent_id, Some(parent_id));
}

#[tokio::test]
async fn test_list_children() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let parent_id = create_component(&mut client, &mut acl, "Parent", None).await;
    create_component(&mut client, &mut acl, "Child1", Some(parent_id)).await;
    create_component(&mut client, &mut acl, "Child2", Some(parent_id)).await;
    create_component(&mut client, &mut acl, "Child3", Some(parent_id)).await;

    let resp = client
        .list_components(ListComponentsRequest {
            parent_id: Some(parent_id),
            page_size: 10,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.components.len(), 3);
}

#[tokio::test]
async fn test_update_component() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let id = create_component(&mut client, &mut acl, "Original", None).await;

    let updated = client
        .update_component(UpdateComponentRequest {
            component_id: id,
            name: Some("Updated".to_string()),
            description: Some("New description".to_string()),
            parent_id: None,
            update_mask: Some(prost_types::FieldMask {
                paths: vec!["name".to_string(), "description".to_string()],
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.description, "New description");
}

#[tokio::test]
async fn test_delete_leaf_component() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let parent_id = create_component(&mut client, &mut acl, "Parent", None).await;
    let child_id = create_component(&mut client, &mut acl, "Child", Some(parent_id)).await;

    client
        .delete_component(DeleteComponentRequest {
            component_id: child_id,
        })
        .await
        .unwrap();

    client
        .delete_component(DeleteComponentRequest {
            component_id: parent_id,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_delete_component_with_children_fails() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let parent_id = create_component(&mut client, &mut acl, "Parent", None).await;
    create_component(&mut client, &mut acl, "Child", Some(parent_id)).await;

    let err = client
        .delete_component(DeleteComponentRequest {
            component_id: parent_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn test_delete_component_with_issues_fails() {
    let f = TestFixture::new().await;
    let mut comp_client = f.component_client();
    let mut issue_client = f.issue_client();
    let mut acl = f.acl_client();

    let comp_id = create_component(&mut comp_client, &mut acl, "Comp", None).await;
    create_issue(&mut issue_client, comp_id, "An issue").await;

    let err = comp_client
        .delete_component(DeleteComponentRequest {
            component_id: comp_id,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn test_get_nonexistent_component() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();

    let err = client
        .get_component(GetComponentRequest {
            component_id: 999999,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_create_component_invalid_parent() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();

    let err = client
        .create_component(CreateComponentRequest {
            name: "Orphan".to_string(),
            description: String::new(),
            parent_id: Some(999999),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

// ── Component Hierarchy ─────────────────────────────────────────────────

#[tokio::test]
async fn test_nested_component_hierarchy_child_count() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    let root = create_component(&mut client, &mut acl, "Root", None).await;
    let _c1 = create_component(&mut client, &mut acl, "Child1", Some(root)).await;
    let _c2 = create_component(&mut client, &mut acl, "Child2", Some(root)).await;
    let _c3 = create_component(&mut client, &mut acl, "Child3", Some(root)).await;

    let got = client
        .get_component(GetComponentRequest { component_id: root })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(got.child_count, 3);
}

// ── Pagination ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_components_pagination() {
    let f = TestFixture::new().await;
    let mut client = f.component_client();
    let mut acl = f.acl_client();

    // Create 5 root components
    for i in 0..5 {
        create_component(&mut client, &mut acl, &format!("Comp{i:02}"), None).await;
    }

    // Page size 2: should get 3 pages (2, 2, 1)
    let page1 = client
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 2,
            page_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page1.components.len(), 2);
    assert!(!page1.next_page_token.is_empty());

    let page2 = client
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 2,
            page_token: page1.next_page_token,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page2.components.len(), 2);
    assert!(!page2.next_page_token.is_empty());

    let page3 = client
        .list_components(ListComponentsRequest {
            parent_id: None,
            page_size: 2,
            page_token: page2.next_page_token,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page3.components.len(), 1);
    assert!(page3.next_page_token.is_empty());
}
