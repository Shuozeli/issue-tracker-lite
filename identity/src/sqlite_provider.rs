use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use quiver_driver_core::{Connection, Pool, Transaction, Transactional, Value};
use quiver_driver_sqlite::SqlitePool;
use quiver_query::{Filter, Order, Query};

use crate::error::IdentityError;
use crate::provider::IdentityProvider;
use crate::row_mapping::{group_from_row, group_member_from_row};
use crate::types::{Group, GroupMember, MemberRole, MemberType};
use crate::validation::{validate_group_name, MAX_NESTING_DEPTH};

/// Database connection pool type for the identity provider.
pub type IdentityDbConn = Arc<SqlitePool>;

pub struct SqliteIdentityProvider {
    db: IdentityDbConn,
}

impl SqliteIdentityProvider {
    pub fn new(db: IdentityDbConn) -> Self {
        Self { db }
    }

    /// Look up a group by name within a transaction, returning its row.
    async fn get_group_by_name<C: Connection>(
        conn: &C,
        name: &str,
    ) -> Result<Group, IdentityError> {
        let q = Query::table("Group")
            .find_first()
            .filter(Filter::eq("name", Value::Text(name.to_string())))
            .build();
        let row = conn
            .query_optional(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        match row {
            Some(r) => group_from_row(&r),
            None => Err(IdentityError::NotFound(format!("group '{name}'"))),
        }
    }

    /// Check for cycles when adding a group member of type GROUP.
    /// Walks upward from `parent_group_name` to see if `child_group_name` is an ancestor.
    async fn check_cycle<C: Connection>(
        conn: &C,
        parent_group_name: &str,
        child_group_name: &str,
    ) -> Result<(), IdentityError> {
        // BFS upward: find all groups that contain parent_group_name as a member
        let mut visited = HashSet::new();
        visited.insert(parent_group_name.to_string());
        let mut queue = VecDeque::new();
        queue.push_back(parent_group_name.to_string());

        while let Some(current) = queue.pop_front() {
            // Find all groups where current is a GROUP member
            let q = Query::table("GroupMember")
                .find_many()
                .filter(Filter::and(vec![
                    Filter::eq("memberType", Value::Text("GROUP".to_string())),
                    Filter::eq("memberValue", Value::Text(current.clone())),
                ]))
                .build();
            let rows = conn
                .query(&q)
                .await
                .map_err(|e| IdentityError::Internal(e.to_string()))?;

            for row in &rows {
                let member = group_member_from_row(row)?;
                // Resolve the group_id to a group name
                let gq = Query::table("Group")
                    .find_first()
                    .filter(Filter::eq("id", Value::Int(member.group_id as i64)))
                    .build();
                if let Some(grow) = conn
                    .query_optional(&gq)
                    .await
                    .map_err(|e| IdentityError::Internal(e.to_string()))?
                {
                    let g = group_from_row(&grow)?;
                    if g.name == child_group_name {
                        return Err(IdentityError::InvalidArgument(format!(
                            "adding group '{child_group_name}' to '{parent_group_name}' would create a cycle"
                        )));
                    }
                    if visited.insert(g.name.clone()) {
                        queue.push_back(g.name);
                    }
                }
            }
        }

        Ok(())
    }

    /// Log an event to the EventLog table.
    async fn log_event<C: Connection>(
        conn: &C,
        event_type: &str,
        actor: &str,
        entity_type: &str,
        entity_id: i32,
        payload: &str,
    ) -> Result<(), IdentityError> {
        let now = chrono::Utc::now().to_rfc3339();
        let q = Query::table("EventLog")
            .create()
            .set("eventTime", Value::Text(now))
            .set("eventType", Value::Text(event_type.to_string()))
            .set("actor", Value::Text(actor.to_string()))
            .set("entityType", Value::Text(entity_type.to_string()))
            .set("entityId", Value::Int(entity_id as i64))
            .set("payload", Value::Text(payload.to_string()))
            .build();
        conn.execute(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Resolve groups for a user within an existing connection (no mutex lock).
    async fn resolve_user_groups_inner<C: Connection>(
        conn: &C,
        user_id: &str,
    ) -> Result<Vec<String>, IdentityError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Level 0: find direct group memberships for this user
        let q = Query::table("GroupMember")
            .find_many()
            .filter(Filter::and(vec![
                Filter::eq("memberType", Value::Text("USER".to_string())),
                Filter::eq("memberValue", Value::Text(user_id.to_string())),
            ]))
            .build();
        let rows = conn
            .query(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        for row in &rows {
            let member = group_member_from_row(row)?;
            // Resolve group_id to group name
            let gq = Query::table("Group")
                .find_first()
                .filter(Filter::eq("id", Value::Int(member.group_id as i64)))
                .build();
            if let Some(grow) = conn
                .query_optional(&gq)
                .await
                .map_err(|e| IdentityError::Internal(e.to_string()))?
            {
                let g = group_from_row(&grow)?;
                if visited.insert(g.name.clone()) {
                    result.push(g.name.clone());
                    queue.push_back((g.name, 1usize));
                }
            }
        }

        // BFS for nested groups
        while let Some((group_name, depth)) = queue.pop_front() {
            if depth >= MAX_NESTING_DEPTH {
                continue;
            }

            // Find groups where this group is a GROUP member
            let q = Query::table("GroupMember")
                .find_many()
                .filter(Filter::and(vec![
                    Filter::eq("memberType", Value::Text("GROUP".to_string())),
                    Filter::eq("memberValue", Value::Text(group_name.clone())),
                ]))
                .build();
            let rows = conn
                .query(&q)
                .await
                .map_err(|e| IdentityError::Internal(e.to_string()))?;

            for row in &rows {
                let member = group_member_from_row(row)?;
                let gq = Query::table("Group")
                    .find_first()
                    .filter(Filter::eq("id", Value::Int(member.group_id as i64)))
                    .build();
                if let Some(grow) = conn
                    .query_optional(&gq)
                    .await
                    .map_err(|e| IdentityError::Internal(e.to_string()))?
                {
                    let g = group_from_row(&grow)?;
                    if visited.insert(g.name.clone()) {
                        result.push(g.name.clone());
                        queue.push_back((g.name, depth + 1));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Fetch a member row by composite key within a transaction.
    async fn get_member_row<C: Connection>(
        conn: &C,
        group_id: i32,
        member_type: MemberType,
        member_value: &str,
    ) -> Result<Option<GroupMember>, IdentityError> {
        let q = Query::table("GroupMember")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("groupId", Value::Int(group_id as i64)),
                Filter::eq(
                    "memberType",
                    Value::Text(member_type.as_str().to_string()),
                ),
                Filter::eq(
                    "memberValue",
                    Value::Text(member_value.to_string()),
                ),
            ]))
            .build();
        let row = conn
            .query_optional(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        match row {
            Some(r) => Ok(Some(group_member_from_row(&r)?)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl IdentityProvider for SqliteIdentityProvider {
    async fn resolve_user_groups(&self, user_id: &str) -> Result<Vec<String>, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let result = Self::resolve_user_groups_inner(&tx, user_id).await?;
        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        Ok(result)
    }

    async fn is_member(&self, user_id: &str, group_name: &str) -> Result<bool, IdentityError> {
        let groups = self.resolve_user_groups(user_id).await?;
        Ok(groups.contains(&group_name.to_string()))
    }

    async fn create_group(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        creator: &str,
    ) -> Result<Group, IdentityError> {
        validate_group_name(name)?;

        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        // Check uniqueness
        let check_q = Query::table("Group")
            .find_first()
            .filter(Filter::eq("name", Value::Text(name.to_string())))
            .build();
        if tx
            .query_optional(&check_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(IdentityError::AlreadyExists(format!("group '{name}'")));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let q = Query::table("Group")
            .create()
            .set("name", Value::Text(name.to_string()))
            .set("displayName", Value::Text(display_name.to_string()))
            .set("description", Value::Text(description.to_string()))
            .set("creator", Value::Text(creator.to_string()))
            .set("createdAt", Value::Text(now.clone()))
            .set("updatedAt", Value::Text(now))
            .build();
        tx.execute(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        // Fetch created group (no RETURNING in SQLite via quiver)
        let fetch_q = Query::table("Group")
            .find_first()
            .filter(Filter::eq("name", Value::Text(name.to_string())))
            .build();
        let row = tx
            .query_optional(&fetch_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?
            .ok_or_else(|| IdentityError::Internal("failed to fetch created group".to_string()))?;
        let group = group_from_row(&row)?;

        Self::log_event(
            &tx,
            "GROUP_CREATED",
            creator,
            "Group",
            group.id,
            &serde_json::json!({"name": name}).to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(group)
    }

    async fn get_group(&self, name: &str) -> Result<Group, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let group = Self::get_group_by_name(&tx, name).await?;
        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        Ok(group)
    }

    async fn list_groups(
        &self,
        page_size: i32,
        page_token: &str,
    ) -> Result<(Vec<Group>, String), IdentityError> {
        let limit = page_size.clamp(1, 100) as u64;

        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let mut filters = Vec::new();
        if !page_token.is_empty() {
            let cursor: i32 = page_token
                .parse()
                .map_err(|_| IdentityError::InvalidArgument("invalid page token".to_string()))?;
            filters.push(Filter::gt("id", Value::Int(cursor as i64)));
        }

        let q = if filters.is_empty() {
            Query::table("Group")
                .find_many()
                .order_by("id", Order::Asc)
                .limit(limit + 1)
                .build()
        } else {
            Query::table("Group")
                .find_many()
                .filter(Filter::and(filters))
                .order_by("id", Order::Asc)
                .limit(limit + 1)
                .build()
        };

        let rows = tx
            .query(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let mut groups: Vec<Group> = rows
            .iter()
            .map(group_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        let next_token = if groups.len() as u64 > limit {
            groups.truncate(limit as usize);
            groups.last().map(|g| g.id.to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        Ok((groups, next_token))
    }

    async fn update_group(
        &self,
        name: &str,
        display_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Group, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let existing = Self::get_group_by_name(&tx, name).await?;

        let now = chrono::Utc::now().to_rfc3339();
        let mut update = Query::table("Group").update();
        update = update.set("updatedAt", Value::Text(now));
        if let Some(dn) = display_name {
            update = update.set("displayName", Value::Text(dn.to_string()));
        }
        if let Some(desc) = description {
            update = update.set("description", Value::Text(desc.to_string()));
        }
        let q = update
            .filter(Filter::eq("id", Value::Int(existing.id as i64)))
            .build();
        tx.execute(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, name).await?;

        Self::log_event(
            &tx,
            "GROUP_UPDATED",
            "",
            "Group",
            group.id,
            &serde_json::json!({"name": name}).to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(group)
    }

    async fn delete_group(&self, name: &str) -> Result<(), IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, name).await?;

        // Check: group is not a member of another group
        let member_q = Query::table("GroupMember")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("memberType", Value::Text("GROUP".to_string())),
                Filter::eq("memberValue", Value::Text(name.to_string())),
            ]))
            .build();
        if tx
            .query_optional(&member_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(IdentityError::FailedPrecondition(
                "group is a member of another group; remove it first".to_string(),
            ));
        }

        // Check: group is not referenced in ComponentAcl
        let cacl_q = Query::table("ComponentAcl")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("identityType", Value::Text("GROUP".to_string())),
                Filter::eq("identityValue", Value::Text(name.to_string())),
            ]))
            .build();
        if tx
            .query_optional(&cacl_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(IdentityError::FailedPrecondition(
                "group is referenced in a component ACL; remove it first".to_string(),
            ));
        }

        // Check: group is not referenced in HotlistAcl
        let hacl_q = Query::table("HotlistAcl")
            .find_first()
            .filter(Filter::and(vec![
                Filter::eq("identityType", Value::Text("GROUP".to_string())),
                Filter::eq("identityValue", Value::Text(name.to_string())),
            ]))
            .build();
        if tx
            .query_optional(&hacl_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(IdentityError::FailedPrecondition(
                "group is referenced in a hotlist ACL; remove it first".to_string(),
            ));
        }

        // Delete all members of this group first
        let del_members_q = Query::table("GroupMember")
            .delete()
            .filter(Filter::eq("groupId", Value::Int(group.id as i64)))
            .build();
        tx.execute(&del_members_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        // Delete the group
        let del_q = Query::table("Group")
            .delete()
            .filter(Filter::eq("id", Value::Int(group.id as i64)))
            .build();
        tx.execute(&del_q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Self::log_event(
            &tx,
            "GROUP_DELETED",
            "",
            "Group",
            group.id,
            &serde_json::json!({"name": name}).to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn add_member(
        &self,
        group_name: &str,
        member_type: MemberType,
        member_value: &str,
        role: MemberRole,
        added_by: &str,
    ) -> Result<GroupMember, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, group_name).await?;

        // If adding a group, validate it exists and check for cycles
        if member_type == MemberType::Group {
            // Self-loop check
            if member_value == group_name {
                return Err(IdentityError::InvalidArgument(format!(
                    "adding group '{member_value}' to '{group_name}' would create a cycle"
                )));
            }
            Self::get_group_by_name(&tx, member_value).await?;
            Self::check_cycle(&tx, group_name, member_value).await?;
        }

        // Check for duplicate
        if Self::get_member_row(&tx, group.id, member_type, member_value)
            .await?
            .is_some()
        {
            return Err(IdentityError::AlreadyExists(format!(
                "member '{member_value}' already in group '{group_name}'"
            )));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let q = Query::table("GroupMember")
            .create()
            .set("groupId", Value::Int(group.id as i64))
            .set(
                "memberType",
                Value::Text(member_type.as_str().to_string()),
            )
            .set("memberValue", Value::Text(member_value.to_string()))
            .set("role", Value::Text(role.as_str().to_string()))
            .set("addedBy", Value::Text(added_by.to_string()))
            .set("createdAt", Value::Text(now))
            .build();
        tx.execute(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let member = Self::get_member_row(&tx, group.id, member_type, member_value)
            .await?
            .ok_or_else(|| {
                IdentityError::Internal("failed to fetch created member".to_string())
            })?;

        Self::log_event(
            &tx,
            "GROUP_MEMBER_ADDED",
            added_by,
            "Group",
            group.id,
            &serde_json::json!({
                "group": group_name,
                "memberType": member_type.as_str(),
                "memberValue": member_value,
                "role": role.as_str()
            })
            .to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(member)
    }

    async fn remove_member(
        &self,
        group_name: &str,
        member_type: MemberType,
        member_value: &str,
    ) -> Result<(), IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, group_name).await?;

        let existing = Self::get_member_row(&tx, group.id, member_type, member_value)
            .await?
            .ok_or_else(|| {
                IdentityError::NotFound(format!(
                    "member '{member_value}' not in group '{group_name}'"
                ))
            })?;

        let q = Query::table("GroupMember")
            .delete()
            .filter(Filter::eq("id", Value::Int(existing.id as i64)))
            .build();
        tx.execute(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Self::log_event(
            &tx,
            "GROUP_MEMBER_REMOVED",
            "",
            "Group",
            group.id,
            &serde_json::json!({
                "group": group_name,
                "memberType": member_type.as_str(),
                "memberValue": member_value
            })
            .to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn list_members(&self, group_name: &str) -> Result<Vec<GroupMember>, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, group_name).await?;

        let q = Query::table("GroupMember")
            .find_many()
            .filter(Filter::eq("groupId", Value::Int(group.id as i64)))
            .order_by("id", Order::Asc)
            .build();
        let rows = tx
            .query(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let members: Vec<GroupMember> = rows
            .iter()
            .map(group_member_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(members)
    }

    async fn update_member_role(
        &self,
        group_name: &str,
        member_type: MemberType,
        member_value: &str,
        new_role: MemberRole,
    ) -> Result<GroupMember, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, group_name).await?;

        let existing = Self::get_member_row(&tx, group.id, member_type, member_value)
            .await?
            .ok_or_else(|| {
                IdentityError::NotFound(format!(
                    "member '{member_value}' not in group '{group_name}'"
                ))
            })?;

        let q = Query::table("GroupMember")
            .update()
            .set("role", Value::Text(new_role.as_str().to_string()))
            .filter(Filter::eq("id", Value::Int(existing.id as i64)))
            .build();
        tx.execute(&q)
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let updated = Self::get_member_row(&tx, group.id, member_type, member_value)
            .await?
            .ok_or_else(|| {
                IdentityError::Internal("failed to fetch updated member".to_string())
            })?;

        Self::log_event(
            &tx,
            "GROUP_MEMBER_ROLE_UPDATED",
            "",
            "Group",
            group.id,
            &serde_json::json!({
                "group": group_name,
                "memberValue": member_value,
                "newRole": new_role.as_str()
            })
            .to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(updated)
    }

    async fn batch_add_members(
        &self,
        group_name: &str,
        members: &[(MemberType, String, MemberRole)],
        added_by: &str,
    ) -> Result<Vec<GroupMember>, IdentityError> {
        let mut conn = self.db.acquire().await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        let group = Self::get_group_by_name(&tx, group_name).await?;
        let now = chrono::Utc::now().to_rfc3339();

        let mut result = Vec::new();

        for (member_type, member_value, role) in members {
            // If adding a group, validate and check cycles
            if *member_type == MemberType::Group {
                if member_value == group_name {
                    return Err(IdentityError::InvalidArgument(format!(
                        "adding group '{member_value}' to '{group_name}' would create a cycle"
                    )));
                }
                Self::get_group_by_name(&tx, member_value).await?;
                Self::check_cycle(&tx, group_name, member_value).await?;
            }

            // Skip duplicates silently in batch mode
            if Self::get_member_row(&tx, group.id, *member_type, member_value)
                .await?
                .is_some()
            {
                let existing =
                    Self::get_member_row(&tx, group.id, *member_type, member_value)
                        .await?
                        .unwrap();
                result.push(existing);
                continue;
            }

            let q = Query::table("GroupMember")
                .create()
                .set("groupId", Value::Int(group.id as i64))
                .set(
                    "memberType",
                    Value::Text(member_type.as_str().to_string()),
                )
                .set("memberValue", Value::Text(member_value.clone()))
                .set("role", Value::Text(role.as_str().to_string()))
                .set("addedBy", Value::Text(added_by.to_string()))
                .set("createdAt", Value::Text(now.clone()))
                .build();
            tx.execute(&q)
                .await
                .map_err(|e| IdentityError::Internal(e.to_string()))?;

            let member = Self::get_member_row(&tx, group.id, *member_type, member_value)
                .await?
                .ok_or_else(|| {
                    IdentityError::Internal("failed to fetch created member".to_string())
                })?;
            result.push(member);
        }

        Self::log_event(
            &tx,
            "GROUP_MEMBERS_BATCH_ADDED",
            added_by,
            "Group",
            group.id,
            &serde_json::json!({
                "group": group_name,
                "count": members.len()
            })
            .to_string(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| IdentityError::Internal(e.to_string()))?;

        Ok(result)
    }
}
