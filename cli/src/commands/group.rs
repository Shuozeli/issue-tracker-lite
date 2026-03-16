use anyhow::{bail, Result};
use clap::Subcommand;
use comfy_table::Table;

use crate::identity_proto::group_service_client::GroupServiceClient;
use crate::identity_proto::{
    AddMemberRequest, CreateGroupRequest, DeleteGroupRequest, GetGroupRequest, IsMemberRequest,
    ListGroupsRequest, ListMembersRequest, MemberRole, MemberType, RemoveMemberRequest,
    ResolveUserGroupsRequest, UpdateGroupRequest, UpdateMemberRoleRequest,
};

#[derive(Subcommand)]
pub enum GroupCommand {
    /// Create a new group
    Create {
        /// Group name (unique identifier)
        name: String,
        /// Display name
        #[arg(long)]
        display_name: String,
        /// Description
        #[arg(long, default_value = "")]
        description: String,
    },
    /// Get a group by name
    Get {
        /// Group name
        name: String,
    },
    /// List all groups
    List {
        /// Page size
        #[arg(short = 's', long, default_value = "50")]
        page_size: i32,
        /// Page token for pagination
        #[arg(short = 't', long, default_value = "")]
        page_token: String,
    },
    /// Update a group
    Update {
        /// Group name
        name: String,
        /// New display name
        #[arg(long)]
        display_name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a group
    Delete {
        /// Group name
        name: String,
    },
    /// Add a member to a group
    AddMember {
        /// Group name
        group_name: String,
        /// Member type: "user" or "group"
        #[arg(long)]
        member_type: String,
        /// Member value (user ID or group name)
        #[arg(long)]
        member_value: String,
        /// Role: "member", "manager", or "owner"
        #[arg(long, default_value = "member")]
        role: String,
    },
    /// Remove a member from a group
    RemoveMember {
        /// Group name
        group_name: String,
        /// Member type: "user" or "group"
        #[arg(long)]
        member_type: String,
        /// Member value (user ID or group name)
        #[arg(long)]
        member_value: String,
    },
    /// List members of a group
    ListMembers {
        /// Group name
        group_name: String,
    },
    /// Update a member's role in a group
    UpdateMemberRole {
        /// Group name
        group_name: String,
        /// Member type: "user" or "group"
        #[arg(long)]
        member_type: String,
        /// Member value (user ID or group name)
        #[arg(long)]
        member_value: String,
        /// New role: "member", "manager", or "owner"
        #[arg(long)]
        role: String,
    },
    /// Resolve all groups a user belongs to (transitive)
    ResolveGroups {
        /// User ID
        user_id: String,
    },
    /// Check if a user is a member of a group
    IsMember {
        /// User ID
        user_id: String,
        /// Group name
        group_name: String,
    },
}

fn parse_member_type(s: &str) -> Result<i32> {
    match s.to_lowercase().as_str() {
        "user" => Ok(MemberType::User as i32),
        "group" => Ok(MemberType::Group as i32),
        _ => bail!("Invalid member_type '{}'. Must be 'user' or 'group'.", s),
    }
}

fn parse_role(s: &str) -> Result<i32> {
    match s.to_lowercase().as_str() {
        "member" => Ok(MemberRole::Member as i32),
        "manager" => Ok(MemberRole::Manager as i32),
        "owner" => Ok(MemberRole::Owner as i32),
        _ => bail!("Invalid role '{}'. Must be 'member', 'manager', or 'owner'.", s),
    }
}

fn member_type_str(val: i32) -> &'static str {
    match MemberType::try_from(val) {
        Ok(MemberType::User) => "user",
        Ok(MemberType::Group) => "group",
        Ok(MemberType::Unspecified) => "unspecified",
        Err(_) => "unknown",
    }
}

fn role_str(val: i32) -> &'static str {
    match MemberRole::try_from(val) {
        Ok(MemberRole::Member) => "member",
        Ok(MemberRole::Manager) => "manager",
        Ok(MemberRole::Owner) => "owner",
        Ok(MemberRole::Unspecified) => "unspecified",
        Err(_) => "unknown",
    }
}

fn print_group(g: &crate::identity_proto::Group) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec!["Name", &g.name]);
    table.add_row(vec!["Display Name", &g.display_name]);
    table.add_row(vec!["Description", &g.description]);
    table.add_row(vec!["Creator", &g.creator]);
    let create_time = g
        .create_time
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_default();
    let update_time = g
        .update_time
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_default();
    table.add_row(vec!["Create Time", &create_time]);
    table.add_row(vec!["Update Time", &update_time]);
    println!("{table}");
}

fn print_groups(groups: &[crate::identity_proto::Group]) {
    let mut table = Table::new();
    table.set_header(vec!["Name", "Display Name", "Description", "Creator"]);
    for g in groups {
        table.add_row(vec![&g.name, &g.display_name, &g.description, &g.creator]);
    }
    println!("{table}");
}

fn print_member(m: &crate::identity_proto::GroupMember) {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec!["Group", &m.group_name]);
    table.add_row(vec!["Member Type", member_type_str(m.member_type)]);
    table.add_row(vec!["Member Value", &m.member_value]);
    table.add_row(vec!["Role", role_str(m.role)]);
    table.add_row(vec!["Added By", &m.added_by]);
    let create_time = m
        .create_time
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_default();
    table.add_row(vec!["Create Time", &create_time]);
    println!("{table}");
}

fn print_members(members: &[crate::identity_proto::GroupMember]) {
    let mut table = Table::new();
    table.set_header(vec![
        "Group",
        "Type",
        "Value",
        "Role",
        "Added By",
    ]);
    for m in members {
        table.add_row(vec![
            &m.group_name,
            member_type_str(m.member_type),
            &m.member_value,
            role_str(m.role),
            &m.added_by,
        ]);
    }
    println!("{table}");
}

pub async fn handle(cmd: GroupCommand, server: &str, user: Option<&str>) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    // Use a macro to avoid repeating the interceptor plumbing.
    // If user is set, wrap calls with the interceptor; otherwise use plain channel.
    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = GroupServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = GroupServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel; // suppress unused warning

    match cmd {
        GroupCommand::Create {
            name,
            display_name,
            description,
        } => {
            let response = call!(create_group, CreateGroupRequest {
                name,
                display_name,
                description,
            })?;
            print_group(&response.into_inner());
        }
        GroupCommand::Get { name } => {
            let response = call!(get_group, GetGroupRequest { name })?;
            print_group(&response.into_inner());
        }
        GroupCommand::List {
            page_size,
            page_token,
        } => {
            let response = call!(list_groups, ListGroupsRequest {
                page_size,
                page_token,
            })?;
            let resp = response.into_inner();
            print_groups(&resp.groups);
            if !resp.next_page_token.is_empty() {
                println!("Next page token: {}", resp.next_page_token);
            }
        }
        GroupCommand::Update {
            name,
            display_name,
            description,
        } => {
            let response = call!(update_group, UpdateGroupRequest {
                name,
                display_name,
                description,
            })?;
            print_group(&response.into_inner());
        }
        GroupCommand::Delete { name } => {
            call!(delete_group, DeleteGroupRequest { name: name.clone() })?;
            println!("Group '{}' deleted.", name);
        }
        GroupCommand::AddMember {
            group_name,
            member_type,
            member_value,
            role,
        } => {
            let mt = parse_member_type(&member_type)?;
            let r = parse_role(&role)?;
            let response = call!(add_member, AddMemberRequest {
                group_name,
                member_type: mt,
                member_value,
                role: r,
            })?;
            print_member(&response.into_inner());
        }
        GroupCommand::RemoveMember {
            group_name,
            member_type,
            member_value,
        } => {
            let mt = parse_member_type(&member_type)?;
            call!(remove_member, RemoveMemberRequest {
                group_name: group_name.clone(),
                member_type: mt,
                member_value: member_value.clone(),
            })?;
            println!(
                "Removed {} '{}' from group '{}'.",
                member_type, member_value, group_name
            );
        }
        GroupCommand::ListMembers { group_name } => {
            let response = call!(list_members, ListMembersRequest { group_name })?;
            print_members(&response.into_inner().members);
        }
        GroupCommand::UpdateMemberRole {
            group_name,
            member_type,
            member_value,
            role,
        } => {
            let mt = parse_member_type(&member_type)?;
            let r = parse_role(&role)?;
            let response = call!(update_member_role, UpdateMemberRoleRequest {
                group_name,
                member_type: mt,
                member_value,
                role: r,
            })?;
            print_member(&response.into_inner());
        }
        GroupCommand::ResolveGroups { user_id } => {
            let response = call!(resolve_user_groups, ResolveUserGroupsRequest { user_id })?;
            let groups = response.into_inner().groups;
            if groups.is_empty() {
                println!("User is not a member of any groups.");
            } else {
                let mut table = Table::new();
                table.set_header(vec!["Group Name"]);
                for g in &groups {
                    table.add_row(vec![g.as_str()]);
                }
                println!("{table}");
            }
        }
        GroupCommand::IsMember {
            user_id,
            group_name,
        } => {
            let response = call!(is_member, IsMemberRequest {
                user_id: user_id.clone(),
                group_name: group_name.clone(),
            })?;
            let is_member = response.into_inner().is_member;
            if is_member {
                println!("User '{}' IS a member of group '{}'.", user_id, group_name);
            } else {
                println!(
                    "User '{}' is NOT a member of group '{}'.",
                    user_id, group_name
                );
            }
        }
    }

    Ok(())
}
