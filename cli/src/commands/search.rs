use anyhow::Result;

use crate::output;
use crate::proto::search_service_client::SearchServiceClient;
use crate::proto::SearchIssuesRequest;

pub async fn handle(
    query: String,
    order_by: String,
    order_direction: String,
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
                let mut c = SearchServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = SearchServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    let response = call!(search_issues, SearchIssuesRequest {
        query,
        page_size,
        page_token,
        order_by,
        order_direction,
    })?;
    let resp = response.into_inner();

    println!("Found {} total results", resp.total_count);
    output::print_issues(&resp.issues);

    if !resp.next_page_token.is_empty() {
        println!("Next page token: {}", resp.next_page_token);
    }

    Ok(())
}
