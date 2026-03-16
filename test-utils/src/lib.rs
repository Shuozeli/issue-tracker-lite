use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::oneshot;
use tonic::service::Interceptor;
use tonic::transport::{Channel, Server};

use identity::SqliteIdentityProvider;
use issuetracker_server::identity_proto::group_service_client::GroupServiceClient;
use issuetracker_server::identity_proto::group_service_server::GroupServiceServer;
use issuetracker_server::proto::acl_service_client::AclServiceClient;
use issuetracker_server::proto::acl_service_server::AclServiceServer;
use issuetracker_server::proto::comment_service_client::CommentServiceClient;
use issuetracker_server::proto::comment_service_server::CommentServiceServer;
use issuetracker_server::proto::component_service_client::ComponentServiceClient;
use issuetracker_server::proto::component_service_server::ComponentServiceServer;
use issuetracker_server::proto::event_log_service_client::EventLogServiceClient;
use issuetracker_server::proto::event_log_service_server::EventLogServiceServer;
use issuetracker_server::proto::health_service_client::HealthServiceClient;
use issuetracker_server::proto::health_service_server::HealthServiceServer;
use issuetracker_server::proto::hotlist_service_client::HotlistServiceClient;
use issuetracker_server::proto::hotlist_service_server::HotlistServiceServer;
use issuetracker_server::proto::issue_service_client::IssueServiceClient;
use issuetracker_server::proto::issue_service_server::IssueServiceServer;
use issuetracker_server::proto::search_service_client::SearchServiceClient;
use issuetracker_server::proto::search_service_server::SearchServiceServer;

pub use issuetracker_server::identity_proto;
pub use issuetracker_server::proto::*;
use issuetracker_server::service::acl_service::AclServiceImpl;
use issuetracker_server::service::comment_service::CommentServiceImpl;
use issuetracker_server::service::component_service::ComponentServiceImpl;
use issuetracker_server::service::event_log_service::EventLogServiceImpl;
use issuetracker_server::service::group_service::GroupServiceImpl;
use issuetracker_server::service::health_service::HealthServiceImpl;
use issuetracker_server::service::hotlist_service::HotlistServiceImpl;
use issuetracker_server::service::issue_service::IssueServiceImpl;
use issuetracker_server::service::search_service::SearchServiceImpl;

// ── Test Fixture ──────────────────────────────────────────────────────────

pub struct TestFixture {
    pub channel: Channel,
    _db_file: NamedTempFile,
    _shutdown_tx: oneshot::Sender<()>,
}

impl TestFixture {
    pub async fn new() -> Self {
        let db_file = NamedTempFile::new().expect("failed to create temp file");
        let db_url = format!("file:{}", db_file.path().display());

        // Initialize quiver database
        let db_conn = issuetracker_server::db::init_db(&db_url)
            .await
            .expect("failed to init db");

        // Bind to random port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind");
        let addr = listener.local_addr().expect("failed to get local addr");

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let c = db_conn.clone();
        let identity: Arc<dyn identity::IdentityProvider> =
            Arc::new(SqliteIdentityProvider::new(c.clone()));
        tokio::spawn(async move {
            let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
            Server::builder()
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
                .await
                .expect("server failed");
        });

        let channel = Channel::from_shared(format!("http://127.0.0.1:{}", addr.port()))
            .expect("invalid uri")
            .connect()
            .await
            .expect("failed to connect");

        TestFixture {
            channel,
            _db_file: db_file,
            _shutdown_tx: shutdown_tx,
        }
    }

    pub fn component_client(
        &self,
    ) -> ComponentServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        ComponentServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn issue_client(
        &self,
    ) -> IssueServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        IssueServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn comment_client(
        &self,
    ) -> CommentServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        CommentServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn hotlist_client(
        &self,
    ) -> HotlistServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        HotlistServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn search_client(
        &self,
    ) -> SearchServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        SearchServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn event_client(
        &self,
    ) -> EventLogServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        EventLogServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn health_client(&self) -> HealthServiceClient<Channel> {
        HealthServiceClient::new(self.channel.clone())
    }

    pub fn acl_client(
        &self,
    ) -> AclServiceClient<tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>>
    {
        AclServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn group_client(
        &self,
    ) -> GroupServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>,
    > {
        GroupServiceClient::with_interceptor(self.channel.clone(), AdminInterceptor)
    }

    pub fn unauthenticated_component_client(&self) -> ComponentServiceClient<Channel> {
        ComponentServiceClient::new(self.channel.clone())
    }

    pub fn unauthenticated_group_client(&self) -> GroupServiceClient<Channel> {
        GroupServiceClient::new(self.channel.clone())
    }
}

/// Test interceptor that injects x-user-id: admin@test.com into every request
/// unless the header is already present.
#[derive(Clone)]
pub struct AdminInterceptor;

pub const TEST_ADMIN_USER: &str = "admin@test.com";

impl Interceptor for AdminInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        if request.metadata().get("x-user-id").is_none() {
            request
                .metadata_mut()
                .insert("x-user-id", TEST_ADMIN_USER.parse().unwrap());
        }
        Ok(request)
    }
}

pub type AuthChannel = tonic::service::interceptor::InterceptedService<Channel, AdminInterceptor>;

/// Helper: grant the test admin user ADMIN_COMPONENTS permission on a component.
pub async fn grant_admin(acl: &mut AclServiceClient<AuthChannel>, component_id: i64) {
    acl.set_component_acl(SetComponentAclRequest {
        component_id,
        identity_type: 1, // USER
        identity_value: TEST_ADMIN_USER.to_string(),
        permissions: vec![7], // ADMIN_COMPONENTS
    })
    .await
    .expect("grant_admin failed");
}

/// Helper: grant the test admin user HOTLIST_ADMIN permission on a hotlist.
pub async fn grant_hotlist_admin(acl: &mut AclServiceClient<AuthChannel>, hotlist_id: i64) {
    acl.set_hotlist_acl(SetHotlistAclRequest {
        hotlist_id,
        identity_type: 1, // USER
        identity_value: TEST_ADMIN_USER.to_string(),
        permission: 3, // HOTLIST_ADMIN
    })
    .await
    .expect("grant_hotlist_admin failed");
}

// Helper: create a component, grant admin to test user, and return its ID
pub async fn create_component(
    client: &mut ComponentServiceClient<AuthChannel>,
    acl: &mut AclServiceClient<AuthChannel>,
    name: &str,
    parent_id: Option<i64>,
) -> i64 {
    let resp = client
        .create_component(CreateComponentRequest {
            name: name.to_string(),
            description: String::new(),
            parent_id,
        })
        .await
        .expect("create_component failed");
    let comp_id = resp.into_inner().component_id;
    grant_admin(acl, comp_id).await;
    comp_id
}

// Helper: create an issue with defaults and return it
pub async fn create_issue(
    client: &mut IssueServiceClient<AuthChannel>,
    component_id: i64,
    title: &str,
) -> Issue {
    client
        .create_issue(CreateIssueRequest {
            component_id,
            title: title.to_string(),
            description: String::new(),
            priority: Priority::P2 as i32,
            r#type: IssueType::Bug as i32,
            severity: None,
            assignee: None,
            reporter: Some("test@example.com".to_string()),
            verifier: None,
            found_in: None,
            targeted_to: None,
        })
        .await
        .expect("create_issue failed")
        .into_inner()
}

/// Helper: create a tonic Request with x-user-id metadata header
pub fn with_user<T>(user_id: &str, msg: T) -> tonic::Request<T> {
    let mut req = tonic::Request::new(msg);
    req.metadata_mut()
        .insert("x-user-id", user_id.parse().unwrap());
    req
}

#[derive(Clone)]
pub struct UserInterceptor(pub String);

impl Interceptor for UserInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request
            .metadata_mut()
            .insert("x-user-id", self.0.parse().unwrap());
        Ok(request)
    }
}
