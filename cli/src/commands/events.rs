use anyhow::Result;

use crate::proto::event_log_service_client::EventLogServiceClient;
use crate::proto::ListEventsRequest;

pub async fn handle(
    entity_type: String,
    entity_id: i64,
    event_type: String,
    actor: String,
    page_size: i32,
    page_token: String,
    server: &str,
    user: Option<&str>,
) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = EventLogServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = EventLogServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    let response = call!(list_events, ListEventsRequest {
        entity_type,
        entity_id,
        event_type,
        actor,
        since: None,
        until: None,
        page_size,
        page_token,
    })?;
    let resp = response.into_inner();

    if resp.events.is_empty() {
        println!("No events found.");
        return Ok(());
    }

    for event in &resp.events {
        let time = event
            .event_time
            .as_ref()
            .map(|t| {
                chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string())
            })
            .unwrap_or_else(|| "-".to_string());

        println!(
            "[{}] {} #{} {} by {} | {}",
            time,
            event.entity_type,
            event.entity_id,
            event.event_type,
            if event.actor.is_empty() {
                "system"
            } else {
                &event.actor
            },
            event.payload,
        );
    }

    if !resp.next_page_token.is_empty() {
        println!("\nNext page token: {}", resp.next_page_token);
    }

    Ok(())
}
