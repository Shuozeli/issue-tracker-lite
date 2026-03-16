use anyhow::Result;
use clap::Subcommand;

use crate::output;
use crate::proto::issue_service_client::IssueServiceClient;
use crate::proto::{
    AddBlockingRequest, AddParentRequest, CreateIssueRequest, GetIssueRequest,
    ListIssuesRequest, ListRelatedIssuesRequest, MarkDuplicateRequest, RemoveBlockingRequest,
    RemoveParentRequest, UnmarkDuplicateRequest, UpdateIssueRequest,
};

#[derive(Subcommand)]
pub enum IssueCommand {
    /// Create a new issue
    Create {
        /// Component ID
        #[arg(short, long)]
        component: i64,
        /// Issue title
        #[arg(short, long)]
        title: String,
        /// Description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Priority: P0, P1, P2, P3, P4
        #[arg(short, long, default_value = "P2")]
        priority: String,
        /// Type: BUG, FEATURE_REQUEST, TASK, etc.
        #[arg(long, default_value = "BUG")]
        r#type: String,
        /// Severity: S0, S1, S2, S3, S4
        #[arg(short, long)]
        severity: Option<String>,
        /// Assignee email
        #[arg(short, long)]
        assignee: Option<String>,
        /// Reporter email
        #[arg(short, long)]
        reporter: Option<String>,
    },
    /// Get an issue by ID
    Get {
        /// Issue ID
        id: i64,
    },
    /// List issues in a component
    List {
        /// Component ID
        #[arg(short, long)]
        component: i64,
        /// Status filter: open, closed, all
        #[arg(short, long, default_value = "open")]
        status: String,
        /// Page size
        #[arg(long, default_value = "50")]
        page_size: i32,
        /// Page token
        #[arg(long, default_value = "")]
        page_token: String,
    },
    /// Update an issue
    Update {
        /// Issue ID
        id: i64,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// New status: NEW, ASSIGNED, IN_PROGRESS, FIXED, etc.
        #[arg(short, long)]
        status: Option<String>,
        /// New priority: P0-P4
        #[arg(short, long)]
        priority: Option<String>,
        /// New severity: S0-S4
        #[arg(long)]
        severity: Option<String>,
        /// New type
        #[arg(long)]
        r#type: Option<String>,
        /// New assignee email (use "" to clear)
        #[arg(short, long)]
        assignee: Option<String>,
        /// New component ID
        #[arg(short, long)]
        component: Option<i64>,
    },
    /// Add a parent to an issue
    AddParent {
        /// Child issue ID
        child_id: i64,
        /// Parent issue ID
        parent_id: i64,
    },
    /// Remove a parent from an issue
    RemoveParent {
        /// Child issue ID
        child_id: i64,
        /// Parent issue ID
        parent_id: i64,
    },
    /// List parents of an issue
    Parents {
        /// Issue ID
        id: i64,
    },
    /// List children of an issue
    Children {
        /// Issue ID
        id: i64,
    },
    /// Add a blocking relationship
    Block {
        /// Blocking issue ID
        blocking_id: i64,
        /// Blocked issue ID
        blocked_id: i64,
    },
    /// Remove a blocking relationship
    Unblock {
        /// Blocking issue ID
        blocking_id: i64,
        /// Blocked issue ID
        blocked_id: i64,
    },
    /// Mark an issue as duplicate
    Duplicate {
        /// Issue ID to mark as duplicate
        id: i64,
        /// Canonical issue ID
        #[arg(long)]
        of: i64,
    },
    /// Unmark an issue as duplicate
    Unduplicate {
        /// Issue ID
        id: i64,
    },
}

fn parse_priority(s: &str) -> i32 {
    match s {
        "P0" => 1,
        "P1" => 2,
        "P2" => 3,
        "P3" => 4,
        "P4" => 5,
        _ => 0,
    }
}

fn parse_severity(s: &str) -> i32 {
    match s {
        "S0" => 1,
        "S1" => 2,
        "S2" => 3,
        "S3" => 4,
        "S4" => 5,
        _ => 0,
    }
}

fn parse_issue_type(s: &str) -> i32 {
    match s {
        "BUG" => 1,
        "FEATURE_REQUEST" => 2,
        "CUSTOMER_ISSUE" => 3,
        "INTERNAL_CLEANUP" => 4,
        "PROCESS" => 5,
        "VULNERABILITY" => 6,
        "PRIVACY_ISSUE" => 7,
        "PROGRAM" => 8,
        "PROJECT" => 9,
        "FEATURE" => 10,
        "MILESTONE" => 11,
        "EPIC" => 12,
        "STORY" => 13,
        "TASK" => 14,
        _ => 0,
    }
}

fn parse_status(s: &str) -> i32 {
    match s {
        "NEW" => 1,
        "ASSIGNED" => 2,
        "IN_PROGRESS" => 3,
        "INACTIVE" => 4,
        "FIXED" => 5,
        "FIXED_VERIFIED" => 6,
        "WONT_FIX_INFEASIBLE" => 7,
        "WONT_FIX_NOT_REPRODUCIBLE" => 8,
        "WONT_FIX_OBSOLETE" => 9,
        "WONT_FIX_INTENDED_BEHAVIOR" => 10,
        "DUPLICATE" => 11,
        _ => 0,
    }
}

pub async fn handle(cmd: IssueCommand, server: &str, user: Option<&str>) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = IssueServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = IssueServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    match cmd {
        IssueCommand::Create {
            component,
            title,
            description,
            priority,
            r#type,
            severity,
            assignee,
            reporter,
        } => {
            let response = call!(create_issue, CreateIssueRequest {
                component_id: component,
                title,
                description,
                priority: parse_priority(&priority),
                r#type: parse_issue_type(&r#type),
                severity: severity.map(|s| parse_severity(&s)),
                assignee,
                reporter,
                verifier: None,
                found_in: None,
                targeted_to: None,
            })?;
            output::print_issue(&response.into_inner());
        }
        IssueCommand::Get { id } => {
            let response = call!(get_issue, GetIssueRequest { issue_id: id })?;
            output::print_issue(&response.into_inner());
        }
        IssueCommand::List {
            component,
            status,
            page_size,
            page_token,
        } => {
            let response = call!(list_issues, ListIssuesRequest {
                component_id: component,
                status_filter: status,
                page_size,
                page_token,
            })?;
            let resp = response.into_inner();
            output::print_issues(&resp.issues);
            if !resp.next_page_token.is_empty() {
                println!("Next page token: {}", resp.next_page_token);
            }
        }
        IssueCommand::Update {
            id,
            title,
            description,
            status,
            priority,
            severity,
            r#type,
            assignee,
            component,
        } => {
            let response = call!(update_issue, UpdateIssueRequest {
                issue_id: id,
                title,
                description,
                status: status.map(|s| parse_status(&s)),
                priority: priority.map(|p| parse_priority(&p)),
                severity: severity.map(|s| parse_severity(&s)),
                r#type: r#type.map(|t| parse_issue_type(&t)),
                component_id: component,
                assignee,
                reporter: None,
                verifier: None,
                found_in: None,
                targeted_to: None,
                verified_in: None,
                in_prod: None,
                archived: None,
                update_mask: None,
            })?;
            output::print_issue(&response.into_inner());
        }
        IssueCommand::AddParent {
            child_id,
            parent_id,
        } => {
            call!(add_parent, AddParentRequest {
                child_id,
                parent_id,
            })?;
            println!("Parent relationship added: {} -> {}", child_id, parent_id);
        }
        IssueCommand::RemoveParent {
            child_id,
            parent_id,
        } => {
            call!(remove_parent, RemoveParentRequest {
                child_id,
                parent_id,
            })?;
            println!(
                "Parent relationship removed: {} -> {}",
                child_id, parent_id
            );
        }
        IssueCommand::Parents { id } => {
            let response = call!(list_parents, ListRelatedIssuesRequest { issue_id: id })?;
            let resp = response.into_inner();
            if resp.issues.is_empty() {
                println!("No parents found.");
            } else {
                output::print_issues(&resp.issues);
            }
        }
        IssueCommand::Children { id } => {
            let response = call!(list_children, ListRelatedIssuesRequest { issue_id: id })?;
            let resp = response.into_inner();
            if resp.issues.is_empty() {
                println!("No children found.");
            } else {
                output::print_issues(&resp.issues);
            }
        }
        IssueCommand::Block {
            blocking_id,
            blocked_id,
        } => {
            call!(add_blocking, AddBlockingRequest {
                blocking_id,
                blocked_id,
            })?;
            println!(
                "Blocking relationship added: {} blocks {}",
                blocking_id, blocked_id
            );
        }
        IssueCommand::Unblock {
            blocking_id,
            blocked_id,
        } => {
            call!(remove_blocking, RemoveBlockingRequest {
                blocking_id,
                blocked_id,
            })?;
            println!(
                "Blocking relationship removed: {} no longer blocks {}",
                blocking_id, blocked_id
            );
        }
        IssueCommand::Duplicate { id, of } => {
            let response = call!(mark_duplicate, MarkDuplicateRequest {
                issue_id: id,
                canonical_id: of,
            })?;
            output::print_issue(&response.into_inner());
        }
        IssueCommand::Unduplicate { id } => {
            let response = call!(unmark_duplicate, UnmarkDuplicateRequest { issue_id: id })?;
            output::print_issue(&response.into_inner());
        }
    }

    Ok(())
}
