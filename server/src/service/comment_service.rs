use std::sync::Arc;

use identity::IdentityProvider;
use quiver_driver_core::{Connection, Pool, Transaction, Transactional, Value};
use quiver_query::{Filter, Order, Query};
use tonic::{Request, Response, Status};

use crate::db::row_mapping::{Comment, CommentRevision, Issue};
use crate::db::DbConn;
use crate::domain::permissions;
use crate::domain::types::DomainError;
use crate::proto::comment_service_server::CommentService;
use crate::proto::{
    Comment as ProtoComment, CommentRevision as ProtoCommentRevision, CreateCommentRequest,
    HideCommentRequest, ListCommentRevisionsRequest, ListCommentRevisionsResponse,
    ListCommentsRequest, ListCommentsResponse, UpdateCommentRequest,
};

pub struct CommentServiceImpl {
    pub db: DbConn,
    pub identity: Arc<dyn IdentityProvider>,
}

fn parse_timestamp(s: &str) -> Option<prost_types::Timestamp> {
    chrono::DateTime::parse_from_rfc3339(s)
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .ok()
        .map(|dt| prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
}

fn comment_to_proto(c: &Comment) -> ProtoComment {
    ProtoComment {
        comment_id: c.id as i64,
        issue_id: c.issue_id as i64,
        author: c.author.clone(),
        body: c.body.clone(),
        is_description: c.is_description,
        create_time: parse_timestamp(&c.created_at),
        modify_time: c.modified_at.as_deref().and_then(parse_timestamp),
        hidden: c.hidden,
        hidden_by: c.hidden_by.clone().unwrap_or_default(),
        hidden_time: c.hidden_at.as_deref().and_then(parse_timestamp),
        revision_count: 0, // filled by caller if needed
    }
}

fn revision_to_proto(r: &CommentRevision) -> ProtoCommentRevision {
    ProtoCommentRevision {
        revision_id: r.id as i64,
        comment_id: r.comment_id as i64,
        body: r.body.clone(),
        edited_by: r.edited_by.clone(),
        create_time: parse_timestamp(&r.created_at),
    }
}

async fn fetch_issue<C: Connection>(
    conn: &C,
    issue_id: i64,
) -> Result<crate::db::row_mapping::Issue, DomainError> {
    let stmt = Query::table("Issue")
        .find_first()
        .filter(Filter::eq("id", Value::Int(issue_id)))
        .build();
    let row = conn
        .query_optional(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let row = row.ok_or_else(|| DomainError::NotFound(format!("issue {issue_id} not found")))?;
    Ok(Issue::try_from(&row)?)
}

async fn fetch_comment<C: Connection>(conn: &C, comment_id: i64) -> Result<Comment, DomainError> {
    let stmt = Query::table("Comment")
        .find_first()
        .filter(Filter::eq("id", Value::Int(comment_id)))
        .build();
    let row = conn
        .query_optional(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let row =
        row.ok_or_else(|| DomainError::NotFound(format!("comment {comment_id} not found")))?;
    Ok(Comment::try_from(&row)?)
}

async fn count_revisions<C: Connection>(conn: &C, comment_id: i32) -> Result<i32, DomainError> {
    let stmt = Query::table("CommentRevision")
        .find_many()
        .filter(Filter::eq("commentId", Value::Int(comment_id as i64)))
        .build();
    let rows = conn
        .query(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(rows.len() as i32)
}

async fn log_event<C: Connection>(
    conn: &C,
    event_type: &str,
    actor: &str,
    entity_id: i32,
    payload: &serde_json::Value,
) -> Result<(), DomainError> {
    let stmt = Query::table("EventLog")
        .create()
        .set("eventTime", Value::Text(chrono::Utc::now().to_rfc3339()))
        .set("eventType", Value::Text(event_type.to_string()))
        .set("actor", Value::Text(actor.to_string()))
        .set("entityType", Value::Text("Comment".to_string()))
        .set("entityId", Value::Int(entity_id as i64))
        .set("payload", Value::Text(payload.to_string()))
        .build();
    conn.execute(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(())
}

#[tonic::async_trait]
impl CommentService for CommentServiceImpl {
    async fn create_comment(
        &self,
        request: Request<CreateCommentRequest>,
    ) -> Result<Response<ProtoComment>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if req.body.trim().is_empty() {
            return Err(DomainError::InvalidArgument("body must not be empty".to_string()).into());
        }

        let user_groups = match user_id.as_deref() {
            Some(uid) => self
                .identity
                .resolve_user_groups(uid)
                .await
                .unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = fetch_issue(&tx, req.issue_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::CommentOnIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        let stmt = Query::table("Comment")
            .create()
            .set("issueId", Value::Int(req.issue_id))
            .set("author", Value::Text(req.author))
            .set("body", Value::Text(req.body))
            .set("isDescription", Value::Bool(false))
            .set("hidden", Value::Bool(false))
            .set("createdAt", Value::Text(chrono::Utc::now().to_rfc3339()))
            .build();
        tx.execute(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let fetch = Query::raw("SELECT * FROM Comment WHERE id = last_insert_rowid()").build();
        let row = tx
            .query_one(&fetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let comment = Comment::try_from(&row).map_err(DomainError::from)?;

        let actor = user_id.as_deref().unwrap_or("system");
        log_event(
            &tx,
            "COMMENT_ADDED",
            actor,
            comment.issue_id,
            &serde_json::json!({"comment_id": comment.id}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(comment_to_proto(&comment)))
    }

    async fn list_comments(
        &self,
        request: Request<ListCommentsRequest>,
    ) -> Result<Response<ListCommentsResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self
                .identity
                .resolve_user_groups(uid)
                .await
                .unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = fetch_issue(&tx, req.issue_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        let page_size = if req.page_size > 0 {
            req.page_size.min(100)
        } else {
            50
        };

        let mut q = Query::table("Comment")
            .find_many()
            .filter(Filter::eq("issueId", Value::Int(req.issue_id)))
            .order_by("createdAt", Order::Asc)
            .limit(page_size as u64);

        if !req.page_token.is_empty() {
            let cursor_id = req
                .page_token
                .parse::<i64>()
                .map_err(|_| DomainError::InvalidArgument("invalid page_token".to_string()))?;
            q = q.filter(Filter::gt("id", Value::Int(cursor_id)));
        }

        let stmt = q.build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let comments: Vec<Comment> = rows
            .iter()
            .map(|r| Comment::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        // Count revisions for each comment
        let mut proto_comments: Vec<ProtoComment> = Vec::with_capacity(comments.len());
        for c in &comments {
            let rev_count = count_revisions(&tx, c.id).await?;
            let mut pc = comment_to_proto(c);
            pc.revision_count = rev_count;
            proto_comments.push(pc);
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let next_page_token = if comments.len() == page_size as usize {
            comments
                .last()
                .map(|c| c.id.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        Ok(Response::new(ListCommentsResponse {
            comments: proto_comments,
            next_page_token,
        }))
    }

    async fn update_comment(
        &self,
        request: Request<UpdateCommentRequest>,
    ) -> Result<Response<ProtoComment>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if req.body.trim().is_empty() {
            return Err(DomainError::InvalidArgument("body must not be empty".to_string()).into());
        }

        let user_groups = match user_id.as_deref() {
            Some(uid) => self
                .identity
                .resolve_user_groups(uid)
                .await
                .unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let existing_comment = fetch_comment(&tx, req.comment_id).await?;
        let issue = fetch_issue(&tx, existing_comment.issue_id as i64).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::CommentOnIssues,
            Some(existing_comment.issue_id as i64),
            &user_groups,
        )
        .await?;

        // Save the current body as a revision before updating
        let actor = user_id.as_deref().unwrap_or("system");
        let rev_stmt = Query::table("CommentRevision")
            .create()
            .set("commentId", Value::Int(existing_comment.id as i64))
            .set("body", Value::Text(existing_comment.body.clone()))
            .set("editedBy", Value::Text(actor.to_string()))
            .set("createdAt", Value::Text(chrono::Utc::now().to_rfc3339()))
            .build();
        tx.execute(&rev_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();
        let update_stmt = Query::table("Comment")
            .update()
            .filter(Filter::eq("id", Value::Int(req.comment_id)))
            .set("body", Value::Text(req.body))
            .set("modifiedAt", Value::Text(now))
            .build();
        tx.execute(&update_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch updated row
        let refetch = Query::table("Comment")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.comment_id)))
            .build();
        let row = tx
            .query_one(&refetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let comment = Comment::try_from(&row).map_err(DomainError::from)?;

        let rev_count = count_revisions(&tx, comment.id).await?;

        log_event(
            &tx,
            "COMMENT_EDITED",
            actor,
            comment.issue_id,
            &serde_json::json!({"comment_id": comment.id, "revision_count": rev_count}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut proto = comment_to_proto(&comment);
        proto.revision_count = rev_count;
        Ok(Response::new(proto))
    }

    async fn hide_comment(
        &self,
        request: Request<HideCommentRequest>,
    ) -> Result<Response<ProtoComment>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self
                .identity
                .resolve_user_groups(uid)
                .await
                .unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let existing_comment = fetch_comment(&tx, req.comment_id).await?;
        let issue = fetch_issue(&tx, existing_comment.issue_id as i64).await?;

        // Hiding requires EDIT_ISSUES permission (moderator action)
        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(existing_comment.issue_id as i64),
            &user_groups,
        )
        .await?;

        let actor = user_id.as_deref().unwrap_or("system");
        let now = chrono::Utc::now().to_rfc3339();

        // When hiding: save the original body as a revision, then redact the body in DB
        if req.hidden && !existing_comment.hidden {
            let rev_stmt = Query::table("CommentRevision")
                .create()
                .set("commentId", Value::Int(existing_comment.id as i64))
                .set("body", Value::Text(existing_comment.body.clone()))
                .set("editedBy", Value::Text(actor.to_string()))
                .set("createdAt", Value::Text(chrono::Utc::now().to_rfc3339()))
                .build();
            tx.execute(&rev_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        if !req.hidden {
            // Unhiding is not possible since the body was redacted.
            return Err(DomainError::InvalidArgument(
                "cannot unhide a comment: the content has been permanently redacted".to_string(),
            )
            .into());
        }

        let update_stmt = Query::table("Comment")
            .update()
            .filter(Filter::eq("id", Value::Int(req.comment_id)))
            .set("hidden", Value::Bool(true))
            .set("hiddenBy", Value::Text(actor.to_string()))
            .set("hiddenAt", Value::Text(now))
            .set(
                "body",
                Value::Text("[This comment has been removed by a moderator]".to_string()),
            )
            .build();
        tx.execute(&update_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch updated row
        let refetch = Query::table("Comment")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.comment_id)))
            .build();
        let row = tx
            .query_one(&refetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let comment = Comment::try_from(&row).map_err(DomainError::from)?;

        log_event(
            &tx,
            "COMMENT_HIDDEN",
            actor,
            comment.issue_id,
            &serde_json::json!({"comment_id": comment.id}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut proto = comment_to_proto(&comment);
        proto.revision_count = 0; // not critical for hide response
        Ok(Response::new(proto))
    }

    async fn list_comment_revisions(
        &self,
        request: Request<ListCommentRevisionsRequest>,
    ) -> Result<Response<ListCommentRevisionsResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self
                .identity
                .resolve_user_groups(uid)
                .await
                .unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let existing_comment = fetch_comment(&tx, req.comment_id).await?;
        let issue = fetch_issue(&tx, existing_comment.issue_id as i64).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            Some(existing_comment.issue_id as i64),
            &user_groups,
        )
        .await?;

        let page_size = if req.page_size > 0 {
            req.page_size.min(100)
        } else {
            50
        };

        let mut q = Query::table("CommentRevision")
            .find_many()
            .filter(Filter::eq("commentId", Value::Int(req.comment_id)))
            .order_by("createdAt", Order::Desc)
            .limit(page_size as u64);

        if !req.page_token.is_empty() {
            let cursor_id = req
                .page_token
                .parse::<i64>()
                .map_err(|_| DomainError::InvalidArgument("invalid page_token".to_string()))?;
            q = q.filter(Filter::gt("id", Value::Int(cursor_id)));
        }

        let stmt = q.build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let revisions: Vec<CommentRevision> = rows
            .iter()
            .map(|r| CommentRevision::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        let next_page_token = if revisions.len() == page_size as usize {
            revisions
                .last()
                .map(|r| r.id.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let proto_revisions: Vec<ProtoCommentRevision> =
            revisions.iter().map(revision_to_proto).collect();

        Ok(Response::new(ListCommentRevisionsResponse {
            revisions: proto_revisions,
            next_page_token,
        }))
    }
}
