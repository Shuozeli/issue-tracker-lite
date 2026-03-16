use std::sync::Arc;

use identity::IdentityProvider;
use quiver_driver_core::{Connection, Pool, Transaction, Transactional, Value};
use quiver_query::{Filter, Query};
use tonic::{Request, Response, Status};

use crate::db::DbConn;
use crate::db::row_mapping::{Component, ComponentAcl, HotlistAcl, Issue};
use crate::domain::permissions::{
    self, ComponentPermission, expand_permissions, expanded_access_permission,
};
use crate::domain::types::DomainError;
use crate::proto::acl_service_server::AclService;
use crate::proto::{
    CheckComponentPermissionRequest, CheckComponentPermissionResponse,
    ComponentAclEntry, GetComponentAclRequest, GetComponentAclResponse,
    GetHotlistAclRequest, GetHotlistAclResponse, HotlistAclEntry,
    RemoveComponentAclRequest, RemoveComponentAclResponse, RemoveHotlistAclRequest,
    RemoveHotlistAclResponse, SetComponentAclRequest, SetHotlistAclRequest,
};

pub struct AclServiceImpl {
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

fn component_acl_to_proto(acl: &ComponentAcl) -> ComponentAclEntry {
    let perms_json: Vec<String> =
        serde_json::from_str(&acl.permissions).unwrap_or_default();
    let proto_perms: Vec<i32> = perms_json
        .iter()
        .filter_map(|s| ComponentPermission::from_str(s).ok())
        .map(|p| p.to_proto())
        .collect();

    ComponentAclEntry {
        component_id: acl.component_id as i64,
        identity_type: permissions::identity_type_to_proto(&acl.identity_type),
        identity_value: acl.identity_value.clone(),
        permissions: proto_perms,
        create_time: parse_timestamp(&acl.created_at),
    }
}

fn hotlist_acl_to_proto(acl: &HotlistAcl) -> HotlistAclEntry {
    HotlistAclEntry {
        hotlist_id: acl.hotlist_id as i64,
        identity_type: permissions::identity_type_to_proto(&acl.identity_type),
        identity_value: acl.identity_value.clone(),
        permission: permissions::hotlist_permission_to_proto(&acl.permission),
        create_time: parse_timestamp(&acl.created_at),
    }
}

impl AclServiceImpl {
    async fn validate_component_exists<C: Connection>(
        tx: &C,
        component_id: i64,
    ) -> Result<(), DomainError> {
        let stmt = Query::table("Component")
            .find_first()
            .filter(Filter::eq("id", Value::Int(component_id)))
            .build();
        let row = tx
            .query_optional(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if row.is_none() {
            return Err(DomainError::NotFound(format!(
                "component {component_id} not found"
            )));
        }
        Ok(())
    }

    async fn validate_hotlist_exists<C: Connection>(
        tx: &C,
        hotlist_id: i64,
    ) -> Result<(), DomainError> {
        let stmt = Query::table("Hotlist")
            .find_first()
            .filter(Filter::eq("id", Value::Int(hotlist_id)))
            .build();
        let row = tx
            .query_optional(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        if row.is_none() {
            return Err(DomainError::NotFound(format!(
                "hotlist {hotlist_id} not found"
            )));
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl AclService for AclServiceImpl {
    async fn set_component_acl(
        &self,
        request: Request<SetComponentAclRequest>,
    ) -> Result<Response<ComponentAclEntry>, Status> {
        let req = request.into_inner();

        let identity_type = permissions::identity_type_from_proto(req.identity_type)?;

        if req.identity_value.trim().is_empty() {
            return Err(DomainError::InvalidArgument(
                "identity_value must not be empty".to_string(),
            )
            .into());
        }

        if req.permissions.is_empty() {
            return Err(DomainError::InvalidArgument(
                "permissions must not be empty".to_string(),
            )
            .into());
        }

        // Validate and convert permissions
        let perm_strings: Vec<String> = req
            .permissions
            .iter()
            .map(|&p| ComponentPermission::from_proto(p).map(|cp| cp.as_str().to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let perms_json = serde_json::to_string(&perm_strings)
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_component_exists(&tx, req.component_id).await?;

        // Upsert: check for existing entry
        let find_stmt = Query::table("ComponentAcl")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("componentId", Value::Int(req.component_id)),
                Filter::eq("identityType", Value::Text(identity_type.clone())),
                Filter::eq("identityValue", Value::Text(req.identity_value.clone())),
            ]))
            .build();
        let existing_row = tx
            .query_optional(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let acl = if let Some(row) = existing_row {
            // Update existing
            let existing_acl = ComponentAcl::try_from(&row).map_err(DomainError::from)?;
            let update_stmt = Query::table("ComponentAcl")
                .update()
                .set("permissions", Value::Text(perms_json.clone()))
                .filter(Filter::eq("id", Value::Int(existing_acl.id as i64)))
                .build();
            tx.execute(&update_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            let fetch = Query::table("ComponentAcl")
                .find_first()
                .filter(Filter::eq("id", Value::Int(existing_acl.id as i64)))
                .build();
            let row = tx
                .query_one(&fetch)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            ComponentAcl::try_from(&row).map_err(DomainError::from)?
        } else {
            // Create new
            let now = Value::Text(chrono::Utc::now().to_rfc3339());
            let create_stmt = Query::table("ComponentAcl")
                .create()
                .set("componentId", Value::Int(req.component_id))
                .set("identityType", Value::Text(identity_type.clone()))
                .set("identityValue", Value::Text(req.identity_value.clone()))
                .set("permissions", Value::Text(perms_json.clone()))
                .set("createdAt", now)
                .build();
            tx.execute(&create_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            let fetch = Query::raw(
                "SELECT * FROM ComponentAcl WHERE id = last_insert_rowid()",
            )
            .build();
            let row = tx
                .query_one(&fetch)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            ComponentAcl::try_from(&row).map_err(DomainError::from)?
        };

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(component_acl_to_proto(&acl)))
    }

    async fn get_component_acl(
        &self,
        request: Request<GetComponentAclRequest>,
    ) -> Result<Response<GetComponentAclResponse>, Status> {
        let req = request.into_inner();

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_component_exists(&tx, req.component_id).await?;

        let stmt = Query::table("ComponentAcl")
            .find_many()
            .filter(Filter::eq("componentId", Value::Int(req.component_id)))
            .build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let acls: Vec<ComponentAcl> = rows
            .iter()
            .map(|r| ComponentAcl::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        let entries: Vec<ComponentAclEntry> = acls.iter().map(component_acl_to_proto).collect();

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(GetComponentAclResponse { entries }))
    }

    async fn remove_component_acl(
        &self,
        request: Request<RemoveComponentAclRequest>,
    ) -> Result<Response<RemoveComponentAclResponse>, Status> {
        let req = request.into_inner();

        let identity_type = permissions::identity_type_from_proto(req.identity_type)?;

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_component_exists(&tx, req.component_id).await?;

        // Find the entry
        let find_stmt = Query::table("ComponentAcl")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("componentId", Value::Int(req.component_id)),
                Filter::eq("identityType", Value::Text(identity_type.clone())),
                Filter::eq("identityValue", Value::Text(req.identity_value.clone())),
            ]))
            .build();
        let existing_row = tx
            .query_optional(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = existing_row.ok_or_else(|| {
            DomainError::NotFound(format!(
                "ACL entry not found for {}:{} on component {}",
                identity_type, req.identity_value, req.component_id
            ))
        })?;

        let acl = ComponentAcl::try_from(&row).map_err(DomainError::from)?;

        let del_stmt = Query::table("ComponentAcl")
            .delete()
            .filter(Filter::eq("id", Value::Int(acl.id as i64)))
            .build();
        tx.execute(&del_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(RemoveComponentAclResponse {}))
    }

    async fn set_hotlist_acl(
        &self,
        request: Request<SetHotlistAclRequest>,
    ) -> Result<Response<HotlistAclEntry>, Status> {
        let req = request.into_inner();

        let identity_type = permissions::identity_type_from_proto(req.identity_type)?;
        let permission = permissions::hotlist_permission_from_proto(req.permission)?;

        if req.identity_value.trim().is_empty() {
            return Err(DomainError::InvalidArgument(
                "identity_value must not be empty".to_string(),
            )
            .into());
        }

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_hotlist_exists(&tx, req.hotlist_id).await?;

        // Upsert
        let find_stmt = Query::table("HotlistAcl")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("hotlistId", Value::Int(req.hotlist_id)),
                Filter::eq("identityType", Value::Text(identity_type.clone())),
                Filter::eq("identityValue", Value::Text(req.identity_value.clone())),
            ]))
            .build();
        let existing_row = tx
            .query_optional(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let acl = if let Some(row) = existing_row {
            let existing_acl = HotlistAcl::try_from(&row).map_err(DomainError::from)?;
            let update_stmt = Query::table("HotlistAcl")
                .update()
                .set("permission", Value::Text(permission.clone()))
                .filter(Filter::eq("id", Value::Int(existing_acl.id as i64)))
                .build();
            tx.execute(&update_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            let fetch = Query::table("HotlistAcl")
                .find_first()
                .filter(Filter::eq("id", Value::Int(existing_acl.id as i64)))
                .build();
            let row = tx
                .query_one(&fetch)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            HotlistAcl::try_from(&row).map_err(DomainError::from)?
        } else {
            let now = Value::Text(chrono::Utc::now().to_rfc3339());
            let create_stmt = Query::table("HotlistAcl")
                .create()
                .set("hotlistId", Value::Int(req.hotlist_id))
                .set("identityType", Value::Text(identity_type.clone()))
                .set("identityValue", Value::Text(req.identity_value.clone()))
                .set("permission", Value::Text(permission.clone()))
                .set("createdAt", now)
                .build();
            tx.execute(&create_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            let fetch =
                Query::raw("SELECT * FROM HotlistAcl WHERE id = last_insert_rowid()").build();
            let row = tx
                .query_one(&fetch)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            HotlistAcl::try_from(&row).map_err(DomainError::from)?
        };

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(hotlist_acl_to_proto(&acl)))
    }

    async fn get_hotlist_acl(
        &self,
        request: Request<GetHotlistAclRequest>,
    ) -> Result<Response<GetHotlistAclResponse>, Status> {
        let req = request.into_inner();

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_hotlist_exists(&tx, req.hotlist_id).await?;

        let stmt = Query::table("HotlistAcl")
            .find_many()
            .filter(Filter::eq("hotlistId", Value::Int(req.hotlist_id)))
            .build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let acls: Vec<HotlistAcl> = rows
            .iter()
            .map(|r| HotlistAcl::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        let entries: Vec<HotlistAclEntry> = acls.iter().map(hotlist_acl_to_proto).collect();

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(GetHotlistAclResponse { entries }))
    }

    async fn remove_hotlist_acl(
        &self,
        request: Request<RemoveHotlistAclRequest>,
    ) -> Result<Response<RemoveHotlistAclResponse>, Status> {
        let req = request.into_inner();

        let identity_type = permissions::identity_type_from_proto(req.identity_type)?;

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_hotlist_exists(&tx, req.hotlist_id).await?;

        let find_stmt = Query::table("HotlistAcl")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("hotlistId", Value::Int(req.hotlist_id)),
                Filter::eq("identityType", Value::Text(identity_type.clone())),
                Filter::eq("identityValue", Value::Text(req.identity_value.clone())),
            ]))
            .build();
        let existing_row = tx
            .query_optional(&find_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = existing_row.ok_or_else(|| {
            DomainError::NotFound(format!(
                "ACL entry not found for {}:{} on hotlist {}",
                identity_type, req.identity_value, req.hotlist_id
            ))
        })?;

        let acl = HotlistAcl::try_from(&row).map_err(DomainError::from)?;

        let del_stmt = Query::table("HotlistAcl")
            .delete()
            .filter(Filter::eq("id", Value::Int(acl.id as i64)))
            .build();
        tx.execute(&del_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(RemoveHotlistAclResponse {}))
    }

    async fn check_component_permission(
        &self,
        request: Request<CheckComponentPermissionRequest>,
    ) -> Result<Response<CheckComponentPermissionResponse>, Status> {
        let req = request.into_inner();

        let user_groups = self.identity.resolve_user_groups(&req.user_id).await.unwrap_or_default();

        let mut conn = self.db.acquire().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Self::validate_component_exists(&tx, req.component_id).await?;

        // Step 1: Check component ACL for direct match
        let acl_stmt = Query::table("ComponentAcl")
            .find_many()
            .filter(Filter::eq("componentId", Value::Int(req.component_id)))
            .build();
        let acl_rows = tx
            .query(&acl_stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let acls: Vec<ComponentAcl> = acl_rows
            .iter()
            .map(|r| ComponentAcl::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        // Aggregate permissions from ALL matching ACL entries
        let mut all_perms = std::collections::HashSet::new();
        let mut has_acl_match = false;

        for acl in &acls {
            let matches = match acl.identity_type.as_str() {
                "USER" => acl.identity_value == req.user_id,
                "PUBLIC" => true,
                "GROUP" => user_groups.contains(&acl.identity_value),
                _ => false,
            };
            if matches {
                has_acl_match = true;
                let perm_strings: Vec<String> =
                    serde_json::from_str(&acl.permissions).unwrap_or_default();
                let perms: Vec<ComponentPermission> = perm_strings
                    .iter()
                    .filter_map(|s| ComponentPermission::from_str(s).ok())
                    .collect();
                let expanded = expand_permissions(&perms);
                all_perms.extend(expanded);
            }
        }

        if has_acl_match {
            let proto_perms: Vec<i32> = all_perms.iter().map(|p| p.to_proto()).collect();
            tx.commit()
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            return Ok(Response::new(CheckComponentPermissionResponse {
                permissions: proto_perms,
                grant_source: "ACL".to_string(),
            }));
        }

        // Step 2: Check expanded access if an issue_id is provided
        if let Some(issue_id) = req.issue_id {
            let comp_stmt = Query::table("Component")
                .find_first()
                .filter(Filter::eq("id", Value::Int(req.component_id)))
                .build();
            let comp_row = tx
                .query_optional(&comp_stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            if let Some(comp_row) = comp_row {
                let comp = Component::try_from(&comp_row).map_err(DomainError::from)?;
                if comp.expanded_access_enabled {
                    let issue_stmt = Query::table("Issue")
                        .find_first()
                        .filter(Filter::eq("id", Value::Int(issue_id as i64)))
                        .build();
                    let issue_row = tx
                        .query_optional(&issue_stmt)
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;

                    if let Some(issue_row) = issue_row {
                        let issue = Issue::try_from(&issue_row).map_err(DomainError::from)?;
                        if let Some(base_perm) = expanded_access_permission(
                            &req.user_id,
                            &issue.assignee,
                            &issue.verifier,
                            &issue.reporter,
                        ) {
                            let expanded = expand_permissions(&[base_perm]);
                            let proto_perms: Vec<i32> =
                                expanded.iter().map(|p| p.to_proto()).collect();
                            tx.commit()
                                .await
                                .map_err(|e| DomainError::Internal(e.to_string()))?;
                            return Ok(Response::new(CheckComponentPermissionResponse {
                                permissions: proto_perms,
                                grant_source: "EXPANDED_ACCESS".to_string(),
                            }));
                        }
                    }
                }
            }
        }

        // Step 3: Deny
        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(Response::new(CheckComponentPermissionResponse {
            permissions: vec![],
            grant_source: "DENIED".to_string(),
        }))
    }
}
