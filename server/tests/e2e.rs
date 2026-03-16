//! E2E tests that exercise the CLI binary against a real server.
//! Each test mirrors a demo pipeline from demo/src/pipeline/*.rs.

use std::path::PathBuf;
use std::sync::Arc;

use identity::SqliteIdentityProvider;
use tempfile::NamedTempFile;
use tokio::sync::oneshot;
use tonic::transport::Server;

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

// ── Test Harness ────────────────────────────────────────────────────────

struct E2eHarness {
    server_addr: String,
    it_binary: PathBuf,
    _db_file: NamedTempFile,
    _shutdown_tx: oneshot::Sender<()>,
}

impl E2eHarness {
    async fn new() -> Self {
        let db_file = NamedTempFile::new().expect("failed to create temp file");
        let db_url = format!("file:{}", db_file.path().display());

        let db_conn = issuetracker_server::db::init_db(&db_url)
            .await
            .expect("failed to init db");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind");
        let addr = listener.local_addr().expect("failed to get local addr");
        let server_addr = format!("http://127.0.0.1:{}", addr.port());

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

        // Find the `it` binary
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let it_binary = workspace_root.join("target/debug/it");
        assert!(
            it_binary.exists(),
            "CLI binary not found at {:?}. Run `cargo build -p issuetracker-cli` first.",
            it_binary
        );

        E2eHarness {
            server_addr,
            it_binary,
            _db_file: db_file,
            _shutdown_tx: shutdown_tx,
        }
    }

    /// Run a CLI command and return (exit_success, stdout, stderr).
    async fn run(&self, args: &[&str]) -> (bool, String, String) {
        let output = tokio::process::Command::new(&self.it_binary)
            .arg("--server")
            .arg(&self.server_addr)
            .args(args)
            .output()
            .await
            .expect("failed to execute CLI");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        (output.status.success(), stdout, stderr)
    }

    /// Run a CLI command, assert success, and assert stdout contains all expected substrings.
    async fn run_ok(&self, desc: &str, args: &[&str], assert_contains: &[&str]) {
        let (ok, stdout, stderr) = self.run(args).await;
        assert!(
            ok,
            "Step '{}' failed.\n  Args: {:?}\n  Stderr: {}",
            desc, args, stderr
        );
        for expected in assert_contains {
            assert!(
                stdout.contains(expected),
                "Step '{}' output missing '{}'.\n  Stdout: {}",
                desc,
                expected,
                stdout
            );
        }
    }

    /// Run a CLI command and assert it fails.
    async fn run_fail(&self, desc: &str, args: &[&str]) {
        let (ok, stdout, _stderr) = self.run(args).await;
        assert!(
            !ok,
            "Step '{}' should have failed but succeeded.\n  Stdout: {}",
            desc, stdout
        );
    }
}

// ── E2E: Quickstart Pipeline ────────────────────────────────────────────

#[tokio::test]
async fn e2e_quickstart() {
    let h = E2eHarness::new().await;

    h.run_ok("ping", &["--user", "admin@demo.com", "ping"], &["pong"])
        .await;
    h.run_ok(
        "create component",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "MyProject",
            "--description",
            "Main project component",
        ],
        &["MyProject"],
    )
    .await;
    h.run_ok(
        "grant admin",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "create issue 1",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Login page crashes on empty password",
            "--priority",
            "P0",
            "--type",
            "BUG",
            "--reporter",
            "qa@example.com",
        ],
        &["Login page crashes"],
    )
    .await;
    h.run_ok(
        "create issue 2",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Add dark mode support",
            "--priority",
            "P2",
            "--type",
            "FEATURE_REQUEST",
            "--reporter",
            "pm@example.com",
        ],
        &["dark mode"],
    )
    .await;
    h.run_ok(
        "create issue 3",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Memory leak in background sync worker",
            "--priority",
            "P1",
            "--type",
            "BUG",
            "--reporter",
            "ops@example.com",
        ],
        &["Memory leak"],
    )
    .await;
    h.run_ok(
        "list issues",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "list",
            "--component",
            "1",
        ],
        &["Login page", "dark mode", "Memory leak"],
    )
    .await;
    h.run_ok(
        "assign + in_progress",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--assignee",
            "alice@example.com",
            "--status",
            "IN_PROGRESS",
        ],
        &["IN_PROGRESS"],
    )
    .await;
    h.run_ok(
        "add comment",
        &[
            "--user",
            "admin@demo.com",
            "comment",
            "add",
            "1",
            "--body",
            "Root cause: missing null check in password validator.",
            "--author",
            "alice@example.com",
        ],
        &["null check"],
    )
    .await;
    h.run_ok(
        "mark fixed",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--status",
            "FIXED",
        ],
        &["FIXED"],
    )
    .await;
    h.run_ok(
        "get issue",
        &["--user", "admin@demo.com", "issue", "get", "1"],
        &["FIXED"],
    )
    .await;
    h.run_ok(
        "list comments",
        &["--user", "admin@demo.com", "comment", "list", "1"],
        &["null check"],
    )
    .await;
    h.run_ok(
        "search memory",
        &["--user", "admin@demo.com", "search", "memory"],
        &["Memory leak"],
    )
    .await;
}

// ── E2E: Hierarchy Pipeline ─────────────────────────────────────────────

#[tokio::test]
async fn e2e_hierarchy() {
    let h = E2eHarness::new().await;

    // Create 4 components with ACL grants
    h.run_ok(
        "create Platform",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "Platform",
            "--description",
            "Top-level",
        ],
        &["Platform"],
    )
    .await;
    h.run_ok(
        "grant comp 1",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "create Frontend",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "Frontend",
            "--description",
            "Frontend",
            "--parent-id",
            "1",
        ],
        &["Frontend"],
    )
    .await;
    h.run_ok(
        "grant comp 2",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "2",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "create Backend",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "Backend",
            "--description",
            "Backend",
            "--parent-id",
            "1",
        ],
        &["Backend"],
    )
    .await;
    h.run_ok(
        "grant comp 3",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "3",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "create Mobile",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "Mobile",
            "--description",
            "Mobile",
            "--parent-id",
            "1",
        ],
        &["Mobile"],
    )
    .await;
    h.run_ok(
        "grant comp 4",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "4",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;

    // List components
    h.run_ok(
        "list root",
        &["--user", "admin@demo.com", "component", "list"],
        &["Platform"],
    )
    .await;
    h.run_ok(
        "list children",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "list",
            "--parent-id",
            "1",
        ],
        &["Frontend", "Backend", "Mobile"],
    )
    .await;

    // Create issues in different components
    h.run_ok(
        "epic",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Improve performance",
            "--priority",
            "P1",
            "--type",
            "EPIC",
        ],
        &["Improve performance"],
    )
    .await;
    h.run_ok(
        "frontend task",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "2",
            "--title",
            "Optimize bundle size",
            "--priority",
            "P2",
            "--type",
            "TASK",
        ],
        &["Optimize bundle"],
    )
    .await;
    h.run_ok(
        "backend task",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "3",
            "--title",
            "Add query caching",
            "--priority",
            "P1",
            "--type",
            "TASK",
        ],
        &["query caching"],
    )
    .await;
    h.run_ok(
        "mobile task",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "4",
            "--title",
            "Reduce startup time",
            "--priority",
            "P2",
            "--type",
            "TASK",
        ],
        &["startup time"],
    )
    .await;

    // Link parent/child and blocking
    h.run_ok(
        "link 2->1",
        &["--user", "admin@demo.com", "issue", "add-parent", "2", "1"],
        &[],
    )
    .await;
    h.run_ok(
        "link 3->1",
        &["--user", "admin@demo.com", "issue", "add-parent", "3", "1"],
        &[],
    )
    .await;
    h.run_ok(
        "link 4->1",
        &["--user", "admin@demo.com", "issue", "add-parent", "4", "1"],
        &[],
    )
    .await;
    h.run_ok(
        "3 blocks 2",
        &["--user", "admin@demo.com", "issue", "block", "3", "2"],
        &[],
    )
    .await;

    // Verify hierarchy
    h.run_ok(
        "children of epic",
        &["--user", "admin@demo.com", "issue", "children", "1"],
        &["Optimize bundle"],
    )
    .await;
    h.run_ok(
        "parents of task 2",
        &["--user", "admin@demo.com", "issue", "parents", "2"],
        &["Improve performance"],
    )
    .await;
    h.run_ok(
        "get Platform",
        &["--user", "admin@demo.com", "component", "get", "1"],
        &["Platform"],
    )
    .await;
}

// ── E2E: Hotlists Pipeline ──────────────────────────────────────────────

#[tokio::test]
async fn e2e_hotlists() {
    let h = E2eHarness::new().await;

    // Setup component + issues
    h.run_ok(
        "create comp",
        &["--user", "admin@demo.com", "component", "create", "WebApp"],
        &["WebApp"],
    )
    .await;
    h.run_ok(
        "grant",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "issue 1",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "CSRF vulnerability",
            "--priority",
            "P0",
            "--type",
            "VULNERABILITY",
        ],
        &["CSRF"],
    )
    .await;
    h.run_ok(
        "issue 2",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Upgrade DB driver",
            "--priority",
            "P2",
            "--type",
            "INTERNAL_CLEANUP",
        ],
        &["Upgrade DB"],
    )
    .await;
    h.run_ok(
        "issue 3",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Add rate limiting",
            "--priority",
            "P1",
            "--type",
            "FEATURE_REQUEST",
        ],
        &["rate limiting"],
    )
    .await;
    h.run_ok(
        "issue 4",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Migrate auth provider",
            "--priority",
            "P1",
            "--type",
            "TASK",
        ],
        &["auth provider"],
    )
    .await;

    // Create hotlist + ACL
    h.run_ok(
        "create hotlist",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "create",
            "--name",
            "Sprint 12",
            "--description",
            "Sprint work",
            "--owner",
            "pm@example.com",
        ],
        &["Sprint 12"],
    )
    .await;
    h.run_ok(
        "grant hl",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-hotlist",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permission",
            "HOTLIST_ADMIN",
        ],
        &[],
    )
    .await;

    // Add issues to hotlist
    h.run_ok(
        "add issue 1",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "add-issue",
            "1",
            "1",
            "--by",
            "pm@example.com",
        ],
        &["added to hotlist"],
    )
    .await;
    h.run_ok(
        "add issue 3",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "add-issue",
            "1",
            "3",
            "--by",
            "pm@example.com",
        ],
        &["added to hotlist"],
    )
    .await;
    h.run_ok(
        "add issue 4",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "add-issue",
            "1",
            "4",
            "--by",
            "pm@example.com",
        ],
        &["added to hotlist"],
    )
    .await;

    // List, reorder, remove
    h.run_ok(
        "list hl issues",
        &["--user", "admin@demo.com", "hotlist", "issues", "1"],
        &["admin@demo.com"],
    )
    .await;
    h.run_ok(
        "reorder",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "reorder",
            "1",
            "--order",
            "1,4,3",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "list after reorder",
        &["--user", "admin@demo.com", "hotlist", "issues", "1"],
        &["admin@demo.com"],
    )
    .await;

    // Second hotlist
    h.run_ok(
        "create hl 2",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "create",
            "--name",
            "Tech Debt",
            "--owner",
            "tech-lead@example.com",
        ],
        &["Tech Debt"],
    )
    .await;
    h.run_ok(
        "grant hl 2",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-hotlist",
            "2",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permission",
            "HOTLIST_ADMIN",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "add to hl 2",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "add-issue",
            "2",
            "2",
            "--by",
            "tech-lead@example.com",
        ],
        &["added to hotlist"],
    )
    .await;
    h.run_ok(
        "get hl 1",
        &["--user", "admin@demo.com", "hotlist", "get", "1"],
        &["Sprint 12"],
    )
    .await;
    h.run_ok(
        "remove issue",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "remove-issue",
            "1",
            "3",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "list after remove",
        &["--user", "admin@demo.com", "hotlist", "issues", "1"],
        &["admin@demo.com"],
    )
    .await;
}

// ── E2E: Access Control Pipeline ────────────────────────────────────────

#[tokio::test]
async fn e2e_access_control() {
    let h = E2eHarness::new().await;

    h.run_ok(
        "create comp",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "Security",
            "--description",
            "Sensitive",
        ],
        &["Security"],
    )
    .await;
    h.run_ok(
        "grant admin",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;

    // Set ACLs
    h.run_ok(
        "alice edit",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "alice@example.com",
            "--permissions",
            "VIEW_ISSUES,EDIT_ISSUES",
        ],
        &["EDIT_ISSUES"],
    )
    .await;
    h.run_ok(
        "bob view",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "bob@example.com",
            "--permissions",
            "VIEW_ISSUES",
        ],
        &["VIEW_ISSUES"],
    )
    .await;
    h.run_ok(
        "public view_comp",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "public",
            "--identity-value",
            "*",
            "--permissions",
            "VIEW_COMPONENTS",
        ],
        &["VIEW_COMPONENTS"],
    )
    .await;

    // Check permissions
    h.run_ok(
        "check alice",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "check",
            "1",
            "--user",
            "alice@example.com",
        ],
        &["ACL", "EDIT_ISSUES"],
    )
    .await;
    h.run_ok(
        "check bob",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "check",
            "1",
            "--user",
            "bob@example.com",
        ],
        &["ACL", "VIEW_ISSUES"],
    )
    .await;
    h.run_ok(
        "check carol (public)",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "check",
            "1",
            "--user",
            "carol@example.com",
        ],
        &["VIEW_COMPONENTS"],
    )
    .await;

    // Upsert bob to ADMIN_ISSUES
    h.run_ok(
        "bob upgrade",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "bob@example.com",
            "--permissions",
            "ADMIN_ISSUES",
        ],
        &["ADMIN_ISSUES"],
    )
    .await;
    h.run_ok(
        "check bob upgraded",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "check",
            "1",
            "--user",
            "bob@example.com",
        ],
        &["ACL", "EDIT_ISSUES", "COMMENT_ON_ISSUES"],
    )
    .await;

    // Hotlist ACL
    h.run_ok(
        "create hotlist",
        &[
            "--user",
            "admin@demo.com",
            "hotlist",
            "create",
            "--name",
            "Security Audit",
            "--owner",
            "alice@example.com",
        ],
        &["Security Audit"],
    )
    .await;
    h.run_ok(
        "grant hl admin",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-hotlist",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permission",
            "HOTLIST_ADMIN",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "alice hl admin",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-hotlist",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "alice@example.com",
            "--permission",
            "HOTLIST_ADMIN",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "public hl view",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-hotlist",
            "1",
            "--identity-type",
            "public",
            "--identity-value",
            "*",
            "--permission",
            "HOTLIST_VIEW",
        ],
        &[],
    )
    .await;

    // Remove bob ACL, check fallback to PUBLIC
    h.run_ok(
        "remove bob",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "remove-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "bob@example.com",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "check bob removed",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "check",
            "1",
            "--user",
            "bob@example.com",
        ],
        &["VIEW_COMPONENTS"],
    )
    .await;
}

// ── E2E: Search Pipeline ────────────────────────────────────────────────

#[tokio::test]
async fn e2e_search() {
    let h = E2eHarness::new().await;

    h.run_ok(
        "create comp",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "SearchDemo",
        ],
        &["SearchDemo"],
    )
    .await;
    h.run_ok(
        "grant",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;

    // Create 5 diverse issues
    h.run_ok(
        "issue P0 bug",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Memory leak in connection pool",
            "--priority",
            "P0",
            "--type",
            "BUG",
            "--assignee",
            "alice@example.com",
        ],
        &["Memory leak"],
    )
    .await;
    h.run_ok(
        "issue P1 bug",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Crash on invalid UTF-8 input",
            "--priority",
            "P1",
            "--type",
            "BUG",
            "--assignee",
            "bob@example.com",
        ],
        &["Crash"],
    )
    .await;
    h.run_ok(
        "issue P2 feat",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Add GraphQL endpoint",
            "--priority",
            "P2",
            "--type",
            "FEATURE_REQUEST",
        ],
        &["GraphQL"],
    )
    .await;
    h.run_ok(
        "issue P1 vuln",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "SQL injection in search",
            "--priority",
            "P1",
            "--type",
            "VULNERABILITY",
        ],
        &["SQL injection"],
    )
    .await;
    h.run_ok(
        "issue P3 cleanup",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Remove deprecated v1 API",
            "--priority",
            "P3",
            "--type",
            "INTERNAL_CLEANUP",
        ],
        &["deprecated"],
    )
    .await;

    // Fix one issue
    h.run_ok(
        "fix issue 1",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--status",
            "FIXED",
        ],
        &["FIXED"],
    )
    .await;

    // Searches
    h.run_ok(
        "search open",
        &["--user", "admin@demo.com", "search", "status:open"],
        &["Crash"],
    )
    .await;
    h.run_ok(
        "search closed",
        &["--user", "admin@demo.com", "search", "status:closed"],
        &["Memory leak"],
    )
    .await;
    h.run_ok(
        "search P0",
        &["--user", "admin@demo.com", "search", "priority:P0"],
        &["Memory leak"],
    )
    .await;
    h.run_ok(
        "search assignee",
        &[
            "--user",
            "admin@demo.com",
            "search",
            "assignee:alice@example.com",
        ],
        &["alice"],
    )
    .await;
    h.run_ok(
        "search bugs",
        &["--user", "admin@demo.com", "search", "type:BUG"],
        &["BUG"],
    )
    .await;
    h.run_ok(
        "search keyword",
        &["--user", "admin@demo.com", "search", "memory"],
        &["Memory leak"],
    )
    .await;
    h.run_ok(
        "search injection",
        &["--user", "admin@demo.com", "search", "injection"],
        &["injection"],
    )
    .await;
    h.run_ok(
        "search combined",
        &[
            "--user",
            "admin@demo.com",
            "search",
            "status:open priority:P1 type:BUG",
        ],
        &["Crash"],
    )
    .await;
    h.run_ok(
        "search ordered",
        &[
            "--user",
            "admin@demo.com",
            "search",
            "status:open",
            "--order-by",
            "priority",
            "--order-dir",
            "asc",
        ],
        &["Crash"],
    )
    .await;
    h.run_ok(
        "search negation",
        &["--user", "admin@demo.com", "search", "--", "-type:BUG"],
        &["GraphQL"],
    )
    .await;
}

// ── E2E: Full Lifecycle Pipeline ────────────────────────────────────────

#[tokio::test]
async fn e2e_full_lifecycle() {
    let h = E2eHarness::new().await;

    h.run_ok(
        "create comp",
        &[
            "--user",
            "admin@demo.com",
            "component",
            "create",
            "Payments",
            "--description",
            "Payment processing",
        ],
        &["Payments"],
    )
    .await;
    h.run_ok(
        "grant",
        &[
            "--user",
            "admin@demo.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@demo.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;

    // Report bug, assign, progress
    h.run_ok(
        "report bug",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Payment fails for amounts over $10,000",
            "--priority",
            "P0",
            "--type",
            "BUG",
            "--reporter",
            "support@example.com",
        ],
        &["Payment fails"],
    )
    .await;
    h.run_ok(
        "assign",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--assignee",
            "alice@example.com",
        ],
        &["ASSIGNED"],
    )
    .await;
    h.run_ok(
        "in progress",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--status",
            "IN_PROGRESS",
        ],
        &["IN_PROGRESS"],
    )
    .await;
    h.run_ok(
        "investigation comment",
        &[
            "--user",
            "admin@demo.com",
            "comment",
            "add",
            "1",
            "--body",
            "Root cause: i32 overflow at 2^31 cents",
            "--author",
            "alice@example.com",
        ],
        &["Root cause"],
    )
    .await;

    // Blocker: DB migration
    h.run_ok(
        "blocker issue",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Migrate payment_amount to BIGINT",
            "--priority",
            "P0",
            "--type",
            "TASK",
            "--assignee",
            "dba@example.com",
        ],
        &["Migrate"],
    )
    .await;
    h.run_ok(
        "block link",
        &["--user", "admin@demo.com", "issue", "block", "2", "1"],
        &[],
    )
    .await;
    h.run_ok(
        "fix blocker",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "2",
            "--status",
            "FIXED",
        ],
        &["FIXED"],
    )
    .await;

    // Fix original + verify
    h.run_ok(
        "fix comment",
        &[
            "--user",
            "admin@demo.com",
            "comment",
            "add",
            "1",
            "--body",
            "DB migration complete. Tests pass.",
            "--author",
            "alice@example.com",
        ],
        &["migration complete"],
    )
    .await;
    h.run_ok(
        "fix original",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--status",
            "FIXED",
        ],
        &["FIXED"],
    )
    .await;
    h.run_ok(
        "qa comment",
        &[
            "--user",
            "admin@demo.com",
            "comment",
            "add",
            "1",
            "--body",
            "Verified: large payments work in staging.",
            "--author",
            "qa@example.com",
        ],
        &["Verified"],
    )
    .await;
    h.run_ok(
        "verified",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--status",
            "FIXED_VERIFIED",
        ],
        &["FIXED_VERIFIED"],
    )
    .await;
    h.run_ok(
        "get verified",
        &["--user", "admin@demo.com", "issue", "get", "1"],
        &["FIXED_VERIFIED"],
    )
    .await;
    h.run_ok(
        "list comments",
        &["--user", "admin@demo.com", "comment", "list", "1"],
        &["Root cause"],
    )
    .await;

    // Duplicate
    h.run_ok(
        "dup report",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "create",
            "--component",
            "1",
            "--title",
            "Large payments rejected 422",
            "--priority",
            "P0",
            "--type",
            "BUG",
            "--reporter",
            "sales@example.com",
        ],
        &["Large payments"],
    )
    .await;
    h.run_ok(
        "mark dup",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "duplicate",
            "3",
            "--of",
            "1",
        ],
        &["DUPLICATE"],
    )
    .await;
    h.run_ok(
        "get original",
        &["--user", "admin@demo.com", "issue", "get", "1"],
        &["FIXED_VERIFIED"],
    )
    .await;

    // Invalid transition
    h.run_fail(
        "bad transition",
        &[
            "--user",
            "admin@demo.com",
            "issue",
            "update",
            "1",
            "--status",
            "IN_PROGRESS",
        ],
    )
    .await;

    // Event log
    h.run_ok(
        "events",
        &[
            "--user",
            "admin@demo.com",
            "events",
            "--entity-type",
            "Issue",
            "--entity-id",
            "1",
        ],
        &["ISSUE"],
    )
    .await;
}

// ── E2E: Groups Pipeline ────────────────────────────────────────────────

#[tokio::test]
async fn e2e_groups() {
    let h = E2eHarness::new().await;

    // Create groups
    h.run_ok(
        "create engineering",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "create",
            "engineering",
            "--display-name",
            "Engineering",
        ],
        &["engineering"],
    )
    .await;
    h.run_ok(
        "create frontend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "create",
            "frontend",
            "--display-name",
            "Frontend",
        ],
        &["frontend"],
    )
    .await;
    h.run_ok(
        "create backend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "create",
            "backend",
            "--display-name",
            "Backend",
        ],
        &["backend"],
    )
    .await;
    h.run_ok(
        "create devops",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "create",
            "devops",
            "--display-name",
            "DevOps",
        ],
        &["devops"],
    )
    .await;
    h.run_ok(
        "create all-staff",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "create",
            "all-staff",
            "--display-name",
            "All Staff",
        ],
        &["all-staff"],
    )
    .await;

    // Add users
    h.run_ok(
        "alice->frontend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "frontend",
            "--member-type",
            "user",
            "--member-value",
            "alice@acme.com",
        ],
        &["alice"],
    )
    .await;
    h.run_ok(
        "bob->frontend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "frontend",
            "--member-type",
            "user",
            "--member-value",
            "bob@acme.com",
        ],
        &["bob"],
    )
    .await;
    h.run_ok(
        "carol->backend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "backend",
            "--member-type",
            "user",
            "--member-value",
            "carol@acme.com",
        ],
        &["carol"],
    )
    .await;
    h.run_ok(
        "dave->backend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "backend",
            "--member-type",
            "user",
            "--member-value",
            "dave@acme.com",
        ],
        &["dave"],
    )
    .await;
    h.run_ok(
        "eve->devops",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "devops",
            "--member-type",
            "user",
            "--member-value",
            "eve@acme.com",
        ],
        &["eve"],
    )
    .await;

    // Nest groups
    h.run_ok(
        "frontend->eng",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "engineering",
            "--member-type",
            "group",
            "--member-value",
            "frontend",
        ],
        &["frontend"],
    )
    .await;
    h.run_ok(
        "backend->eng",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "engineering",
            "--member-type",
            "group",
            "--member-value",
            "backend",
        ],
        &["backend"],
    )
    .await;
    h.run_ok(
        "devops->eng",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "engineering",
            "--member-type",
            "group",
            "--member-value",
            "devops",
        ],
        &["devops"],
    )
    .await;
    h.run_ok(
        "eng->all-staff",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "all-staff",
            "--member-type",
            "group",
            "--member-value",
            "engineering",
        ],
        &["engineering"],
    )
    .await;

    // Query hierarchy
    h.run_ok(
        "list groups",
        &["--user", "admin@acme.com", "group", "list"],
        &["engineering", "frontend", "backend", "devops", "all-staff"],
    )
    .await;
    h.run_ok(
        "eng members",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "list-members",
            "engineering",
        ],
        &["frontend", "backend", "devops"],
    )
    .await;
    h.run_ok(
        "frontend members",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "list-members",
            "frontend",
        ],
        &["alice", "bob"],
    )
    .await;
    h.run_ok(
        "resolve alice",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "resolve-groups",
            "alice@acme.com",
        ],
        &["frontend", "engineering", "all-staff"],
    )
    .await;
    h.run_ok(
        "alice in all-staff",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "is-member",
            "alice@acme.com",
            "all-staff",
        ],
        &["IS a member"],
    )
    .await;
    h.run_ok(
        "carol not in frontend",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "is-member",
            "carol@acme.com",
            "frontend",
        ],
        &["NOT a member"],
    )
    .await;

    // ACL integration
    h.run_ok(
        "create comp",
        &[
            "--user",
            "admin@acme.com",
            "component",
            "create",
            "Infrastructure",
        ],
        &["Infrastructure"],
    )
    .await;
    h.run_ok(
        "grant admin",
        &[
            "--user",
            "admin@acme.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@acme.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "group acl",
        &[
            "--user",
            "admin@acme.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "group",
            "--identity-value",
            "engineering",
            "--permissions",
            "VIEW_ISSUES,COMMENT_ON_ISSUES",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "check alice perms",
        &[
            "--user",
            "admin@acme.com",
            "acl",
            "check",
            "1",
            "--user",
            "alice@acme.com",
        ],
        &["VIEW_ISSUES", "COMMENT_ON_ISSUES"],
    )
    .await;

    // More ops
    h.run_ok(
        "frank->devops",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "add-member",
            "devops",
            "--member-type",
            "user",
            "--member-value",
            "frank@acme.com",
        ],
        &["frank"],
    )
    .await;
    h.run_ok(
        "devops members",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "list-members",
            "devops",
        ],
        &["eve", "frank"],
    )
    .await;
    h.run_ok(
        "promote eve",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "update-member-role",
            "devops",
            "--member-type",
            "user",
            "--member-value",
            "eve@acme.com",
            "--role",
            "manager",
        ],
        &["manager"],
    )
    .await;
    h.run_ok(
        "update eng name",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "update",
            "engineering",
            "--display-name",
            "Engineering Division",
        ],
        &["Engineering Division"],
    )
    .await;
    h.run_ok(
        "get eng",
        &["--user", "admin@acme.com", "group", "get", "engineering"],
        &["Engineering Division"],
    )
    .await;

    // Deletion preconditions
    h.run_fail(
        "delete frontend (still member)",
        &["--user", "admin@acme.com", "group", "delete", "frontend"],
    )
    .await;
    h.run_ok(
        "remove frontend from eng",
        &[
            "--user",
            "admin@acme.com",
            "group",
            "remove-member",
            "engineering",
            "--member-type",
            "group",
            "--member-value",
            "frontend",
        ],
        &[],
    )
    .await;
    h.run_ok(
        "delete frontend",
        &["--user", "admin@acme.com", "group", "delete", "frontend"],
        &["deleted"],
    )
    .await;
    h.run_ok(
        "list remaining",
        &["--user", "admin@acme.com", "group", "list"],
        &["engineering", "backend", "devops", "all-staff"],
    )
    .await;
}

// ── E2E: Security Hardening Pipeline ──────────────────────────────────

#[tokio::test]
async fn e2e_security_hardening() {
    let h = E2eHarness::new().await;

    // --- ACL Auth Enforcement ---

    // Unauthenticated component creation still works (component_service allows it for bootstrap)
    h.run_ok(
        "create comp (authed)",
        &["--user", "admin@sec.com", "component", "create", "Secure"],
        &["Secure"],
    )
    .await;

    // Bootstrap: first ACL entry allowed for any authenticated user
    h.run_ok(
        "bootstrap acl",
        &[
            "--user",
            "admin@sec.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@sec.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
        &[],
    )
    .await;

    // Unauthenticated ACL set should fail
    h.run_fail(
        "unauth set-component-acl denied",
        &[
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "evil@hack.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
    )
    .await;

    // Unauthenticated ACL get should fail
    h.run_fail(
        "unauth get-component-acl denied",
        &["acl", "get-component", "1"],
    )
    .await;

    // Unauthenticated ACL remove should fail
    h.run_fail(
        "unauth remove-component-acl denied",
        &[
            "acl",
            "remove-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "admin@sec.com",
        ],
    )
    .await;

    // Unauthenticated permission check should fail
    h.run_fail(
        "unauth check-permission denied",
        &["acl", "check", "1", "--user", "admin@sec.com"],
    )
    .await;

    // Non-admin user cannot modify ACL
    h.run_ok(
        "grant viewer alice",
        &[
            "--user",
            "admin@sec.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "viewer@sec.com",
            "--permissions",
            "VIEW_ISSUES",
        ],
        &[],
    )
    .await;

    h.run_fail(
        "viewer cannot set acl",
        &[
            "--user",
            "viewer@sec.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "viewer@sec.com",
            "--permissions",
            "ADMIN_COMPONENTS",
        ],
    )
    .await;

    // Admin can check permissions
    h.run_ok(
        "admin checks viewer perms",
        &[
            "--user",
            "admin@sec.com",
            "acl",
            "check",
            "1",
            "--user",
            "viewer@sec.com",
        ],
        &["VIEW_ISSUES"],
    )
    .await;

    // --- Hotlist Auth Enforcement ---

    // Unauthenticated hotlist creation should fail
    h.run_fail(
        "unauth create-hotlist denied",
        &[
            "hotlist",
            "create",
            "--name",
            "Evil Hotlist",
            "--owner",
            "attacker@hack.com",
        ],
    )
    .await;

    // Create hotlist: owner is overridden to authenticated user
    h.run_ok(
        "create hotlist (owner override)",
        &[
            "--user",
            "admin@sec.com",
            "hotlist",
            "create",
            "--name",
            "Security Audit",
            "--owner",
            "someone_else@sec.com",
        ],
        &["Security Audit", "admin@sec.com"],
    )
    .await;

    // Creator auto-gets HOTLIST_ADMIN - can view ACL
    h.run_ok(
        "creator can get hotlist acl",
        &["--user", "admin@sec.com", "acl", "get-hotlist", "1"],
        &["admin@sec.com"],
    )
    .await;

    // Unauthenticated hotlist ACL set should fail
    h.run_fail(
        "unauth set-hotlist-acl denied",
        &[
            "acl",
            "set-hotlist",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "evil@hack.com",
            "--permission",
            "HOTLIST_ADMIN",
        ],
    )
    .await;

    // Unauthenticated hotlist ACL get should fail
    h.run_fail(
        "unauth get-hotlist-acl denied",
        &["acl", "get-hotlist", "1"],
    )
    .await;

    // --- Issue creation with ACL enforcement ---
    h.run_ok(
        "grant create issues",
        &[
            "--user",
            "admin@sec.com",
            "acl",
            "set-component",
            "1",
            "--identity-type",
            "user",
            "--identity-value",
            "dev@sec.com",
            "--permissions",
            "CREATE_ISSUES,VIEW_ISSUES",
        ],
        &[],
    )
    .await;

    h.run_ok(
        "dev creates issue",
        &[
            "--user",
            "dev@sec.com",
            "issue",
            "create",
            "-c",
            "1",
            "-t",
            "Fix auth bypass",
            "-p",
            "P0",
            "--type",
            "BUG",
        ],
        &["Fix auth bypass"],
    )
    .await;

    // Viewer cannot create issues
    h.run_fail(
        "viewer cannot create issue",
        &[
            "--user",
            "viewer@sec.com",
            "issue",
            "create",
            "-c",
            "1",
            "-t",
            "Should fail",
            "-p",
            "P2",
            "--type",
            "BUG",
        ],
    )
    .await;
}
