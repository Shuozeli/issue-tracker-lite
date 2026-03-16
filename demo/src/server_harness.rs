use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::oneshot;
use tonic::transport::Server;

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

/// An in-process demo server with a temporary database.
/// Drops cleanly, shutting down the server and deleting the temp DB.
pub struct DemoServer {
    pub port: u16,
    _db_file: NamedTempFile,
    _shutdown_tx: oneshot::Sender<()>,
}

impl DemoServer {
    pub async fn start() -> anyhow::Result<Self> {
        let db_file = NamedTempFile::new()?;
        let db_url = format!("file:{}", db_file.path().display());

        let db_conn = db::init_db(&db_url)
            .await
            .map_err(|e| anyhow::anyhow!("failed to init db: {}", e))?;

        // Bind to random port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let port = addr.port();

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let c = db_conn.clone();
        let identity: Arc<dyn identity::IdentityProvider> =
            Arc::new(SqliteIdentityProvider::new(c.clone()));
        tokio::spawn(async move {
            let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
            let _ = Server::builder()
                .add_service(HealthServiceServer::new(HealthServiceImpl))
                .add_service(ComponentServiceServer::new(ComponentServiceImpl {
                    db: c.clone(),
                    identity: identity.clone(),
                }))
                .add_service(IssueServiceServer::new(IssueServiceImpl {
                    db: c.clone(),
                    identity: identity.clone(),
                }))
                .add_service(CommentServiceServer::new(CommentServiceImpl {
                    db: c.clone(),
                    identity: identity.clone(),
                }))
                .add_service(HotlistServiceServer::new(HotlistServiceImpl {
                    db: c.clone(),
                    identity: identity.clone(),
                }))
                .add_service(SearchServiceServer::new(SearchServiceImpl {
                    db: c.clone(),
                    identity: identity.clone(),
                }))
                .add_service(EventLogServiceServer::new(EventLogServiceImpl {
                    db: c.clone(),
                }))
                .add_service(AclServiceServer::new(AclServiceImpl {
                    db: c.clone(),
                    identity: identity.clone(),
                }))
                .add_service(GroupServiceServer::new(GroupServiceImpl {
                    identity: identity.clone(),
                }))
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        Ok(DemoServer {
            port,
            _db_file: db_file,
            _shutdown_tx: shutdown_tx,
        })
    }

    pub fn server_addr(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}
