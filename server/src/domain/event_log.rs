/// Shared event logging helper.
///
/// All services should use this function instead of defining their own
/// `log_event` copies.
use quiver_driver_core::{Connection, Value};
use quiver_query::Query;

use crate::domain::types::DomainError;

/// Insert an event-log row.  `actor` should be the real user ID when
/// available; fall back to `"system"` only for truly system-initiated actions.
pub async fn log_event<C: Connection>(
    conn: &C,
    event_type: &str,
    actor: &str,
    entity_type: &str,
    entity_id: i32,
    payload: &serde_json::Value,
) -> Result<(), DomainError> {
    let stmt = Query::table("EventLog")
        .create()
        .set("eventTime", Value::Text(chrono::Utc::now().to_rfc3339()))
        .set("eventType", Value::Text(event_type.to_string()))
        .set("actor", Value::Text(actor.to_string()))
        .set("entityType", Value::Text(entity_type.to_string()))
        .set("entityId", Value::Int(entity_id as i64))
        .set("payload", Value::Text(payload.to_string()))
        .build();
    conn.execute(&stmt)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(())
}
