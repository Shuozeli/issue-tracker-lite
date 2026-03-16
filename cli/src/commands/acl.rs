use anyhow::{bail, Result};
use clap::Subcommand;

use crate::output;
use crate::proto::acl_service_client::AclServiceClient;
use crate::proto::{
    CheckComponentPermissionRequest, GetComponentAclRequest, GetHotlistAclRequest,
    RemoveComponentAclRequest, RemoveHotlistAclRequest, SetComponentAclRequest,
    SetHotlistAclRequest,
};

#[derive(Subcommand)]
pub enum AclCommand {
    /// Set component ACL for an identity
    #[command(name = "set-component")]
    SetComponent {
        /// Component ID
        component_id: i64,
        /// Identity type: user, group, public
        #[arg(long)]
        identity_type: String,
        /// Identity value (email, group name, or "*" for public)
        #[arg(long)]
        identity_value: String,
        /// Comma-separated permissions: VIEW_ISSUES,COMMENT_ON_ISSUES,EDIT_ISSUES,ADMIN_ISSUES,CREATE_ISSUES,VIEW_COMPONENTS,ADMIN_COMPONENTS,VIEW_RESTRICTED,VIEW_RESTRICTED_PLUS
        #[arg(long)]
        permissions: String,
    },
    /// Get all ACL entries for a component
    #[command(name = "get-component")]
    GetComponent {
        /// Component ID
        component_id: i64,
    },
    /// Remove a component ACL entry
    #[command(name = "remove-component")]
    RemoveComponent {
        /// Component ID
        component_id: i64,
        /// Identity type: user, group, public
        #[arg(long)]
        identity_type: String,
        /// Identity value
        #[arg(long)]
        identity_value: String,
    },
    /// Check effective permissions for a user on a component
    #[command(name = "check")]
    Check {
        /// Component ID
        component_id: i64,
        /// User email to check
        #[arg(long)]
        user: String,
        /// Optional issue ID (for expanded access evaluation)
        #[arg(long)]
        issue_id: Option<i64>,
    },
    /// Set hotlist ACL for an identity
    #[command(name = "set-hotlist")]
    SetHotlist {
        /// Hotlist ID
        hotlist_id: i64,
        /// Identity type: user, group, public
        #[arg(long)]
        identity_type: String,
        /// Identity value
        #[arg(long)]
        identity_value: String,
        /// Permission: HOTLIST_VIEW, HOTLIST_VIEW_APPEND, HOTLIST_ADMIN
        #[arg(long)]
        permission: String,
    },
    /// Get all ACL entries for a hotlist
    #[command(name = "get-hotlist")]
    GetHotlist {
        /// Hotlist ID
        hotlist_id: i64,
    },
    /// Remove a hotlist ACL entry
    #[command(name = "remove-hotlist")]
    RemoveHotlist {
        /// Hotlist ID
        hotlist_id: i64,
        /// Identity type: user, group, public
        #[arg(long)]
        identity_type: String,
        /// Identity value
        #[arg(long)]
        identity_value: String,
    },
}

fn parse_identity_type(s: &str) -> Result<i32> {
    match s.to_uppercase().as_str() {
        "USER" => Ok(1),
        "GROUP" => Ok(2),
        "PUBLIC" => Ok(3),
        other => bail!("unknown identity type '{}': expected user, group, or public", other),
    }
}

fn parse_component_permission(s: &str) -> Result<i32> {
    match s.trim().to_uppercase().as_str() {
        "VIEW_ISSUES" => Ok(1),
        "COMMENT_ON_ISSUES" => Ok(2),
        "EDIT_ISSUES" => Ok(3),
        "ADMIN_ISSUES" => Ok(4),
        "CREATE_ISSUES" => Ok(5),
        "VIEW_COMPONENTS" => Ok(6),
        "ADMIN_COMPONENTS" => Ok(7),
        "VIEW_RESTRICTED" => Ok(8),
        "VIEW_RESTRICTED_PLUS" => Ok(9),
        other => bail!("unknown component permission '{}'", other),
    }
}

fn parse_hotlist_permission(s: &str) -> Result<i32> {
    match s.trim().to_uppercase().as_str() {
        "HOTLIST_VIEW" => Ok(1),
        "HOTLIST_VIEW_APPEND" => Ok(2),
        "HOTLIST_ADMIN" => Ok(3),
        other => bail!("unknown hotlist permission '{}': expected HOTLIST_VIEW, HOTLIST_VIEW_APPEND, or HOTLIST_ADMIN", other),
    }
}

pub async fn handle(cmd: AclCommand, server: &str, user: Option<&str>) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = AclServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = AclServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    match cmd {
        AclCommand::SetComponent {
            component_id,
            identity_type,
            identity_value,
            permissions,
        } => {
            let id_type = parse_identity_type(&identity_type)?;
            let perms: Result<Vec<i32>> = permissions
                .split(',')
                .map(|s| parse_component_permission(s))
                .collect();
            let perms = perms?;

            let resp = call!(set_component_acl, SetComponentAclRequest {
                component_id,
                identity_type: id_type,
                identity_value,
                permissions: perms,
            })?;
            output::print_component_acl_entry(&resp.into_inner());
        }
        AclCommand::GetComponent { component_id } => {
            let resp = call!(get_component_acl, GetComponentAclRequest { component_id })?;
            output::print_component_acl_entries(&resp.into_inner().entries);
        }
        AclCommand::RemoveComponent {
            component_id,
            identity_type,
            identity_value,
        } => {
            let id_type = parse_identity_type(&identity_type)?;
            call!(remove_component_acl, RemoveComponentAclRequest {
                component_id,
                identity_type: id_type,
                identity_value,
            })?;
            println!("ACL entry removed.");
        }
        AclCommand::Check {
            component_id,
            user: check_user,
            issue_id,
        } => {
            let resp = call!(check_component_permission, CheckComponentPermissionRequest {
                component_id,
                user_id: check_user,
                issue_id,
            })?;
            output::print_permission_check(&resp.into_inner());
        }
        AclCommand::SetHotlist {
            hotlist_id,
            identity_type,
            identity_value,
            permission,
        } => {
            let id_type = parse_identity_type(&identity_type)?;
            let perm = parse_hotlist_permission(&permission)?;

            let resp = call!(set_hotlist_acl, SetHotlistAclRequest {
                hotlist_id,
                identity_type: id_type,
                identity_value,
                permission: perm,
            })?;
            output::print_hotlist_acl_entry(&resp.into_inner());
        }
        AclCommand::GetHotlist { hotlist_id } => {
            let resp = call!(get_hotlist_acl, GetHotlistAclRequest { hotlist_id })?;
            output::print_hotlist_acl_entries(&resp.into_inner().entries);
        }
        AclCommand::RemoveHotlist {
            hotlist_id,
            identity_type,
            identity_value,
        } => {
            let id_type = parse_identity_type(&identity_type)?;
            call!(remove_hotlist_acl, RemoveHotlistAclRequest {
                hotlist_id,
                identity_type: id_type,
                identity_value,
            })?;
            println!("Hotlist ACL entry removed.");
        }
    }

    Ok(())
}
