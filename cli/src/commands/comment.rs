use anyhow::Result;
use clap::Subcommand;

use crate::output;
use crate::proto::comment_service_client::CommentServiceClient;
use crate::proto::{CreateCommentRequest, ListCommentsRequest, UpdateCommentRequest};

#[derive(Subcommand)]
pub enum CommentCommand {
    /// Add a comment to an issue
    Add {
        /// Issue ID
        issue_id: i64,
        /// Comment body
        #[arg(short, long)]
        body: String,
        /// Author email
        #[arg(short, long, default_value = "")]
        author: String,
    },
    /// List comments on an issue
    List {
        /// Issue ID
        issue_id: i64,
        /// Page size
        #[arg(long, default_value = "50")]
        page_size: i32,
        /// Page token
        #[arg(long, default_value = "")]
        page_token: String,
    },
    /// Edit a comment
    Edit {
        /// Comment ID
        comment_id: i64,
        /// New body
        #[arg(short, long)]
        body: String,
    },
}

pub async fn handle(cmd: CommentCommand, server: &str, user: Option<&str>) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = CommentServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = CommentServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    match cmd {
        CommentCommand::Add {
            issue_id,
            body,
            author,
        } => {
            let response = call!(create_comment, CreateCommentRequest {
                issue_id,
                body,
                author,
            })?;
            output::print_comment(&response.into_inner());
        }
        CommentCommand::List {
            issue_id,
            page_size,
            page_token,
        } => {
            let response = call!(list_comments, ListCommentsRequest {
                issue_id,
                page_size,
                page_token,
            })?;
            let resp = response.into_inner();
            output::print_comments(&resp.comments);
            if !resp.next_page_token.is_empty() {
                println!("Next page token: {}", resp.next_page_token);
            }
        }
        CommentCommand::Edit { comment_id, body } => {
            let response = call!(update_comment, UpdateCommentRequest { comment_id, body })?;
            output::print_comment(&response.into_inner());
        }
    }

    Ok(())
}
