use quiver_driver_core::{Connection, Pool, Transaction, Transactional, Value};
use quiver_query::{Filter, Order, Query};
use serde_json::json;
use tonic::{Request, Response, Status};

use crate::db::DbConn;
use crate::db::row_mapping::{Issue, IssueBlocking, IssueParent};
use crate::domain::permissions;
use crate::domain::status_machine;
use crate::domain::types::DomainError;
use std::collections::HashSet;
use std::sync::Arc;
use identity::IdentityProvider;

use crate::proto::issue_service_server::IssueService;
use crate::proto::{
    AddBlockingRequest, AddParentRequest, CreateIssueRequest, GetIssueRequest,
    Issue as ProtoIssue, ListIssuesRequest, ListIssuesResponse, ListRelatedIssuesRequest,
    ListRelatedIssuesResponse, MarkDuplicateRequest, RelationshipResponse, RemoveBlockingRequest,
    RemoveParentRequest, UnmarkDuplicateRequest, UpdateIssueRequest,
};

pub struct IssueServiceImpl {
    pub db: DbConn,
    pub identity: Arc<dyn IdentityProvider>,
}

// Proto enum <-> DB string conversion helpers.
//
// The proto enum variant names (e.g., "NEW", "P0", "BUG") match the DB
// strings exactly, so we use prost's as_str_name() / from_str_name()
// instead of maintaining manual match arms.

use crate::proto::{
    IssueType as ProtoIssueType, Priority as ProtoPriority, Severity as ProtoSeverity,
    Status as ProtoStatus,
};

fn proto_status_to_str(status: i32) -> Result<String, DomainError> {
    let s = ProtoStatus::try_from(status)
        .map_err(|_| DomainError::InvalidArgument(format!("unknown status value: {status}")))?;
    if s == ProtoStatus::Unspecified {
        return Err(DomainError::InvalidArgument("status must be specified".into()));
    }
    Ok(s.as_str_name().to_string())
}

fn str_to_proto_status(s: &str) -> i32 {
    ProtoStatus::from_str_name(s).map(|v| v as i32).unwrap_or(0)
}

fn proto_priority_to_str(priority: i32) -> Result<String, DomainError> {
    let p = ProtoPriority::try_from(priority)
        .map_err(|_| DomainError::InvalidArgument(format!("unknown priority value: {priority}")))?;
    if p == ProtoPriority::Unspecified {
        return Err(DomainError::InvalidArgument("priority must be specified".into()));
    }
    Ok(p.as_str_name().to_string())
}

fn str_to_proto_priority(s: &str) -> i32 {
    ProtoPriority::from_str_name(s).map(|v| v as i32).unwrap_or(0)
}

fn proto_severity_to_str(severity: i32) -> Result<String, DomainError> {
    let s = ProtoSeverity::try_from(severity)
        .map_err(|_| DomainError::InvalidArgument(format!("unknown severity value: {severity}")))?;
    if s == ProtoSeverity::Unspecified {
        return Ok("S2".to_string()); // default severity
    }
    Ok(s.as_str_name().to_string())
}

fn str_to_proto_severity(s: &str) -> i32 {
    ProtoSeverity::from_str_name(s).map(|v| v as i32).unwrap_or(0)
}

fn proto_issue_type_to_str(issue_type: i32) -> Result<String, DomainError> {
    let t = ProtoIssueType::try_from(issue_type)
        .map_err(|_| DomainError::InvalidArgument(format!("unknown issue type value: {issue_type}")))?;
    if t == ProtoIssueType::Unspecified {
        return Err(DomainError::InvalidArgument("type must be specified".into()));
    }
    Ok(t.as_str_name().to_string())
}

fn str_to_proto_issue_type(s: &str) -> i32 {
    ProtoIssueType::from_str_name(s).map(|v| v as i32).unwrap_or(0)
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

pub fn issue_to_proto(issue: &Issue) -> ProtoIssue {
    ProtoIssue {
        issue_id: issue.id as i64,
        title: issue.title.clone(),
        description: issue.description.clone(),
        status: str_to_proto_status(&issue.status),
        priority: str_to_proto_priority(&issue.priority),
        severity: str_to_proto_severity(&issue.severity),
        r#type: str_to_proto_issue_type(&issue.issue_type),
        component_id: issue.component_id as i64,
        assignee: issue.assignee.clone(),
        reporter: issue.reporter.clone(),
        verifier: issue.verifier.clone(),
        create_time: parse_timestamp(&issue.created_at),
        modify_time: parse_timestamp(&issue.modified_at),
        resolve_time: issue.resolved_at.as_deref().and_then(parse_timestamp),
        verify_time: issue.verified_at.as_deref().and_then(parse_timestamp),
        vote_count: issue.vote_count,
        duplicate_count: issue.duplicate_count,
        found_in: issue.found_in.clone(),
        targeted_to: issue.targeted_to.clone(),
        verified_in: issue.verified_in.clone(),
        in_prod: issue.in_prod,
        archived: issue.archived,
        access_level: issue.access_level.clone(),
    }
}

async fn log_event<C: Connection>(
    conn: &C,
    event_type: &str,
    entity_id: i32,
    payload: &serde_json::Value,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let stmt = Query::table("EventLog")
        .create()
        .set("eventTime", Value::Text(now))
        .set("eventType", Value::Text(event_type.to_string()))
        .set("actor", Value::Text("system".to_string()))
        .set("entityType", Value::Text("Issue".to_string()))
        .set("entityId", Value::Int(entity_id as i64))
        .set("payload", Value::Text(payload.to_string()))
        .build();
    conn.execute(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(())
}

async fn validate_issue_exists<C: Connection>(
    conn: &C,
    issue_id: i64,
) -> Result<Issue, DomainError> {
    let stmt = Query::table("Issue")
        .find_first()
        .filter(Filter::eq("id", Value::Int(issue_id)))
        .build();
    let row = conn
        .query_optional(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    match row {
        Some(r) => Ok(Issue::try_from(&r)?),
        None => Err(DomainError::NotFound(format!("issue {issue_id} not found"))),
    }
}

/// BFS cycle detection: check if adding child_id -> parent_id would create a cycle.
/// A cycle exists if parent_id is already a descendant of child_id.
async fn would_create_cycle<C: Connection>(
    conn: &C,
    child_id: i32,
    parent_id: i32,
) -> Result<bool, DomainError> {
    // Check if child_id is an ancestor of parent_id (BFS up from parent_id)
    let mut visited = HashSet::new();
    let mut queue = vec![parent_id];

    while let Some(current) = queue.pop() {
        if current == child_id {
            return Ok(true);
        }
        if !visited.insert(current) {
            continue;
        }
        // Get parents of current
        let stmt = Query::table("IssueParent")
            .find_many()
            .filter(Filter::eq("childId", Value::Int(current as i64)))
            .build();
        let rows = conn
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        for row in rows {
            let link = IssueParent::try_from(&row)?;
            queue.push(link.parent_id);
        }
    }
    Ok(false)
}

async fn count_direct_children<C: Connection>(
    conn: &C,
    parent_id: i32,
) -> Result<i64, DomainError> {
    let stmt = Query::table("IssueParent")
        .find_many()
        .filter(Filter::eq("parentId", Value::Int(parent_id as i64)))
        .build();
    let rows = conn
        .query(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(rows.len() as i64)
}

async fn get_issues_by_ids<C: Connection>(
    conn: &C,
    ids: &[i32],
) -> Result<Vec<Issue>, DomainError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let id_values: Vec<Value> = ids.iter().map(|&id| Value::Int(id as i64)).collect();
    let stmt = Query::table("Issue")
        .find_many()
        .filter(Filter::is_in("id", id_values))
        .build();
    let rows = conn
        .query(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    rows.iter().map(|r| Ok(Issue::try_from(r)?)).collect()
}

#[tonic::async_trait]
impl IssueService for IssueServiceImpl {
    async fn create_issue(
        &self,
        request: Request<CreateIssueRequest>,
    ) -> Result<Response<ProtoIssue>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        // Validate required fields
        if req.title.trim().is_empty() {
            return Err(DomainError::InvalidArgument("title must not be empty".to_string()).into());
        }

        let priority = proto_priority_to_str(req.priority)?;
        let issue_type = proto_issue_type_to_str(req.r#type)?;
        let severity = proto_severity_to_str(req.severity.unwrap_or(0))?;

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Validate component exists
        let comp_stmt = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.component_id as i64)))
            .build();
        let comp_row = tx
            .query_optional(&comp_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if comp_row.is_none() {
            return Err(DomainError::NotFound(format!(
                "component {} not found",
                req.component_id
            ))
            .into());
        }

        permissions::check_component_permission_quiver(
            &tx,
            req.component_id,
            user_id.as_deref(),
            permissions::ComponentPermission::CreateIssues,
            None,
            &user_groups,
        )
        .await?;

        // Determine initial status based on assignee
        let assignee = req.assignee.unwrap_or_default();
        let initial_status = if assignee.is_empty() {
            "NEW".to_string()
        } else {
            "ASSIGNED".to_string()
        };

        let now = chrono::Utc::now().to_rfc3339();
        let create_stmt = Query::table("Issue")
            .create()
            .set("title", Value::Text(req.title.clone()))
            .set("description", Value::Text(req.description.clone()))
            .set("status", Value::Text(initial_status))
            .set("priority", Value::Text(priority))
            .set("severity", Value::Text(severity))
            .set("issueType", Value::Text(issue_type))
            .set("componentId", Value::Int(req.component_id as i64))
            .set("assignee", Value::Text(assignee.clone()))
            .set("reporter", Value::Text(req.reporter.clone().unwrap_or_default()))
            .set("verifier", Value::Text(req.verifier.clone().unwrap_or_default()))
            .set("foundIn", Value::Text(req.found_in.clone().unwrap_or_default()))
            .set("targetedTo", Value::Text(req.targeted_to.clone().unwrap_or_default()))
            .set("verifiedIn", Value::Text(String::new()))
            .set("inProd", Value::Bool(false))
            .set("archived", Value::Bool(false))
            .set("accessLevel", Value::Text("DEFAULT".to_string()))
            .set("voteCount", Value::Int(0))
            .set("duplicateCount", Value::Int(0))
            .set("createdAt", Value::Text(now.clone()))
            .set("modifiedAt", Value::Text(now.clone()))
            .build();

        tx.execute(&create_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let fetch_stmt = Query::raw("SELECT * FROM Issue WHERE id = last_insert_rowid()").build();
        let issue_row = tx
            .query_one(&fetch_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let issue = Issue::try_from(&issue_row).map_err(DomainError::from)?;

        // Auto-create description comment (comment #1)
        if !issue.description.is_empty() {
            let comment_now = chrono::Utc::now().to_rfc3339();
            let comment_stmt = Query::table("Comment")
                .create()
                .set("issueId", Value::Int(issue.id as i64))
                .set("author", Value::Text(issue.reporter.clone()))
                .set("body", Value::Text(issue.description.clone()))
                .set("isDescription", Value::Bool(true))
                .set("hidden", Value::Bool(false))
                .set("createdAt", Value::Text(comment_now))
                .build();
            tx.execute(&comment_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        // Log event
        log_event(
            &tx,
            "ISSUE_CREATED",
            issue.id,
            &json!({"title": issue.title, "component_id": issue.component_id}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(issue_to_proto(&issue)))
    }

    async fn get_issue(
        &self,
        request: Request<GetIssueRequest>,
    ) -> Result<Response<ProtoIssue>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let stmt = Query::table("Issue")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        let row_opt = tx
            .query_optional(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = match row_opt {
            Some(r) => Issue::try_from(&r).map_err(DomainError::from)?,
            None => {
                return Err(
                    DomainError::NotFound(format!("issue {} not found", req.issue_id)).into(),
                )
            }
        };

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(issue_to_proto(&issue)))
    }

    async fn list_issues(
        &self,
        request: Request<ListIssuesRequest>,
    ) -> Result<Response<ListIssuesResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let page_size = if req.page_size > 0 {
            req.page_size.min(100)
        } else {
            50
        };

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        permissions::check_component_permission_quiver(
            &tx,
            req.component_id,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            None,
            &user_groups,
        )
        .await?;

        // Build status filter
        let open_statuses = vec![
            Value::Text("NEW".to_string()),
            Value::Text("ASSIGNED".to_string()),
            Value::Text("IN_PROGRESS".to_string()),
            Value::Text("INACTIVE".to_string()),
        ];
        let closed_statuses = vec![
            Value::Text("FIXED".to_string()),
            Value::Text("FIXED_VERIFIED".to_string()),
            Value::Text("WONT_FIX_INFEASIBLE".to_string()),
            Value::Text("WONT_FIX_NOT_REPRODUCIBLE".to_string()),
            Value::Text("WONT_FIX_OBSOLETE".to_string()),
            Value::Text("WONT_FIX_INTENDED_BEHAVIOR".to_string()),
            Value::Text("DUPLICATE".to_string()),
        ];

        let component_filter = Filter::eq("componentId", Value::Int(req.component_id as i64));
        let status_filter = match req.status_filter.as_str() {
            "closed" => Some(Filter::is_in("status", closed_statuses)),
            "all" => None,
            _ => Some(Filter::is_in("status", open_statuses)),
        };

        let combined_filter = match status_filter {
            Some(sf) => Filter::and(vec![component_filter, sf]),
            None => component_filter,
        };

        let mut query_builder = Query::table("Issue")
            .find_many()
            .filter(combined_filter)
            .order_by("modifiedAt", Order::Desc)
            .limit(page_size as u64);

        if !req.page_token.is_empty() {
            let cursor_id = req.page_token.parse::<i64>().map_err(|_| {
                DomainError::InvalidArgument("invalid page_token".to_string())
            })?;
            query_builder = query_builder.filter(Filter::lt("id", Value::Int(cursor_id)));
        }

        let stmt = query_builder.build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issues: Vec<Issue> = rows.iter().map(|r| Issue::try_from(r).map_err(DomainError::from)).collect::<Result<_, _>>()?;

        let next_page_token = if issues.len() == page_size as usize {
            issues
                .last()
                .map(|i| i.id.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let proto_issues: Vec<ProtoIssue> = issues.iter().map(issue_to_proto).collect();

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(ListIssuesResponse {
            issues: proto_issues,
            next_page_token,
        }))
    }

    async fn update_issue(
        &self,
        request: Request<UpdateIssueRequest>,
    ) -> Result<Response<ProtoIssue>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        // Validate title if being updated
        if let Some(ref title) = req.title {
            if title.trim().is_empty() {
                return Err(
                    DomainError::InvalidArgument("title must not be empty".to_string()).into(),
                );
            }
        }

        // Validate enum values if provided
        let priority = match req.priority {
            Some(p) => Some(proto_priority_to_str(p)?),
            None => None,
        };
        let severity = match req.severity {
            Some(s) => Some(proto_severity_to_str(s)?),
            None => None,
        };
        let issue_type = match req.r#type {
            Some(t) => Some(proto_issue_type_to_str(t)?),
            None => None,
        };

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Fetch existing issue
        let existing = validate_issue_exists(&tx, req.issue_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            existing.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        // Validate component if being changed
        if let Some(component_id) = req.component_id {
            let comp_stmt = Query::table("Component")
                .find_first()
                .filter(Filter::eq("id", Value::Int(component_id as i64)))
                .build();
            let comp_row = tx
                .query_optional(&comp_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            if comp_row.is_none() {
                return Err(DomainError::NotFound(format!(
                    "component {component_id} not found"
                ))
                .into());
            }
        }

        // Determine assignee change for auto-transitions
        let assignee_changed = req.assignee.is_some();
        let new_assignee = req
            .assignee
            .as_deref()
            .unwrap_or(&existing.assignee);

        // Determine status
        let mut new_status = if let Some(status_val) = req.status {
            let status_str = proto_status_to_str(status_val)?;
            // Validate the explicit transition
            status_machine::validate_transition(&existing.status, &status_str)?;
            Some(status_str)
        } else {
            None
        };

        // Apply auto-transition if no explicit status change
        if new_status.is_none() {
            if let Some(auto_status) =
                status_machine::auto_transition(&existing.status, assignee_changed, new_assignee)
            {
                new_status = Some(auto_status.to_string());
            }
        }

        // Build conditional timestamps (resolved_at and verified_at are business logic)
        let now = chrono::Utc::now().to_rfc3339();
        let resolved_at = match new_status.as_deref() {
            Some(s) if status_machine::is_closed(s) && existing.resolved_at.is_none() => {
                Some(now.clone())
            }
            _ => None,
        };
        let verified_at = match new_status.as_deref() {
            Some("FIXED_VERIFIED") if existing.verified_at.is_none() => Some(now.clone()),
            _ => None,
        };

        // Build update query - only set fields that are provided
        let mut update = Query::table("Issue").update();

        if let Some(ref t) = req.title {
            update = update.set("title", Value::Text(t.clone()));
        }
        if let Some(ref d) = req.description {
            update = update.set("description", Value::Text(d.clone()));
        }
        if let Some(ref s) = new_status {
            update = update.set("status", Value::Text(s.clone()));
        }
        if let Some(ref p) = priority {
            update = update.set("priority", Value::Text(p.clone()));
        }
        if let Some(ref sv) = severity {
            update = update.set("severity", Value::Text(sv.clone()));
        }
        if let Some(ref it) = issue_type {
            update = update.set("issueType", Value::Text(it.clone()));
        }
        if let Some(cid) = req.component_id {
            update = update.set("componentId", Value::Int(cid as i64));
        }
        if let Some(ref a) = req.assignee {
            update = update.set("assignee", Value::Text(a.clone()));
        }
        if let Some(ref r) = req.reporter {
            update = update.set("reporter", Value::Text(r.clone()));
        }
        if let Some(ref v) = req.verifier {
            update = update.set("verifier", Value::Text(v.clone()));
        }
        if let Some(ref fi) = req.found_in {
            update = update.set("foundIn", Value::Text(fi.clone()));
        }
        if let Some(ref tt) = req.targeted_to {
            update = update.set("targetedTo", Value::Text(tt.clone()));
        }
        if let Some(ref vi) = req.verified_in {
            update = update.set("verifiedIn", Value::Text(vi.clone()));
        }
        if let Some(ip) = req.in_prod {
            update = update.set("inProd", Value::Bool(ip));
        }
        if let Some(ar) = req.archived {
            update = update.set("archived", Value::Bool(ar));
        }
        if let Some(ref ra) = resolved_at {
            update = update.set("resolvedAt", Value::Text(ra.clone()));
        }
        if let Some(ref va) = verified_at {
            update = update.set("verifiedAt", Value::Text(va.clone()));
        }
        update = update.set("modifiedAt", Value::Text(now.clone()));

        let update_stmt = update
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        tx.execute(&update_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch the updated issue
        let fetch_stmt = Query::table("Issue")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        let fetch_row = tx
            .query_one(&fetch_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let issue = Issue::try_from(&fetch_row).map_err(DomainError::from)?;

        // Log event
        log_event(
            &tx,
            "ISSUE_UPDATED",
            issue.id,
            &json!({"issue_id": req.issue_id, "modified_at": now}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(issue_to_proto(&issue)))
    }

    // --- Relationship RPCs ---

    async fn add_parent(
        &self,
        request: Request<AddParentRequest>,
    ) -> Result<Response<RelationshipResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if req.child_id == req.parent_id {
            return Err(
                DomainError::InvalidArgument("issue cannot be its own parent".to_string()).into(),
            );
        }

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Validate both issues exist
        let child_issue = validate_issue_exists(&tx, req.child_id).await?;
        validate_issue_exists(&tx, req.parent_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            child_issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.child_id),
            &user_groups,
        )
        .await?;

        // Check max children (500)
        let child_count = count_direct_children(&tx, req.parent_id as i32).await?;
        if child_count >= 500 {
            return Err(DomainError::FailedPrecondition(format!(
                "parent issue {} already has 500 children",
                req.parent_id
            ))
            .into());
        }

        // Cycle detection
        if would_create_cycle(&tx, req.child_id as i32, req.parent_id as i32).await? {
            return Err(DomainError::FailedPrecondition(
                "adding this parent would create a cycle".to_string(),
            )
            .into());
        }

        // Create the relationship
        let stmt = Query::table("IssueParent")
            .create()
            .set("childId", Value::Int(req.child_id as i64))
            .set("parentId", Value::Int(req.parent_id as i64))
            .build();

        tx.execute(&stmt)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DomainError::AlreadyExists(format!(
                        "parent relationship {}->{} already exists",
                        req.child_id, req.parent_id
                    ))
                } else {
                    DomainError::Internal(e.to_string())
                }
            })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(RelationshipResponse {}))
    }

    async fn remove_parent(
        &self,
        request: Request<RemoveParentRequest>,
    ) -> Result<Response<RelationshipResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let child_issue = validate_issue_exists(&tx, req.child_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            child_issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.child_id),
            &user_groups,
        )
        .await?;

        // Find the link
        let find_stmt = Query::table("IssueParent")
            .find_many()
            .filter(Filter::and(vec![
                Filter::eq("childId", Value::Int(req.child_id as i64)),
                Filter::eq("parentId", Value::Int(req.parent_id as i64)),
            ]))
            .build();
        let rows = tx
            .query(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Err(DomainError::NotFound(format!(
                "parent relationship {}->{} not found",
                req.child_id, req.parent_id
            ))
            .into());
        }

        let link = IssueParent::try_from(&rows[0]).map_err(DomainError::from)?;
        let delete_stmt = Query::table("IssueParent")
            .delete()
            .filter(Filter::eq("id", Value::Int(link.id as i64)))
            .build();
        tx.execute(&delete_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(RelationshipResponse {}))
    }

    async fn list_parents(
        &self,
        request: Request<ListRelatedIssuesRequest>,
    ) -> Result<Response<ListRelatedIssuesResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = validate_issue_exists(&tx, req.issue_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        let stmt = Query::table("IssueParent")
            .find_many()
            .filter(Filter::eq("childId", Value::Int(req.issue_id as i64)))
            .build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let parent_ids: Vec<i32> = rows
            .iter()
            .map(|r| IssueParent::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|l| l.parent_id)
            .collect();

        let issues = get_issues_by_ids(&tx, &parent_ids).await?;
        let proto_issues: Vec<ProtoIssue> = issues.iter().map(issue_to_proto).collect();

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(ListRelatedIssuesResponse {
            issues: proto_issues,
        }))
    }

    async fn list_children(
        &self,
        request: Request<ListRelatedIssuesRequest>,
    ) -> Result<Response<ListRelatedIssuesResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = validate_issue_exists(&tx, req.issue_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        let stmt = Query::table("IssueParent")
            .find_many()
            .filter(Filter::eq("parentId", Value::Int(req.issue_id as i64)))
            .build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let child_ids: Vec<i32> = rows
            .iter()
            .map(|r| IssueParent::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|l| l.child_id)
            .collect();

        let issues = get_issues_by_ids(&tx, &child_ids).await?;
        let proto_issues: Vec<ProtoIssue> = issues.iter().map(issue_to_proto).collect();

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(ListRelatedIssuesResponse {
            issues: proto_issues,
        }))
    }

    async fn add_blocking(
        &self,
        request: Request<AddBlockingRequest>,
    ) -> Result<Response<RelationshipResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if req.blocking_id == req.blocked_id {
            return Err(
                DomainError::InvalidArgument("issue cannot block itself".to_string()).into(),
            );
        }

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let blocking_issue = validate_issue_exists(&tx, req.blocking_id).await?;
        validate_issue_exists(&tx, req.blocked_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            blocking_issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.blocking_id),
            &user_groups,
        )
        .await?;

        let stmt = Query::table("IssueBlocking")
            .create()
            .set("blockingId", Value::Int(req.blocking_id as i64))
            .set("blockedId", Value::Int(req.blocked_id as i64))
            .build();

        tx.execute(&stmt)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DomainError::AlreadyExists(format!(
                        "blocking relationship {}->{} already exists",
                        req.blocking_id, req.blocked_id
                    ))
                } else {
                    DomainError::Internal(e.to_string())
                }
            })?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(RelationshipResponse {}))
    }

    async fn remove_blocking(
        &self,
        request: Request<RemoveBlockingRequest>,
    ) -> Result<Response<RelationshipResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let blocking_issue = validate_issue_exists(&tx, req.blocking_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            blocking_issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.blocking_id),
            &user_groups,
        )
        .await?;

        let find_stmt = Query::table("IssueBlocking")
            .find_many()
            .filter(Filter::and(vec![
                Filter::eq("blockingId", Value::Int(req.blocking_id as i64)),
                Filter::eq("blockedId", Value::Int(req.blocked_id as i64)),
            ]))
            .build();
        let rows = tx
            .query(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Err(DomainError::NotFound(format!(
                "blocking relationship {}->{} not found",
                req.blocking_id, req.blocked_id
            ))
            .into());
        }

        let link = IssueBlocking::try_from(&rows[0]).map_err(DomainError::from)?;
        let delete_stmt = Query::table("IssueBlocking")
            .delete()
            .filter(Filter::eq("id", Value::Int(link.id as i64)))
            .build();
        tx.execute(&delete_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(RelationshipResponse {}))
    }

    async fn mark_duplicate(
        &self,
        request: Request<MarkDuplicateRequest>,
    ) -> Result<Response<ProtoIssue>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if req.issue_id == req.canonical_id {
            return Err(
                DomainError::InvalidArgument("issue cannot be duplicate of itself".to_string())
                    .into(),
            );
        }

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = validate_issue_exists(&tx, req.issue_id).await?;
        let canonical = validate_issue_exists(&tx, req.canonical_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        // Set issue status to DUPLICATE and store duplicate_of
        let now = chrono::Utc::now().to_rfc3339();
        let mut update = Query::table("Issue")
            .update()
            .set("status", Value::Text("DUPLICATE".to_string()))
            .set("duplicateOfId", Value::Int(req.canonical_id as i64))
            .set("modifiedAt", Value::Text(now.clone()));

        if issue.resolved_at.is_none() {
            update = update.set("resolvedAt", Value::Text(now.clone()));
        }

        let update_stmt = update
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        tx.execute(&update_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch the updated issue
        let fetch_stmt = Query::table("Issue")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        let fetch_row = tx
            .query_one(&fetch_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let updated_issue = Issue::try_from(&fetch_row).map_err(DomainError::from)?;

        // Increment duplicate_count on canonical
        let dup_count_stmt = Query::table("Issue")
            .update()
            .set("duplicateCount", Value::Int((canonical.duplicate_count + 1) as i64))
            .set("modifiedAt", Value::Text(chrono::Utc::now().to_rfc3339()))
            .filter(Filter::eq("id", Value::Int(req.canonical_id as i64)))
            .build();
        tx.execute(&dup_count_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(issue_to_proto(&updated_issue)))
    }

    async fn unmark_duplicate(
        &self,
        request: Request<UnmarkDuplicateRequest>,
    ) -> Result<Response<ProtoIssue>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let user_groups = match user_id.as_deref() {
            Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(),
            None => vec![],
        };

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let issue = validate_issue_exists(&tx, req.issue_id).await?;

        permissions::check_component_permission_quiver(
            &tx,
            issue.component_id as i64,
            user_id.as_deref(),
            permissions::ComponentPermission::EditIssues,
            Some(req.issue_id),
            &user_groups,
        )
        .await?;

        if issue.status != "DUPLICATE" {
            return Err(DomainError::FailedPrecondition(format!(
                "issue {} is not marked as duplicate (status: {})",
                req.issue_id, issue.status
            ))
            .into());
        }

        let canonical_id = issue.duplicate_of_id;

        // Restore status based on assignee
        let new_status = if issue.assignee.is_empty() {
            "NEW"
        } else {
            "ASSIGNED"
        };

        let now = chrono::Utc::now().to_rfc3339();
        let update_stmt = Query::table("Issue")
            .update()
            .set("status", Value::Text(new_status.to_string()))
            .set("modifiedAt", Value::Text(now.clone()))
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        tx.execute(&update_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch the updated issue
        let fetch_stmt = Query::table("Issue")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.issue_id as i64)))
            .build();
        let fetch_row = tx
            .query_one(&fetch_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let updated_issue = Issue::try_from(&fetch_row).map_err(DomainError::from)?;

        // Decrement duplicate_count on canonical
        if let Some(cid) = canonical_id {
            let canonical = validate_issue_exists(&tx, cid as i64).await?;
            let new_count = (canonical.duplicate_count - 1).max(0);
            let dec_stmt = Query::table("Issue")
                .update()
                .set("duplicateCount", Value::Int(new_count as i64))
                .set("modifiedAt", Value::Text(chrono::Utc::now().to_rfc3339()))
                .filter(Filter::eq("id", Value::Int(cid as i64)))
                .build();
            tx.execute(&dec_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(issue_to_proto(&updated_issue)))
    }
}
