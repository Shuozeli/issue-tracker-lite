use std::sync::Arc;

use quiver_driver_core::{Connection, Pool, Transaction, Transactional, Value};
use quiver_query::{Filter, Order, Query};
use tonic::{Request, Response, Status};

use identity::IdentityProvider;

use crate::db::DbConn;
use crate::db::row_mapping::Component;
use crate::domain::permissions;
use crate::domain::types::DomainError;
use crate::proto::component_service_server::ComponentService;
use crate::proto::{
    Component as ProtoComponent, CreateComponentRequest, DeleteComponentRequest,
    DeleteComponentResponse, GetComponentRequest, ListComponentsRequest,
    ListComponentsResponse, UpdateComponentRequest,
};

pub struct ComponentServiceImpl {
    pub db: DbConn,
    pub identity: Arc<dyn IdentityProvider>,
}

fn component_to_proto(c: &Component) -> ProtoComponent {
    ProtoComponent {
        component_id: c.id as i64,
        name: c.name.clone(),
        description: c.description.clone(),
        parent_id: c.parent_id.map(|id| id as i64),
        expanded_access_enabled: c.expanded_access_enabled,
        editable_comments_enabled: c.editable_comments_enabled,
        create_time: parse_timestamp(&c.created_at),
        update_time: parse_timestamp(&c.updated_at),
        child_count: 0,
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

fn now_utc() -> Value {
    Value::Text(chrono::Utc::now().to_rfc3339())
}

#[tonic::async_trait]
impl ComponentService for ComponentServiceImpl {
    async fn create_component(
        &self,
        request: Request<CreateComponentRequest>,
    ) -> Result<Response<ProtoComponent>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if req.name.trim().is_empty() {
            return Err(DomainError::InvalidArgument("name must not be empty".to_string()).into());
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

        // Validate parent exists if specified
        if let Some(parent_id) = req.parent_id {
            let find_parent = Query::table("Component")
                .find_first()
                .filter(Filter::eq("id", Value::Int(parent_id as i64)))
                .build();
            let parent_row = tx
                .query_optional(&find_parent)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            if parent_row.is_none() {
                return Err(
                    DomainError::NotFound(format!("parent component {parent_id} not found")).into(),
                );
            }
            permissions::check_component_permission_quiver(
                &tx,
                parent_id,
                user_id.as_deref(),
                permissions::ComponentPermission::AdminComponents,
                None,
                &user_groups,
            )
            .await?;
        }

        let now = now_utc();
        let mut create_q = Query::table("Component")
            .create()
            .set("name", Value::Text(req.name))
            .set("description", Value::Text(req.description))
            .set("expandedAccessEnabled", Value::Bool(true))
            .set("editableCommentsEnabled", Value::Bool(false))
            .set("createdAt", now.clone())
            .set("updatedAt", now);

        if let Some(parent_id) = req.parent_id {
            create_q = create_q.set("parentId", Value::Int(parent_id as i64));
        }

        let stmt = create_q.build();
        tx.execute(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // No RETURNING clause -- fetch the created row via last_insert_rowid
        let fetch = Query::raw("SELECT * FROM Component WHERE id = last_insert_rowid()").build();
        let row = tx
            .query_one(&fetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let component = Component::try_from(&row).map_err(DomainError::from)?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(component_to_proto(&component)))
    }

    async fn get_component(
        &self,
        request: Request<GetComponentRequest>,
    ) -> Result<Response<ProtoComponent>, Status> {
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

        let find_q = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.component_id as i64)))
            .build();

        let row = tx
            .query_optional(&find_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = row.ok_or_else(|| {
            DomainError::NotFound(format!("component {} not found", req.component_id))
        })?;

        permissions::check_component_permission_quiver(
            &tx,
            req.component_id,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewComponents,
            None,
            &user_groups,
        )
        .await?;

        let component = Component::try_from(&row).map_err(DomainError::from)?;

        // Count children
        let count_q = Query::table("Component")
            .find_many()
            .filter(Filter::eq("parentId", Value::Int(req.component_id as i64)))
            .build();
        let children = tx
            .query(&count_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let child_count = children.len() as i32;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut proto = component_to_proto(&component);
        proto.child_count = child_count;

        Ok(Response::new(proto))
    }

    async fn list_components(
        &self,
        request: Request<ListComponentsRequest>,
    ) -> Result<Response<ListComponentsResponse>, Status> {
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

        // Get accessible component IDs for this user
        let accessible_ids = permissions::get_accessible_component_ids(
            &tx,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            &user_groups,
        )
        .await?;

        let mut q = Query::table("Component").find_many();

        // Filter by parent_id
        match req.parent_id {
            Some(pid) => {
                q = q.filter(Filter::eq("parentId", Value::Int(pid as i64)));
            }
            None => {
                q = q.filter(Filter::is_null("parentId"));
            }
        }

        q = q.order_by("name", Order::Asc).limit(page_size as u64);

        // Cursor-based pagination: use id > cursor
        if !req.page_token.is_empty() {
            let cursor_id = req.page_token.parse::<i64>().map_err(|_| {
                DomainError::InvalidArgument("invalid page_token".to_string())
            })?;
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

        let components: Vec<Component> = rows
            .iter()
            .map(|r| Component::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        // Filter to only accessible components
        let components: Vec<Component> = components
            .into_iter()
            .filter(|c| accessible_ids.contains(&(c.id as i64)))
            .collect();

        let next_page_token = if components.len() == page_size as usize {
            components
                .last()
                .map(|c| c.id.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let proto_components: Vec<ProtoComponent> =
            components.iter().map(component_to_proto).collect();

        Ok(Response::new(ListComponentsResponse {
            components: proto_components,
            next_page_token,
        }))
    }

    async fn update_component(
        &self,
        request: Request<UpdateComponentRequest>,
    ) -> Result<Response<ProtoComponent>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        if let Some(ref name) = req.name {
            if name.trim().is_empty() {
                return Err(
                    DomainError::InvalidArgument("name must not be empty".to_string()).into(),
                );
            }
        }

        if let Some(parent_id) = req.parent_id {
            if parent_id == req.component_id {
                return Err(DomainError::InvalidArgument(
                    "component cannot be its own parent".to_string(),
                )
                .into());
            }
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

        // Verify exists
        let find_q = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.component_id as i64)))
            .build();
        let existing = tx
            .query_optional(&find_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if existing.is_none() {
            return Err(
                DomainError::NotFound(format!("component {} not found", req.component_id)).into(),
            );
        }

        permissions::check_component_permission_quiver(
            &tx,
            req.component_id,
            user_id.as_deref(),
            permissions::ComponentPermission::AdminComponents,
            None,
            &user_groups,
        )
        .await?;

        // Validate parent if being updated
        if let Some(parent_id) = req.parent_id {
            let find_parent = Query::table("Component")
                .find_first()
                .filter(Filter::eq("id", Value::Int(parent_id as i64)))
                .build();
            let parent_row = tx
                .query_optional(&find_parent)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            if parent_row.is_none() {
                return Err(
                    DomainError::NotFound(format!("parent component {parent_id} not found")).into(),
                );
            }
        }

        // Build update query
        let mut update_q = Query::table("Component")
            .update()
            .filter(Filter::eq("id", Value::Int(req.component_id as i64)))
            .set("updatedAt", now_utc());

        if let Some(name) = req.name {
            update_q = update_q.set("name", Value::Text(name));
        }
        if let Some(description) = req.description {
            update_q = update_q.set("description", Value::Text(description));
        }
        if let Some(parent_id) = req.parent_id {
            update_q = update_q.set("parentId", Value::Int(parent_id as i64));
        }
        if let Some(expanded) = req.expanded_access_enabled {
            update_q = update_q.set("expandedAccessEnabled", Value::Bool(expanded));
        }
        if let Some(editable) = req.editable_comments_enabled {
            update_q = update_q.set("editableCommentsEnabled", Value::Bool(editable));
        }

        let stmt = update_q.build();
        tx.execute(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Re-fetch updated row (no RETURNING)
        let refetch = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(req.component_id as i64)))
            .build();
        let row = tx
            .query_one(&refetch)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let component = Component::try_from(&row).map_err(DomainError::from)?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(component_to_proto(&component)))
    }

    async fn delete_component(
        &self,
        request: Request<DeleteComponentRequest>,
    ) -> Result<Response<DeleteComponentResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();
        let cid = req.component_id;

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

        // Check exists
        let find_q = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(cid as i64)))
            .build();
        let existing = tx
            .query_optional(&find_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if existing.is_none() {
            return Err(DomainError::NotFound(format!("component {cid} not found")).into());
        }

        permissions::check_component_permission_quiver(
            &tx,
            cid,
            user_id.as_deref(),
            permissions::ComponentPermission::AdminComponents,
            None,
            &user_groups,
        )
        .await?;

        // Check no children
        let children_q = Query::table("Component")
            .find_many()
            .filter(Filter::eq("parentId", Value::Int(cid as i64)))
            .build();
        let children = tx
            .query(&children_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if !children.is_empty() {
            return Err(DomainError::FailedPrecondition(format!(
                "component {cid} has {} children, delete them first",
                children.len()
            ))
            .into());
        }

        // Check no issues
        let issues_q = Query::table("Issue")
            .find_many()
            .filter(Filter::eq("componentId", Value::Int(cid as i64)))
            .limit(1)
            .build();
        let issues = tx
            .query(&issues_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if !issues.is_empty() {
            return Err(DomainError::FailedPrecondition(format!(
                "component {cid} has issues, move or delete them first"
            ))
            .into());
        }

        // Delete ACLs first (foreign key constraint)
        let delete_acls = Query::table("ComponentAcl")
            .delete()
            .filter(Filter::eq("componentId", Value::Int(cid as i64)))
            .build();
        tx.execute(&delete_acls)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let delete_q = Query::table("Component")
            .delete()
            .filter(Filter::eq("id", Value::Int(cid as i64)))
            .build();
        tx.execute(&delete_q)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(DeleteComponentResponse {}))
    }
}
