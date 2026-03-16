use std::sync::Arc;

use identity::IdentityProvider;
use quiver_driver_core::{Connection, Pool, Statement, Transaction, Transactional, Value};
use quiver_query::{Filter, Query};
use tonic::{Request, Response, Status};

use crate::db::DbConn;
use crate::db::row_mapping::{Component, HotlistIssue, Issue};
use crate::domain::permissions;
use crate::domain::query_parser::{self, FilterField, FilterOp};
use crate::domain::types::DomainError;
use crate::service::issue_service::issue_to_proto;

use crate::proto::search_service_server::SearchService;
use crate::proto::{Issue as ProtoIssue, SearchIssuesRequest, SearchIssuesResponse};

pub struct SearchServiceImpl {
    pub db: DbConn,
    pub identity: Arc<dyn IdentityProvider>,
}

impl SearchServiceImpl {
    /// Get all descendant component IDs for a given component (recursive).
    async fn get_descendant_component_ids<C: Connection>(
        tx: &C,
        root_id: i32,
    ) -> Result<Vec<i32>, DomainError> {
        let mut result = vec![root_id];
        let mut queue = vec![root_id];

        while let Some(parent_id) = queue.pop() {
            let stmt = Query::table("Component")
                .find_many()
                .filter(Filter::eq("parentId", Value::Int(parent_id as i64)))
                .build();
            let rows = tx
                .query(&stmt)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            for row in rows {
                let child = Component::try_from(&row)?;
                result.push(child.id);
                queue.push(child.id);
            }
        }

        Ok(result)
    }

    /// Get issue IDs that belong to a hotlist.
    async fn get_hotlist_issue_ids<C: Connection>(
        tx: &C,
        hotlist_id: i32,
    ) -> Result<Vec<i32>, DomainError> {
        let stmt = Query::table("HotlistIssue")
            .find_many()
            .filter(Filter::eq("hotlistId", Value::Int(hotlist_id as i64)))
            .build();
        let rows = tx
            .query(&stmt)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let ids = rows
            .iter()
            .map(HotlistIssue::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids.iter().map(|hi| hi.issue_id).collect())
    }

    /// Build a raw SQL WHERE clause and params from the parsed query.
    /// Returns (conditions_sql, params) where conditions_sql is an AND-joined
    /// list of SQL condition fragments (each with a single `?` placeholder).
    async fn build_where_parts<C: Connection>(
        tx: &C,
        parsed: &query_parser::ParsedQuery,
    ) -> Result<(Vec<String>, Vec<Value>), DomainError> {
        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Value> = Vec::new();

        for filter in &parsed.filters {
            match (&filter.field, &filter.op) {
                (FilterField::Status, FilterOp::Equals) => {
                    let statuses = query_parser::resolve_status_value(&filter.value);
                    if statuses.len() == 1 {
                        conditions.push("status = ?".to_string());
                        params.push(Value::Text(statuses[0].to_string()));
                    } else {
                        let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        conditions.push(format!("status IN ({})", placeholders));
                        for s in statuses {
                            params.push(Value::Text(s.to_string()));
                        }
                    }
                }
                (FilterField::Status, FilterOp::NotEquals) => {
                    let statuses = query_parser::resolve_status_value(&filter.value);
                    if statuses.len() == 1 {
                        conditions.push("status != ?".to_string());
                        params.push(Value::Text(statuses[0].to_string()));
                    } else {
                        let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        conditions.push(format!("status NOT IN ({})", placeholders));
                        for s in statuses {
                            params.push(Value::Text(s.to_string()));
                        }
                    }
                }
                (FilterField::Priority, FilterOp::Equals) => {
                    conditions.push("priority = ?".to_string());
                    params.push(Value::Text(filter.value.to_uppercase()));
                }
                (FilterField::Priority, FilterOp::NotEquals) => {
                    conditions.push("priority != ?".to_string());
                    params.push(Value::Text(filter.value.to_uppercase()));
                }
                (FilterField::Severity, FilterOp::Equals) => {
                    conditions.push("severity = ?".to_string());
                    params.push(Value::Text(filter.value.to_uppercase()));
                }
                (FilterField::Severity, FilterOp::NotEquals) => {
                    conditions.push("severity != ?".to_string());
                    params.push(Value::Text(filter.value.to_uppercase()));
                }
                (FilterField::IssueType, FilterOp::Equals) => {
                    conditions.push("issueType = ?".to_string());
                    params.push(Value::Text(filter.value.to_uppercase()));
                }
                (FilterField::IssueType, FilterOp::NotEquals) => {
                    conditions.push("issueType != ?".to_string());
                    params.push(Value::Text(filter.value.to_uppercase()));
                }
                (FilterField::Assignee, FilterOp::Equals) => {
                    match filter.value.to_lowercase().as_str() {
                        "none" => {
                            conditions.push("assignee = ''".to_string());
                        }
                        "any" => {
                            conditions.push("assignee != ''".to_string());
                        }
                        _ => {
                            conditions.push("assignee = ?".to_string());
                            params.push(Value::Text(filter.value.clone()));
                        }
                    }
                }
                (FilterField::Assignee, FilterOp::NotEquals) => {
                    conditions.push("assignee != ?".to_string());
                    params.push(Value::Text(filter.value.clone()));
                }
                (FilterField::Reporter, FilterOp::Equals) => {
                    match filter.value.to_lowercase().as_str() {
                        "none" => {
                            conditions.push("reporter = ''".to_string());
                        }
                        "any" => {
                            conditions.push("reporter != ''".to_string());
                        }
                        _ => {
                            conditions.push("reporter = ?".to_string());
                            params.push(Value::Text(filter.value.clone()));
                        }
                    }
                }
                (FilterField::Reporter, FilterOp::NotEquals) => {
                    conditions.push("reporter != ?".to_string());
                    params.push(Value::Text(filter.value.clone()));
                }
                (FilterField::ComponentId, FilterOp::Equals) => {
                    if let Ok(id) = filter.value.parse::<i32>() {
                        conditions.push("componentId = ?".to_string());
                        params.push(Value::Int(id as i64));
                    }
                }
                (FilterField::ComponentId, FilterOp::NotEquals) => {
                    if let Ok(id) = filter.value.parse::<i32>() {
                        conditions.push("componentId != ?".to_string());
                        params.push(Value::Int(id as i64));
                    }
                }
                (FilterField::ComponentIdRecursive, FilterOp::Equals) => {
                    if let Ok(id) = filter.value.parse::<i32>() {
                        let ids = Self::get_descendant_component_ids(tx, id).await?;
                        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        conditions.push(format!("componentId IN ({})", placeholders));
                        for cid in ids {
                            params.push(Value::Int(cid as i64));
                        }
                    }
                }
                (FilterField::ComponentIdRecursive, FilterOp::NotEquals) => {
                    if let Ok(id) = filter.value.parse::<i32>() {
                        let ids = Self::get_descendant_component_ids(tx, id).await?;
                        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        conditions.push(format!("componentId NOT IN ({})", placeholders));
                        for cid in ids {
                            params.push(Value::Int(cid as i64));
                        }
                    }
                }
                (FilterField::HotlistId, FilterOp::Equals) => {
                    if let Ok(id) = filter.value.parse::<i32>() {
                        let issue_ids = Self::get_hotlist_issue_ids(tx, id).await?;
                        let placeholders = issue_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        conditions.push(format!("id IN ({})", placeholders));
                        for iid in issue_ids {
                            params.push(Value::Int(iid as i64));
                        }
                    }
                }
                (FilterField::HotlistId, FilterOp::NotEquals) => {
                    if let Ok(id) = filter.value.parse::<i32>() {
                        let issue_ids = Self::get_hotlist_issue_ids(tx, id).await?;
                        let placeholders = issue_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        conditions.push(format!("id NOT IN ({})", placeholders));
                        for iid in issue_ids {
                            params.push(Value::Int(iid as i64));
                        }
                    }
                }
            }
        }

        // Keyword search: LIKE on title and description (escape LIKE wildcards)
        for keyword in &parsed.keywords {
            conditions.push("(title LIKE ? ESCAPE '\\' OR description LIKE ? ESCAPE '\\')".to_string());
            let escaped = keyword
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let pattern = Value::Text(format!("%{escaped}%"));
            params.push(pattern.clone());
            params.push(pattern);
        }

        Ok((conditions, params))
    }
}

#[tonic::async_trait]
impl SearchService for SearchServiceImpl {
    async fn search_issues(
        &self,
        request: Request<SearchIssuesRequest>,
    ) -> Result<Response<SearchIssuesResponse>, Status> {
        let user_id = permissions::extract_user_id(&request);
        let req = request.into_inner();

        let parsed = query_parser::parse_query(&req.query);

        let page_size = if req.page_size > 0 {
            req.page_size.min(100)
        } else {
            50
        };

        // Determine order
        let order_field = match req.order_by.as_str() {
            "created" => "createdAt",
            "priority" => "priority",
            "status" => "status",
            _ => "modifiedAt", // default: modified
        };
        let order_dir = match req.order_direction.as_str() {
            "asc" => "ASC",
            _ => "DESC",
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

        // Filter to only accessible components
        let accessible_comp_ids = permissions::get_accessible_component_ids(
            &tx,
            user_id.as_deref(),
            permissions::ComponentPermission::ViewIssues,
            &user_groups,
        )
        .await?;

        if accessible_comp_ids.is_empty() {
            tx.commit()
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            return Ok(Response::new(SearchIssuesResponse {
                issues: vec![],
                next_page_token: String::new(),
                total_count: 0,
            }));
        }

        let (mut conditions, mut params) = Self::build_where_parts(&tx, &parsed).await?;

        // Restrict to accessible components
        let placeholders = accessible_comp_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        conditions.push(format!("componentId IN ({})", placeholders));
        for cid in &accessible_comp_ids {
            params.push(Value::Int(*cid));
        }

        // Cursor-based pagination: filter by id relative to cursor
        if !req.page_token.is_empty() {
            let cursor_id = req.page_token.parse::<i64>().map_err(|_| {
                DomainError::InvalidArgument("invalid page_token".to_string())
            })?;
            if order_dir == "DESC" {
                conditions.push("id < ?".to_string());
            } else {
                conditions.push("id > ?".to_string());
            }
            params.push(Value::Int(cursor_id));
        }

        let where_sql = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        let sql = format!(
            "SELECT * FROM Issue WHERE {} ORDER BY {} {} LIMIT {}",
            where_sql,
            order_field,
            order_dir,
            page_size + 1
        );

        let built = Statement::new(sql, params);

        let rows = tx
            .query(&built)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let items: Vec<_> = rows
            .iter()
            .map(|r| Issue::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        let has_more = items.len() > page_size as usize;
        let items = if has_more {
            &items[..page_size as usize]
        } else {
            &items
        };

        let issues: Vec<ProtoIssue> = items.iter().map(issue_to_proto).collect();

        let next_page_token = if has_more {
            items.last().map(|i| i.id.to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        // Approximate total count: if first page and not full, we know total
        let total_count = if req.page_token.is_empty() && !has_more {
            issues.len() as i32
        } else {
            -1 // unknown
        };

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(SearchIssuesResponse {
            issues,
            next_page_token,
            total_count,
        }))
    }
}
