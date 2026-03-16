use std::sync::Arc;

use identity::IdentityProvider;
use quiver_driver_core::{Connection, Pool, Transaction, Transactional, Value};
use quiver_query::{Filter, Order, Query};
use tonic::{Request, Response, Status};

use crate::db::row_mapping::{Hotlist, HotlistIssue};
use crate::db::DbConn;
use crate::domain::permissions;
use crate::domain::types::DomainError;

use crate::proto::hotlist_service_server::HotlistService;
use crate::proto::{
    AddIssueToHotlistRequest, CreateHotlistRequest, GetHotlistRequest, Hotlist as ProtoHotlist,
    HotlistIssue as ProtoHotlistIssue, ListHotlistIssuesRequest, ListHotlistIssuesResponse,
    ListHotlistsRequest, ListHotlistsResponse, RemoveIssueFromHotlistRequest,
    RemoveIssueFromHotlistResponse, ReorderHotlistIssuesRequest, ReorderHotlistIssuesResponse,
    UpdateHotlistRequest,
};

pub struct HotlistServiceImpl {
    pub db: DbConn,
    pub identity: Arc<dyn IdentityProvider>,
}

fn hotlist_to_proto(h: &Hotlist, issue_count: i32) -> ProtoHotlist {
    ProtoHotlist {
        hotlist_id: h.id as i64,
        name: h.name.clone(),
        description: h.description.clone(),
        owner: h.owner.clone(),
        archived: h.archived,
        create_time: parse_timestamp(&h.created_at),
        modify_time: parse_timestamp(&h.modified_at),
        issue_count,
    }
}

fn hotlist_issue_to_proto(hi: &HotlistIssue) -> ProtoHotlistIssue {
    ProtoHotlistIssue {
        hotlist_id: hi.hotlist_id as i64,
        issue_id: hi.issue_id as i64,
        position: hi.position,
        add_time: parse_timestamp(&hi.added_at),
        added_by: hi.added_by.clone(),
    }
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

async fn log_event<C: Connection>(
    conn: &C,
    event_type: &str,
    entity_id: i32,
    payload: &serde_json::Value,
) -> Result<(), DomainError> {
    let stmt = Query::table("EventLog")
        .create()
        .set("eventTime", Value::Text(chrono::Utc::now().to_rfc3339()))
        .set("eventType", Value::Text(event_type.to_string()))
        .set("actor", Value::Text("system".to_string()))
        .set("entityType", Value::Text("Hotlist".to_string()))
        .set("entityId", Value::Int(entity_id as i64))
        .set("payload", Value::Text(payload.to_string()))
        .build();
    conn.execute(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(())
}

async fn get_hotlist_by_id<C: Connection>(conn: &C, id: i32) -> Result<Hotlist, DomainError> {
    let stmt = Query::table("Hotlist")
        .find_first()
        .filter(Filter::eq("id", Value::Int(id as i64)))
        .build();
    let row = conn
        .query_optional(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let row = row.ok_or_else(|| DomainError::NotFound(format!("hotlist {id} not found")))?;
    Ok(Hotlist::try_from(&row)?)
}

async fn count_issues<C: Connection>(conn: &C, hotlist_id: i32) -> Result<i32, DomainError> {
    let stmt = Query::table("HotlistIssue")
        .find_many()
        .filter(Filter::eq("hotlistId", Value::Int(hotlist_id as i64)))
        .build();
    let rows = conn
        .query(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(rows.len() as i32)
}

#[tonic::async_trait]
impl HotlistService for HotlistServiceImpl {
    async fn create_hotlist(
        &self,
        request: Request<CreateHotlistRequest>,
    ) -> Result<Response<ProtoHotlist>, Status> {
        let req = request.into_inner();

        if req.name.trim().is_empty() {
            return Err(DomainError::InvalidArgument("name must not be empty".to_string()).into());
        }

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let now = Value::Text(chrono::Utc::now().to_rfc3339());
        let stmt = Query::table("Hotlist")
            .create()
            .set("name", Value::Text(req.name))
            .set("description", Value::Text(req.description))
            .set("owner", Value::Text(req.owner))
            .set("archived", Value::Bool(false))
            .set("createdAt", now.clone())
            .set("modifiedAt", now)
            .build();
        tx.execute(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let fetch = Query::raw("SELECT * FROM Hotlist WHERE id = last_insert_rowid()").build();
        let row = tx
            .query_one(&fetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let hotlist = Hotlist::try_from(&row).map_err(DomainError::from)?;

        log_event(
            &tx,
            "HOTLIST_CREATED",
            hotlist.id,
            &serde_json::json!({"name": hotlist.name}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(hotlist_to_proto(&hotlist, 0)))
    }

    async fn get_hotlist(
        &self,
        request: Request<GetHotlistRequest>,
    ) -> Result<Response<ProtoHotlist>, Status> {
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

        permissions::check_hotlist_permission_quiver(
            &tx,
            req.hotlist_id,
            user_id.as_deref(),
            "HOTLIST_VIEW",
            &user_groups,
        )
        .await?;

        let hotlist = get_hotlist_by_id(&tx, req.hotlist_id as i32).await?;
        let count = count_issues(&tx, hotlist.id).await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(hotlist_to_proto(&hotlist, count)))
    }

    async fn list_hotlists(
        &self,
        request: Request<ListHotlistsRequest>,
    ) -> Result<Response<ListHotlistsResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();
        let page_size = if req.page_size > 0 {
            req.page_size.min(100)
        } else {
            50
        };

        let user_groups = match user_id.as_deref() {
            Some(uid) => self
                .identity
                .resolve_user_groups(uid)
                .await
                .unwrap_or_default(),
            None => vec![],
        };

        let mut q = Query::table("Hotlist").find_many();

        match req.filter.as_str() {
            "archived" => {
                q = q.filter(Filter::eq("archived", Value::Bool(true)));
            }
            "all" => {}
            _ => {
                // "active" or default
                q = q.filter(Filter::eq("archived", Value::Bool(false)));
            }
        }

        q = q.order_by("id", Order::Desc).limit((page_size + 1) as u64);

        if !req.page_token.is_empty() {
            let cursor_id = req
                .page_token
                .parse::<i64>()
                .map_err(|_| DomainError::InvalidArgument("invalid page_token".to_string()))?;
            q = q.filter(Filter::lt("id", Value::Int(cursor_id)));
        }

        let stmt = q.build();

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Get accessible hotlist IDs
        let accessible_ids = permissions::get_accessible_hotlist_ids(
            &tx,
            user_id.as_deref(),
            "HOTLIST_VIEW",
            &user_groups,
        )
        .await?;

        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let all_items: Vec<Hotlist> = rows
            .iter()
            .map(|r| Hotlist::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        // Filter to only accessible hotlists
        let items: Vec<&Hotlist> = all_items
            .iter()
            .filter(|h| accessible_ids.contains(&(h.id as i64)))
            .collect();

        let has_more = items.len() > page_size as usize;
        let items = if has_more {
            &items[..page_size as usize]
        } else {
            &items[..]
        };

        let mut hotlists = Vec::new();
        for h in items {
            let count = count_issues(&tx, h.id).await?;
            hotlists.push(hotlist_to_proto(h, count));
        }

        let next_page_token = if has_more {
            items.last().map(|h| h.id.to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(ListHotlistsResponse {
            hotlists,
            next_page_token,
        }))
    }

    async fn update_hotlist(
        &self,
        request: Request<UpdateHotlistRequest>,
    ) -> Result<Response<ProtoHotlist>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if let Some(ref name) = req.name {
            if name.trim().is_empty() {
                return Err(
                    DomainError::InvalidArgument("name must not be empty".to_string()).into(),
                );
            }
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

        permissions::check_hotlist_permission_quiver(
            &tx,
            req.hotlist_id,
            user_id.as_deref(),
            "HOTLIST_ADMIN",
            &user_groups,
        )
        .await?;

        // Verify the hotlist exists before updating
        get_hotlist_by_id(&tx, req.hotlist_id as i32).await?;

        let mut update_q = Query::table("Hotlist")
            .update()
            .filter(Filter::eq("id", Value::Int(req.hotlist_id)))
            .set("modifiedAt", Value::Text(chrono::Utc::now().to_rfc3339()));

        if let Some(name) = req.name {
            update_q = update_q.set("name", Value::Text(name));
        }
        if let Some(description) = req.description {
            update_q = update_q.set("description", Value::Text(description));
        }
        if let Some(archived) = req.archived {
            update_q = update_q.set("archived", Value::Bool(archived));
        }

        let stmt = update_q.build();
        tx.execute(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch updated hotlist
        let hotlist = get_hotlist_by_id(&tx, req.hotlist_id as i32).await?;
        let count = count_issues(&tx, hotlist.id).await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(hotlist_to_proto(&hotlist, count)))
    }

    async fn add_issue(
        &self,
        request: Request<AddIssueToHotlistRequest>,
    ) -> Result<Response<ProtoHotlistIssue>, Status> {
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

        permissions::check_hotlist_permission_quiver(
            &tx,
            req.hotlist_id,
            user_id.as_deref(),
            "HOTLIST_VIEW_APPEND",
            &user_groups,
        )
        .await?;

        // Validate hotlist exists
        get_hotlist_by_id(&tx, req.hotlist_id as i32).await?;

        // Validate issue exists
        let issue_stmt = Query::table("Issue")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.issue_id)))
            .build();
        let issue_row = tx
            .query_optional(&issue_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if issue_row.is_none() {
            return Err(DomainError::NotFound(format!("issue {} not found", req.issue_id)).into());
        }

        // Check for duplicate membership
        let existing_stmt = Query::table("HotlistIssue")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("hotlistId", Value::Int(req.hotlist_id)),
                Filter::eq("issueId", Value::Int(req.issue_id)),
            ]))
            .build();
        let existing_row = tx
            .query_optional(&existing_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if existing_row.is_some() {
            return Err(DomainError::AlreadyExists(format!(
                "issue {} is already in hotlist {}",
                req.issue_id, req.hotlist_id
            ))
            .into());
        }

        // Get max position
        let max_pos_stmt = Query::table("HotlistIssue")
            .find_many()
            .filter(Filter::eq("hotlistId", Value::Int(req.hotlist_id)))
            .order_by("position", Order::Desc)
            .limit(1)
            .build();
        let max_pos_rows = tx
            .query(&max_pos_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let next_position = if let Some(row) = max_pos_rows.first() {
            let item = HotlistIssue::try_from(row).map_err(DomainError::from)?;
            item.position + 1
        } else {
            0
        };

        let now = Value::Text(chrono::Utc::now().to_rfc3339());
        let create_stmt = Query::table("HotlistIssue")
            .create()
            .set("hotlistId", Value::Int(req.hotlist_id))
            .set("issueId", Value::Int(req.issue_id))
            .set("position", Value::Int(next_position as i64))
            .set("addedBy", Value::Text(req.added_by))
            .set("addedAt", now)
            .build();
        tx.execute(&create_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let fetch = Query::raw("SELECT * FROM HotlistIssue WHERE id = last_insert_rowid()").build();
        let row = tx
            .query_one(&fetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let hi = HotlistIssue::try_from(&row).map_err(DomainError::from)?;

        log_event(
            &tx,
            "HOTLIST_ISSUE_ADDED",
            req.hotlist_id as i32,
            &serde_json::json!({"issueId": req.issue_id}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(hotlist_issue_to_proto(&hi)))
    }

    async fn remove_issue(
        &self,
        request: Request<RemoveIssueFromHotlistRequest>,
    ) -> Result<Response<RemoveIssueFromHotlistResponse>, Status> {
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

        permissions::check_hotlist_permission_quiver(
            &tx,
            req.hotlist_id,
            user_id.as_deref(),
            "HOTLIST_VIEW_APPEND",
            &user_groups,
        )
        .await?;

        // Find the membership
        let find_stmt = Query::table("HotlistIssue")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("hotlistId", Value::Int(req.hotlist_id)),
                Filter::eq("issueId", Value::Int(req.issue_id)),
            ]))
            .build();
        let find_row = tx
            .query_optional(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let membership_row = find_row.ok_or_else(|| {
            DomainError::NotFound(format!(
                "issue {} is not in hotlist {}",
                req.issue_id, req.hotlist_id
            ))
        })?;
        let membership = HotlistIssue::try_from(&membership_row).map_err(DomainError::from)?;

        // Delete the membership by its primary key
        let del_stmt = Query::table("HotlistIssue")
            .delete()
            .filter(Filter::eq("id", Value::Int(membership.id as i64)))
            .build();
        tx.execute(&del_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        log_event(
            &tx,
            "HOTLIST_ISSUE_REMOVED",
            req.hotlist_id as i32,
            &serde_json::json!({"issueId": req.issue_id}),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(RemoveIssueFromHotlistResponse {}))
    }

    async fn list_issues(
        &self,
        request: Request<ListHotlistIssuesRequest>,
    ) -> Result<Response<ListHotlistIssuesResponse>, Status> {
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

        permissions::check_hotlist_permission_quiver(
            &tx,
            req.hotlist_id,
            user_id.as_deref(),
            "HOTLIST_VIEW",
            &user_groups,
        )
        .await?;

        // Validate hotlist exists
        get_hotlist_by_id(&tx, req.hotlist_id as i32).await?;

        let stmt = Query::table("HotlistIssue")
            .find_many()
            .filter(Filter::eq("hotlistId", Value::Int(req.hotlist_id)))
            .order_by("position", Order::Asc)
            .build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let items: Vec<HotlistIssue> = rows
            .iter()
            .map(|r| HotlistIssue::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        let issues: Vec<ProtoHotlistIssue> = items.iter().map(hotlist_issue_to_proto).collect();

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(ListHotlistIssuesResponse { issues }))
    }

    async fn reorder_issues(
        &self,
        request: Request<ReorderHotlistIssuesRequest>,
    ) -> Result<Response<ReorderHotlistIssuesResponse>, Status> {
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

        permissions::check_hotlist_permission_quiver(
            &tx,
            req.hotlist_id,
            user_id.as_deref(),
            "HOTLIST_ADMIN",
            &user_groups,
        )
        .await?;

        // Validate hotlist exists
        get_hotlist_by_id(&tx, req.hotlist_id as i32).await?;

        // Get all current memberships
        let all_stmt = Query::table("HotlistIssue")
            .find_many()
            .filter(Filter::eq("hotlistId", Value::Int(req.hotlist_id)))
            .build();
        let all_rows = tx
            .query(&all_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let items: Vec<HotlistIssue> = all_rows
            .iter()
            .map(|r| HotlistIssue::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        // Build a map of issue_id -> membership id
        let mut id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
        for item in &items {
            id_map.insert(item.issue_id, item.id);
        }

        // Update positions based on the new order
        for (position, issue_id) in req.issue_ids.iter().enumerate() {
            let issue_id_i32 = *issue_id as i32;
            if let Some(membership_id) = id_map.get(&issue_id_i32) {
                let update_stmt = Query::table("HotlistIssue")
                    .update()
                    .filter(Filter::eq("id", Value::Int(*membership_id as i64)))
                    .set("position", Value::Int(position as i64))
                    .build();
                tx.execute(&update_stmt)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(ReorderHotlistIssuesResponse {}))
    }
}
