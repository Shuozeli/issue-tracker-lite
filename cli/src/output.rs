use comfy_table::{Cell, Table};

use crate::proto::{
    CheckComponentPermissionResponse, Comment, Component, ComponentAclEntry, Hotlist,
    HotlistAclEntry, HotlistIssue, Issue,
};

fn format_timestamp(ts: &Option<prost_types::Timestamp>) -> String {
    match ts {
        Some(t) => {
            let dt = chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32);
            match dt {
                Some(d) => d.format("%Y-%m-%d %H:%M:%S").to_string(),
                None => "-".to_string(),
            }
        }
        None => "-".to_string(),
    }
}

// --- Component output ---

pub fn print_component(c: &Component) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![Cell::new("ID"), Cell::new(c.component_id)]);
    table.add_row(vec![Cell::new("Name"), Cell::new(&c.name)]);
    table.add_row(vec![Cell::new("Description"), Cell::new(&c.description)]);
    table.add_row(vec![
        Cell::new("Parent ID"),
        Cell::new(
            c.parent_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
    ]);
    table.add_row(vec![Cell::new("Child Count"), Cell::new(c.child_count)]);
    table.add_row(vec![
        Cell::new("Expanded Access"),
        Cell::new(c.expanded_access_enabled),
    ]);
    table.add_row(vec![
        Cell::new("Editable Comments"),
        Cell::new(c.editable_comments_enabled),
    ]);
    table.add_row(vec![
        Cell::new("Created"),
        Cell::new(format_timestamp(&c.create_time)),
    ]);
    table.add_row(vec![
        Cell::new("Updated"),
        Cell::new(format_timestamp(&c.update_time)),
    ]);
    println!("{table}");
}

pub fn print_components(components: &[Component]) {
    if components.is_empty() {
        println!("No components found.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec![
        "ID",
        "Name",
        "Description",
        "Parent",
        "Children",
        "Created",
    ]);

    for c in components {
        table.add_row(vec![
            Cell::new(c.component_id),
            Cell::new(&c.name),
            Cell::new(&c.description),
            Cell::new(
                c.parent_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::new(c.child_count),
            Cell::new(format_timestamp(&c.create_time)),
        ]);
    }

    println!("{table}");
}

// --- Issue output ---

fn status_name(val: i32) -> &'static str {
    match val {
        1 => "NEW",
        2 => "ASSIGNED",
        3 => "IN_PROGRESS",
        4 => "INACTIVE",
        5 => "FIXED",
        6 => "FIXED_VERIFIED",
        7 => "WONT_FIX_INFEASIBLE",
        8 => "WONT_FIX_NOT_REPRODUCIBLE",
        9 => "WONT_FIX_OBSOLETE",
        10 => "WONT_FIX_INTENDED_BEHAVIOR",
        11 => "DUPLICATE",
        _ => "UNKNOWN",
    }
}

fn priority_name(val: i32) -> &'static str {
    match val {
        1 => "P0",
        2 => "P1",
        3 => "P2",
        4 => "P3",
        5 => "P4",
        _ => "UNKNOWN",
    }
}

fn severity_name(val: i32) -> &'static str {
    match val {
        1 => "S0",
        2 => "S1",
        3 => "S2",
        4 => "S3",
        5 => "S4",
        _ => "UNKNOWN",
    }
}

fn issue_type_name(val: i32) -> &'static str {
    match val {
        1 => "BUG",
        2 => "FEATURE_REQUEST",
        3 => "CUSTOMER_ISSUE",
        4 => "INTERNAL_CLEANUP",
        5 => "PROCESS",
        6 => "VULNERABILITY",
        7 => "PRIVACY_ISSUE",
        8 => "PROGRAM",
        9 => "PROJECT",
        10 => "FEATURE",
        11 => "MILESTONE",
        12 => "EPIC",
        13 => "STORY",
        14 => "TASK",
        _ => "UNKNOWN",
    }
}

pub fn print_issue(i: &Issue) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![Cell::new("ID"), Cell::new(i.issue_id)]);
    table.add_row(vec![Cell::new("Title"), Cell::new(&i.title)]);
    table.add_row(vec![Cell::new("Status"), Cell::new(status_name(i.status))]);
    table.add_row(vec![
        Cell::new("Priority"),
        Cell::new(priority_name(i.priority)),
    ]);
    table.add_row(vec![
        Cell::new("Severity"),
        Cell::new(severity_name(i.severity)),
    ]);
    table.add_row(vec![
        Cell::new("Type"),
        Cell::new(issue_type_name(i.r#type)),
    ]);
    table.add_row(vec![Cell::new("Component"), Cell::new(i.component_id)]);
    table.add_row(vec![
        Cell::new("Assignee"),
        Cell::new(if i.assignee.is_empty() {
            "-"
        } else {
            &i.assignee
        }),
    ]);
    table.add_row(vec![
        Cell::new("Reporter"),
        Cell::new(if i.reporter.is_empty() {
            "-"
        } else {
            &i.reporter
        }),
    ]);
    if !i.description.is_empty() {
        table.add_row(vec![Cell::new("Description"), Cell::new(&i.description)]);
    }
    table.add_row(vec![
        Cell::new("Created"),
        Cell::new(format_timestamp(&i.create_time)),
    ]);
    table.add_row(vec![
        Cell::new("Modified"),
        Cell::new(format_timestamp(&i.modify_time)),
    ]);
    if i.resolve_time.is_some() {
        table.add_row(vec![
            Cell::new("Resolved"),
            Cell::new(format_timestamp(&i.resolve_time)),
        ]);
    }
    if i.verify_time.is_some() {
        table.add_row(vec![
            Cell::new("Verified"),
            Cell::new(format_timestamp(&i.verify_time)),
        ]);
    }
    println!("{table}");
}

pub fn print_issues(issues: &[Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec![
        "ID", "Title", "Status", "Priority", "Type", "Assignee", "Modified",
    ]);

    for i in issues {
        table.add_row(vec![
            Cell::new(i.issue_id),
            Cell::new(&i.title),
            Cell::new(status_name(i.status)),
            Cell::new(priority_name(i.priority)),
            Cell::new(issue_type_name(i.r#type)),
            Cell::new(if i.assignee.is_empty() {
                "-"
            } else {
                &i.assignee
            }),
            Cell::new(format_timestamp(&i.modify_time)),
        ]);
    }

    println!("{table}");
}

// --- Comment output ---

pub fn print_comment(c: &Comment) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![Cell::new("Comment ID"), Cell::new(c.comment_id)]);
    table.add_row(vec![Cell::new("Issue ID"), Cell::new(c.issue_id)]);
    table.add_row(vec![Cell::new("Author"), Cell::new(&c.author)]);
    table.add_row(vec![
        Cell::new("Description"),
        Cell::new(if c.is_description { "yes" } else { "no" }),
    ]);
    table.add_row(vec![Cell::new("Body"), Cell::new(&c.body)]);
    table.add_row(vec![
        Cell::new("Created"),
        Cell::new(format_timestamp(&c.create_time)),
    ]);
    if c.modify_time.is_some() {
        table.add_row(vec![
            Cell::new("Modified"),
            Cell::new(format_timestamp(&c.modify_time)),
        ]);
    }
    println!("{table}");
}

// --- Hotlist output ---

pub fn print_hotlist(h: &Hotlist) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![Cell::new("ID"), Cell::new(h.hotlist_id)]);
    table.add_row(vec![Cell::new("Name"), Cell::new(&h.name)]);
    table.add_row(vec![Cell::new("Description"), Cell::new(&h.description)]);
    table.add_row(vec![
        Cell::new("Owner"),
        Cell::new(if h.owner.is_empty() { "-" } else { &h.owner }),
    ]);
    table.add_row(vec![Cell::new("Archived"), Cell::new(h.archived)]);
    table.add_row(vec![Cell::new("Issues"), Cell::new(h.issue_count)]);
    table.add_row(vec![
        Cell::new("Created"),
        Cell::new(format_timestamp(&h.create_time)),
    ]);
    table.add_row(vec![
        Cell::new("Modified"),
        Cell::new(format_timestamp(&h.modify_time)),
    ]);
    println!("{table}");
}

pub fn print_hotlists(hotlists: &[Hotlist]) {
    if hotlists.is_empty() {
        println!("No hotlists found.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec![
        "ID", "Name", "Owner", "Issues", "Archived", "Modified",
    ]);

    for h in hotlists {
        table.add_row(vec![
            Cell::new(h.hotlist_id),
            Cell::new(&h.name),
            Cell::new(if h.owner.is_empty() { "-" } else { &h.owner }),
            Cell::new(h.issue_count),
            Cell::new(if h.archived { "yes" } else { "no" }),
            Cell::new(format_timestamp(&h.modify_time)),
        ]);
    }

    println!("{table}");
}

pub fn print_hotlist_issues(issues: &[HotlistIssue]) {
    if issues.is_empty() {
        println!("No issues in hotlist.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["Position", "Issue ID", "Added By", "Added At"]);

    for hi in issues {
        table.add_row(vec![
            Cell::new(hi.position),
            Cell::new(hi.issue_id),
            Cell::new(if hi.added_by.is_empty() {
                "-"
            } else {
                &hi.added_by
            }),
            Cell::new(format_timestamp(&hi.add_time)),
        ]);
    }

    println!("{table}");
}

// --- ACL output ---

fn identity_type_name(val: i32) -> &'static str {
    match val {
        1 => "USER",
        2 => "GROUP",
        3 => "PUBLIC",
        _ => "UNKNOWN",
    }
}

fn component_permission_name(val: i32) -> &'static str {
    match val {
        1 => "VIEW_ISSUES",
        2 => "COMMENT_ON_ISSUES",
        3 => "EDIT_ISSUES",
        4 => "ADMIN_ISSUES",
        5 => "CREATE_ISSUES",
        6 => "VIEW_COMPONENTS",
        7 => "ADMIN_COMPONENTS",
        8 => "VIEW_RESTRICTED",
        9 => "VIEW_RESTRICTED_PLUS",
        _ => "UNKNOWN",
    }
}

fn hotlist_permission_name(val: i32) -> &'static str {
    match val {
        1 => "HOTLIST_VIEW",
        2 => "HOTLIST_VIEW_APPEND",
        3 => "HOTLIST_ADMIN",
        _ => "UNKNOWN",
    }
}

pub fn print_component_acl_entry(e: &ComponentAclEntry) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![Cell::new("Component ID"), Cell::new(e.component_id)]);
    table.add_row(vec![
        Cell::new("Identity Type"),
        Cell::new(identity_type_name(e.identity_type)),
    ]);
    table.add_row(vec![
        Cell::new("Identity Value"),
        Cell::new(&e.identity_value),
    ]);
    let perms: Vec<&str> = e
        .permissions
        .iter()
        .map(|p| component_permission_name(*p))
        .collect();
    table.add_row(vec![Cell::new("Permissions"), Cell::new(perms.join(", "))]);
    table.add_row(vec![
        Cell::new("Created"),
        Cell::new(format_timestamp(&e.create_time)),
    ]);
    println!("{table}");
}

pub fn print_component_acl_entries(entries: &[ComponentAclEntry]) {
    if entries.is_empty() {
        println!("No ACL entries found.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec![
        "Identity Type",
        "Identity Value",
        "Permissions",
        "Created",
    ]);

    for e in entries {
        let perms: Vec<&str> = e
            .permissions
            .iter()
            .map(|p| component_permission_name(*p))
            .collect();
        table.add_row(vec![
            Cell::new(identity_type_name(e.identity_type)),
            Cell::new(&e.identity_value),
            Cell::new(perms.join(", ")),
            Cell::new(format_timestamp(&e.create_time)),
        ]);
    }

    println!("{table}");
}

pub fn print_hotlist_acl_entry(e: &HotlistAclEntry) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![Cell::new("Hotlist ID"), Cell::new(e.hotlist_id)]);
    table.add_row(vec![
        Cell::new("Identity Type"),
        Cell::new(identity_type_name(e.identity_type)),
    ]);
    table.add_row(vec![
        Cell::new("Identity Value"),
        Cell::new(&e.identity_value),
    ]);
    table.add_row(vec![
        Cell::new("Permission"),
        Cell::new(hotlist_permission_name(e.permission)),
    ]);
    table.add_row(vec![
        Cell::new("Created"),
        Cell::new(format_timestamp(&e.create_time)),
    ]);
    println!("{table}");
}

pub fn print_hotlist_acl_entries(entries: &[HotlistAclEntry]) {
    if entries.is_empty() {
        println!("No ACL entries found.");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec![
        "Identity Type",
        "Identity Value",
        "Permission",
        "Created",
    ]);

    for e in entries {
        table.add_row(vec![
            Cell::new(identity_type_name(e.identity_type)),
            Cell::new(&e.identity_value),
            Cell::new(hotlist_permission_name(e.permission)),
            Cell::new(format_timestamp(&e.create_time)),
        ]);
    }

    println!("{table}");
}

pub fn print_permission_check(resp: &CheckComponentPermissionResponse) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec![
        Cell::new("Grant Source"),
        Cell::new(&resp.grant_source),
    ]);
    let perms: Vec<&str> = resp
        .permissions
        .iter()
        .map(|p| component_permission_name(*p))
        .collect();
    table.add_row(vec![
        Cell::new("Effective Permissions"),
        Cell::new(if perms.is_empty() {
            "none".to_string()
        } else {
            perms.join(", ")
        }),
    ]);
    println!("{table}");
}

pub fn print_comments(comments: &[Comment]) {
    if comments.is_empty() {
        println!("No comments found.");
        return;
    }

    for c in comments {
        let desc_marker = if c.is_description {
            " [description]"
        } else {
            ""
        };
        let author = if c.author.is_empty() { "-" } else { &c.author };
        println!(
            "#{} by {} at {}{}\n{}\n",
            c.comment_id,
            author,
            format_timestamp(&c.create_time),
            desc_marker,
            c.body
        );
    }
}
