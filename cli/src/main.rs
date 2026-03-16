use anyhow::Result;
use clap::{Parser, Subcommand};

mod auth;
mod commands;
mod output;

mod proto {
    tonic::include_proto!("issuetracker.v1");
}

mod identity_proto {
    tonic::include_proto!("identity.v1");
}

#[derive(Parser)]
#[command(name = "it", about = "Issue Tracker CLI")]
struct Cli {
    /// gRPC server address
    #[arg(long, env = "IT_SERVER_ADDR", default_value = "http://localhost:50051")]
    server: String,

    /// Authenticated user identity (sent as x-user-id header)
    #[arg(long, env = "IT_USER")]
    user: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check server health
    Ping,
    /// Manage access control lists
    #[command(subcommand)]
    Acl(commands::acl::AclCommand),
    /// Manage components
    #[command(subcommand)]
    Component(commands::component::ComponentCommand),
    /// Manage issues
    #[command(subcommand)]
    Issue(commands::issue::IssueCommand),
    /// Manage comments
    #[command(subcommand)]
    Comment(commands::comment::CommentCommand),
    /// Manage hotlists
    #[command(subcommand)]
    Hotlist(commands::hotlist::HotlistCommand),
    /// Manage groups
    #[command(subcommand)]
    Group(commands::group::GroupCommand),
    /// Query event log
    Events {
        /// Entity type filter: Issue, Component, Hotlist, Comment
        #[arg(long, default_value = "")]
        entity_type: String,
        /// Entity ID filter (0 = all)
        #[arg(long, default_value = "0")]
        entity_id: i64,
        /// Event type filter: ISSUE_CREATED, ISSUE_UPDATED, etc.
        #[arg(long, default_value = "")]
        event_type: String,
        /// Actor filter
        #[arg(long, default_value = "")]
        actor: String,
        /// Page size
        #[arg(long, default_value = "50")]
        page_size: i32,
        /// Page token
        #[arg(long, default_value = "")]
        page_token: String,
    },
    /// Search issues
    Search {
        /// Search query (e.g. "status:open priority:P0 memory leak")
        query: String,
        /// Order by: created, modified, priority
        #[arg(long, default_value = "modified")]
        order_by: String,
        /// Order direction: asc, desc
        #[arg(long, default_value = "desc")]
        order_dir: String,
        /// Page size
        #[arg(long, default_value = "50")]
        page_size: i32,
        /// Page token
        #[arg(long, default_value = "")]
        page_token: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ping => {
            use proto::health_service_client::HealthServiceClient;
            use proto::PingRequest;

            let channel = tonic::transport::Channel::from_shared(cli.server.clone())?
                .connect()
                .await?;
            let response = if let Some(ref uid) = cli.user {
                let mut client = HealthServiceClient::with_interceptor(
                    channel,
                    auth::UserInterceptor::new(uid.clone()),
                );
                client.ping(PingRequest {}).await?
            } else {
                let mut client = HealthServiceClient::new(channel);
                client.ping(PingRequest {}).await?
            };
            println!("{}", response.into_inner().message);
        }
        Commands::Acl(cmd) => {
            commands::acl::handle(cmd, &cli.server, cli.user.as_deref()).await?;
        }
        Commands::Component(cmd) => {
            commands::component::handle(cmd, &cli.server, cli.user.as_deref()).await?;
        }
        Commands::Issue(cmd) => {
            commands::issue::handle(cmd, &cli.server, cli.user.as_deref()).await?;
        }
        Commands::Comment(cmd) => {
            commands::comment::handle(cmd, &cli.server, cli.user.as_deref()).await?;
        }
        Commands::Hotlist(cmd) => {
            commands::hotlist::handle(cmd, &cli.server, cli.user.as_deref()).await?;
        }
        Commands::Group(cmd) => {
            commands::group::handle(cmd, &cli.server, cli.user.as_deref()).await?;
        }
        Commands::Events {
            entity_type,
            entity_id,
            event_type,
            actor,
            page_size,
            page_token,
        } => {
            commands::events::handle(
                entity_type,
                entity_id,
                event_type,
                actor,
                page_size,
                page_token,
                &cli.server,
                cli.user.as_deref(),
            )
            .await?;
        }
        Commands::Search {
            query,
            order_by,
            order_dir,
            page_size,
            page_token,
        } => {
            commands::search::handle(
                query,
                order_by,
                order_dir,
                page_size,
                page_token,
                &cli.server,
                cli.user.as_deref(),
            )
            .await?;
        }
    }

    Ok(())
}
