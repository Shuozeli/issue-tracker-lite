use quiver_driver_core::{Connection, Pool, Statement, Transaction, Transactional, Value};
use tonic::{Request, Response, Status};

use crate::db::row_mapping::EventLog;
use crate::db::DbConn;
use crate::domain::types::DomainError;

use crate::proto::event_log_service_server::EventLogService;
use crate::proto::{Event as ProtoEvent, ListEventsRequest, ListEventsResponse};

pub struct EventLogServiceImpl {
    pub db: DbConn,
}

fn event_log_to_proto(e: &crate::db::row_mapping::EventLog) -> ProtoEvent {
    ProtoEvent {
        event_id: e.id as i64,
        event_time: parse_timestamp(&e.event_time),
        event_type: e.event_type.clone(),
        actor: e.actor.clone(),
        entity_type: e.entity_type.clone(),
        entity_id: e.entity_id as i64,
        payload: e.payload.clone(),
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

#[tonic::async_trait]
impl EventLogService for EventLogServiceImpl {
    async fn list_events(
        &self,
        request: Request<ListEventsRequest>,
    ) -> Result<Response<ListEventsResponse>, Status> {
        let req = request.into_inner();

        let page_size = if req.page_size > 0 {
            req.page_size.min(100)
        } else {
            50
        };

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Value> = Vec::new();

        if !req.entity_type.is_empty() {
            conditions.push("entityType = ?".to_string());
            params.push(Value::Text(req.entity_type.clone()));
        }

        if req.entity_id != 0 {
            conditions.push("entityId = ?".to_string());
            params.push(Value::Int(req.entity_id));
        }

        if !req.event_type.is_empty() {
            conditions.push("eventType = ?".to_string());
            params.push(Value::Text(req.event_type.clone()));
        }

        if !req.actor.is_empty() {
            conditions.push("actor = ?".to_string());
            params.push(Value::Text(req.actor.clone()));
        }

        if let Some(since) = req.since {
            let dt = chrono::DateTime::from_timestamp(since.seconds, since.nanos as u32)
                .unwrap_or_default();
            conditions.push("eventTime >= ?".to_string());
            params.push(Value::Text(dt.to_rfc3339()));
        }

        if let Some(until) = req.until {
            let dt = chrono::DateTime::from_timestamp(until.seconds, until.nanos as u32)
                .unwrap_or_default();
            conditions.push("eventTime <= ?".to_string());
            params.push(Value::Text(dt.to_rfc3339()));
        }

        // Cursor-based pagination: DESC order on id, so cursor means id < cursor
        if !req.page_token.is_empty() {
            let cursor_id = req
                .page_token
                .parse::<i64>()
                .map_err(|_| DomainError::InvalidArgument("invalid page_token".to_string()))?;
            conditions.push("id < ?".to_string());
            params.push(Value::Int(cursor_id));
        }

        let where_sql = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        let sql = format!(
            "SELECT * FROM EventLog WHERE {} ORDER BY id DESC LIMIT {}",
            where_sql,
            page_size + 1
        );

        let built = Statement::new(sql, params);

        let mut conn = self
            .db
            .acquire()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let tx = conn
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rows = tx
            .query(&built)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let items: Vec<_> = rows
            .iter()
            .map(|r| EventLog::try_from(r).map_err(DomainError::from))
            .collect::<Result<Vec<_>, _>>()?;

        let has_more = items.len() > page_size as usize;
        let items = if has_more {
            &items[..page_size as usize]
        } else {
            &items
        };

        let events: Vec<ProtoEvent> = items.iter().map(event_log_to_proto).collect();

        let next_page_token = if has_more {
            items.last().map(|e| e.id.to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Response::new(ListEventsResponse {
            events,
            next_page_token,
        }))
    }
}
