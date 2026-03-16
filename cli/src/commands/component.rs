use anyhow::Result;
use clap::Subcommand;

use crate::output;
use crate::proto::component_service_client::ComponentServiceClient;
use crate::proto::{
    CreateComponentRequest, DeleteComponentRequest, GetComponentRequest, ListComponentsRequest,
    UpdateComponentRequest,
};

#[derive(Subcommand)]
pub enum ComponentCommand {
    /// Create a new component
    Create {
        /// Component name
        name: String,
        /// Description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Parent component ID
        #[arg(short, long)]
        parent_id: Option<i64>,
    },
    /// Get a component by ID
    Get {
        /// Component ID
        id: i64,
    },
    /// List components
    List {
        /// Filter by parent ID (omit for root components)
        #[arg(short, long)]
        parent_id: Option<i64>,
        /// Page size
        #[arg(short = 's', long, default_value = "50")]
        page_size: i32,
        /// Page token for pagination
        #[arg(short = 't', long, default_value = "")]
        page_token: String,
    },
    /// Update a component
    Update {
        /// Component ID
        id: i64,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// New parent ID
        #[arg(short, long)]
        parent_id: Option<i64>,
    },
    /// Delete a component
    Delete {
        /// Component ID
        id: i64,
    },
}

pub async fn handle(cmd: ComponentCommand, server: &str, user: Option<&str>) -> Result<()> {
    let channel = tonic::transport::Channel::from_shared(server.to_string())?
        .connect()
        .await?;

    macro_rules! call {
        ($method:ident, $req:expr) => {
            if let Some(uid) = user {
                let mut c = ComponentServiceClient::with_interceptor(
                    channel.clone(),
                    crate::auth::UserInterceptor::new(uid.to_string()),
                );
                c.$method($req).await
            } else {
                let mut c = ComponentServiceClient::new(channel.clone());
                c.$method($req).await
            }
        };
    }

    let _ = &channel;

    match cmd {
        ComponentCommand::Create {
            name,
            description,
            parent_id,
        } => {
            let response = call!(
                create_component,
                CreateComponentRequest {
                    name,
                    description,
                    parent_id,
                }
            )?;
            output::print_component(&response.into_inner());
        }
        ComponentCommand::Get { id } => {
            let response = call!(get_component, GetComponentRequest { component_id: id })?;
            output::print_component(&response.into_inner());
        }
        ComponentCommand::List {
            parent_id,
            page_size,
            page_token,
        } => {
            let response = call!(
                list_components,
                ListComponentsRequest {
                    parent_id,
                    page_size,
                    page_token,
                }
            )?;
            let resp = response.into_inner();
            output::print_components(&resp.components);
            if !resp.next_page_token.is_empty() {
                println!("Next page token: {}", resp.next_page_token);
            }
        }
        ComponentCommand::Update {
            id,
            name,
            description,
            parent_id,
        } => {
            let response = call!(
                update_component,
                UpdateComponentRequest {
                    component_id: id,
                    name,
                    description,
                    parent_id,
                    update_mask: None,
                    ..Default::default()
                }
            )?;
            output::print_component(&response.into_inner());
        }
        ComponentCommand::Delete { id } => {
            call!(
                delete_component,
                DeleteComponentRequest { component_id: id }
            )?;
            println!("Component {} deleted.", id);
        }
    }

    Ok(())
}
