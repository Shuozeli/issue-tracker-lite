use anyhow::Result;
use clap::Subcommand;

use crate::output;
use crate::proto::hotlist_service_client::HotlistServiceClient;
use crate::proto::{
    AddIssueToHotlistRequest, CreateHotlistRequest, GetHotlistRequest, ListHotlistIssuesRequest,
    ListHotlistsRequest, RemoveIssueFromHotlistRequest, ReorderHotlistIssuesRequest,
    UpdateHotlistRequest,
};

#[derive(Subcommand)]
pub enum HotlistCommand {
    /// Create a new hotlist
    Create {
        /// Hotlist name
        #[arg(short, long)]
        name: String,
        /// Description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Owner email
        #[arg(short, long, default_value = "")]
        owner: String,
    },
    /// Get a hotlist by ID
    Get {
        /// Hotlist ID
        id: i64,
    },
    /// List hotlists
    List {
        /// Filter: active, archived, all
        #[arg(short, long, default_value = "active")]
        filter: String,
        /// Page size
        #[arg(long, default_value = "50")]
        page_size: i32,
        /// Page token
        #[arg(long, default_value = "")]
        page_token: String,
    },
    /// Update a hotlist
    Update {
        /// Hotlist ID
        id: i64,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// Archive/unarchive
        #[arg(long)]
        archived: Option<bool>,
    },
    /// Add an issue to a hotlist
    AddIssue {
        /// Hotlist ID
        hotlist_id: i64,
        /// Issue ID
        issue_id: i64,
        /// Added by (email)
        #[arg(short, long, default_value = "")]
        by: String,
    },
    /// Remove an issue from a hotlist
    RemoveIssue {
        /// Hotlist ID
        hotlist_id: i64,
        /// Issue ID
        issue_id: i64,
    },
    /// List issues in a hotlist
    Issues {
        /// Hotlist ID
        id: i64,
    },
    /// Reorder issues in a hotlist
    Reorder {
        /// Hotlist ID
        hotlist_id: i64,
        /// Issue IDs in new order (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        order: Vec<i64>,
    },
}

pub async fn handle(cmd: HotlistCommand, server: &str, user: Option<&str>) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = HotlistServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = HotlistServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    match cmd {
        HotlistCommand::Create {
            name,
            description,
            owner,
        } => {
            let response = call!(create_hotlist, CreateHotlistRequest {
                name,
                description,
                owner,
            })?;
            output::print_hotlist(&response.into_inner());
        }
        HotlistCommand::Get { id } => {
            let response = call!(get_hotlist, GetHotlistRequest { hotlist_id: id })?;
            output::print_hotlist(&response.into_inner());
        }
        HotlistCommand::List {
            filter,
            page_size,
            page_token,
        } => {
            let response = call!(list_hotlists, ListHotlistsRequest {
                filter,
                page_size,
                page_token,
            })?;
            let resp = response.into_inner();
            output::print_hotlists(&resp.hotlists);
            if !resp.next_page_token.is_empty() {
                println!("Next page token: {}", resp.next_page_token);
            }
        }
        HotlistCommand::Update {
            id,
            name,
            description,
            archived,
        } => {
            let response = call!(update_hotlist, UpdateHotlistRequest {
                hotlist_id: id,
                name,
                description,
                archived,
            })?;
            output::print_hotlist(&response.into_inner());
        }
        HotlistCommand::AddIssue {
            hotlist_id,
            issue_id,
            by,
        } => {
            let response = call!(add_issue, AddIssueToHotlistRequest {
                hotlist_id,
                issue_id,
                added_by: by,
            })?;
            let hi = response.into_inner();
            println!(
                "Issue {} added to hotlist {} at position {}",
                hi.issue_id, hi.hotlist_id, hi.position
            );
        }
        HotlistCommand::RemoveIssue {
            hotlist_id,
            issue_id,
        } => {
            call!(remove_issue, RemoveIssueFromHotlistRequest {
                hotlist_id,
                issue_id,
            })?;
            println!("Issue {} removed from hotlist {}", issue_id, hotlist_id);
        }
        HotlistCommand::Issues { id } => {
            let response = call!(list_issues, ListHotlistIssuesRequest { hotlist_id: id })?;
            let resp = response.into_inner();
            output::print_hotlist_issues(&resp.issues);
        }
        HotlistCommand::Reorder {
            hotlist_id,
            order,
        } => {
            call!(reorder_issues, ReorderHotlistIssuesRequest {
                hotlist_id,
                issue_ids: order,
            })?;
            println!("Issues reordered in hotlist {}", hotlist_id);
        }
    }

    Ok(())
}
