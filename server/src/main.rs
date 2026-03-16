use std::sync::Arc;

use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use identity::SqliteIdentityProvider;
use issuetracker_server::db;
use issuetracker_server::identity_proto::group_service_server::GroupServiceServer;
use issuetracker_server::proto::acl_service_server::AclServiceServer;
use issuetracker_server::proto::comment_service_server::CommentServiceServer;
use issuetracker_server::proto::component_service_server::ComponentServiceServer;
use issuetracker_server::proto::event_log_service_server::EventLogServiceServer;
use issuetracker_server::proto::health_service_server::HealthServiceServer;
use issuetracker_server::proto::hotlist_service_server::HotlistServiceServer;
use issuetracker_server::proto::issue_service_server::IssueServiceServer;
use issuetracker_server::proto::search_service_server::SearchServiceServer;
use issuetracker_server::service::acl_service::AclServiceImpl;
use issuetracker_server::service::comment_service::CommentServiceImpl;
use issuetracker_server::service::component_service::ComponentServiceImpl;
use issuetracker_server::service::event_log_service::EventLogServiceImpl;
use issuetracker_server::service::group_service::GroupServiceImpl;
use issuetracker_server::service::health_service::HealthServiceImpl;
use issuetracker_server::service::hotlist_service::HotlistServiceImpl;
use issuetracker_server::service::issue_service::IssueServiceImpl;
use issuetracker_server::service::search_service::SearchServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db_conn = db::init_db(&db_url)
        .await
        .expect("failed to initialize database");

    // The identity provider shares the same DbConn (same SQLite file, same schema).
    let identity: Arc<dyn identity::IdentityProvider> =
        Arc::new(SqliteIdentityProvider::new(db_conn.clone()));

    let addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()?;

    tracing::info!("issuetracker-server listening on {}", addr);

    Server::builder()
        .add_service(HealthServiceServer::new(HealthServiceImpl))
        .add_service(ComponentServiceServer::new(ComponentServiceImpl {
            db: db_conn.clone(),
            identity: identity.clone(),
        }))
        .add_service(IssueServiceServer::new(IssueServiceImpl {
            db: db_conn.clone(),
            identity: identity.clone(),
        }))
        .add_service(CommentServiceServer::new(CommentServiceImpl {
            db: db_conn.clone(),
            identity: identity.clone(),
        }))
        .add_service(HotlistServiceServer::new(HotlistServiceImpl {
            db: db_conn.clone(),
            identity: identity.clone(),
        }))
        .add_service(SearchServiceServer::new(SearchServiceImpl {
            db: db_conn.clone(),
            identity: identity.clone(),
        }))
        .add_service(EventLogServiceServer::new(EventLogServiceImpl {
            db: db_conn.clone(),
        }))
        .add_service(AclServiceServer::new(AclServiceImpl {
            db: db_conn.clone(),
            identity: identity.clone(),
        }))
        .add_service(GroupServiceServer::new(GroupServiceImpl {
            identity: identity.clone(),
        }))
        .serve(addr)
        .await?;

    Ok(())
}
