use std::sync::Arc;

use tonic::{Request, Response, Status};

use identity::{IdentityError, IdentityProvider, MemberRole, MemberType};

use crate::domain::permissions;
use crate::identity_proto::group_service_server::GroupService;
use crate::identity_proto::{
    AddMemberRequest, BatchAddMembersRequest, BatchAddMembersResponse, CreateGroupRequest,
    DeleteGroupRequest, DeleteGroupResponse, GetGroupRequest, Group as ProtoGroup,
    GroupMember as ProtoGroupMember, IsMemberRequest, IsMemberResponse, ListGroupsRequest,
    ListGroupsResponse, ListMembersRequest, ListMembersResponse, RemoveMemberRequest,
    RemoveMemberResponse, ResolveUserGroupsRequest, ResolveUserGroupsResponse, UpdateGroupRequest,
    UpdateMemberRoleRequest,
};

pub struct GroupServiceImpl {
    pub identity: Arc<dyn IdentityProvider>,
}

fn identity_error_to_status(e: IdentityError) -> Status {
    match e {
        IdentityError::NotFound(msg) => Status::not_found(msg),
        IdentityError::InvalidArgument(msg) => Status::invalid_argument(msg),
        IdentityError::AlreadyExists(msg) => Status::already_exists(msg),
        IdentityError::FailedPrecondition(msg) => Status::failed_precondition(msg),
        IdentityError::PermissionDenied(msg) => Status::permission_denied(msg),
        IdentityError::Internal(msg) => Status::internal(msg),
    }
}

fn parse_timestamp(s: &str) -> Option<prost_types::Timestamp> {
    chrono::DateTime::parse_from_rfc3339(s)
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .ok()
        .map(|dt| prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
}

fn group_to_proto(g: &identity::Group) -> ProtoGroup {
    ProtoGroup {
        name: g.name.clone(),
        display_name: g.display_name.clone(),
        description: g.description.clone(),
        creator: g.creator.clone(),
        create_time: parse_timestamp(&g.created_at),
        update_time: parse_timestamp(&g.updated_at),
    }
}

fn member_to_proto(m: &identity::GroupMember, group_name: &str) -> ProtoGroupMember {
    ProtoGroupMember {
        group_name: group_name.to_string(),
        member_type: match m.member_type {
            MemberType::User => 1,
            MemberType::Group => 2,
        },
        member_value: m.member_value.clone(),
        role: match m.role {
            MemberRole::Member => 1,
            MemberRole::Manager => 2,
            MemberRole::Owner => 3,
        },
        added_by: m.added_by.clone(),
        create_time: parse_timestamp(&m.created_at),
    }
}

fn proto_member_type(val: i32) -> Option<MemberType> {
    match val {
        1 => Some(MemberType::User),
        2 => Some(MemberType::Group),
        _ => None,
    }
}

fn proto_member_role(val: i32) -> Option<MemberRole> {
    match val {
        1 => Some(MemberRole::Member),
        2 => Some(MemberRole::Manager),
        3 => Some(MemberRole::Owner),
        _ => None,
    }
}

#[tonic::async_trait]
impl GroupService for GroupServiceImpl {
    async fn create_group(
        &self,
        request: Request<CreateGroupRequest>,
    ) -> Result<Response<ProtoGroup>, Status> {
        let user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        let group = self
            .identity
            .create_group(&req.name, &req.display_name, &req.description, &user_id)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(group_to_proto(&group)))
    }

    async fn get_group(
        &self,
        request: Request<GetGroupRequest>,
    ) -> Result<Response<ProtoGroup>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        let group = self
            .identity
            .get_group(&req.name)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(group_to_proto(&group)))
    }

    async fn list_groups(
        &self,
        request: Request<ListGroupsRequest>,
    ) -> Result<Response<ListGroupsResponse>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        let (groups, next_token) = self
            .identity
            .list_groups(req.page_size, &req.page_token)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(ListGroupsResponse {
            groups: groups.iter().map(group_to_proto).collect(),
            next_page_token: next_token,
        }))
    }

    async fn update_group(
        &self,
        request: Request<UpdateGroupRequest>,
    ) -> Result<Response<ProtoGroup>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        let group = self
            .identity
            .update_group(
                &req.name,
                req.display_name.as_deref(),
                req.description.as_deref(),
            )
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(group_to_proto(&group)))
    }

    async fn delete_group(
        &self,
        request: Request<DeleteGroupRequest>,
    ) -> Result<Response<DeleteGroupResponse>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        self.identity
            .delete_group(&req.name)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(DeleteGroupResponse {}))
    }

    async fn add_member(
        &self,
        request: Request<AddMemberRequest>,
    ) -> Result<Response<ProtoGroupMember>, Status> {
        let user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.group_name.is_empty() {
            return Err(Status::invalid_argument("group_name is required"));
        }
        if req.member_value.is_empty() {
            return Err(Status::invalid_argument("member_value is required"));
        }
        let member_type = proto_member_type(req.member_type).ok_or_else(|| {
            Status::invalid_argument(format!("invalid member type: {}", req.member_type))
        })?;
        let role = proto_member_role(req.role).ok_or_else(|| {
            Status::invalid_argument(format!("invalid member role: {}", req.role))
        })?;
        let member = self
            .identity
            .add_member(
                &req.group_name,
                member_type,
                &req.member_value,
                role,
                &user_id,
            )
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(member_to_proto(&member, &req.group_name)))
    }

    async fn remove_member(
        &self,
        request: Request<RemoveMemberRequest>,
    ) -> Result<Response<RemoveMemberResponse>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.group_name.is_empty() {
            return Err(Status::invalid_argument("group_name is required"));
        }
        let member_type = proto_member_type(req.member_type).ok_or_else(|| {
            Status::invalid_argument(format!("invalid member type: {}", req.member_type))
        })?;
        self.identity
            .remove_member(&req.group_name, member_type, &req.member_value)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(RemoveMemberResponse {}))
    }

    async fn list_members(
        &self,
        request: Request<ListMembersRequest>,
    ) -> Result<Response<ListMembersResponse>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.group_name.is_empty() {
            return Err(Status::invalid_argument("group_name is required"));
        }
        let members = self
            .identity
            .list_members(&req.group_name)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(ListMembersResponse {
            members: members
                .iter()
                .map(|m| member_to_proto(m, &req.group_name))
                .collect(),
        }))
    }

    async fn update_member_role(
        &self,
        request: Request<UpdateMemberRoleRequest>,
    ) -> Result<Response<ProtoGroupMember>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.group_name.is_empty() {
            return Err(Status::invalid_argument("group_name is required"));
        }
        let member_type = proto_member_type(req.member_type).ok_or_else(|| {
            Status::invalid_argument(format!("invalid member type: {}", req.member_type))
        })?;
        let role = proto_member_role(req.role).ok_or_else(|| {
            Status::invalid_argument(format!("invalid member role: {}", req.role))
        })?;
        let member = self
            .identity
            .update_member_role(&req.group_name, member_type, &req.member_value, role)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(member_to_proto(&member, &req.group_name)))
    }

    async fn batch_add_members(
        &self,
        request: Request<BatchAddMembersRequest>,
    ) -> Result<Response<BatchAddMembersResponse>, Status> {
        let user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.group_name.is_empty() {
            return Err(Status::invalid_argument("group_name is required"));
        }
        let mut entries = Vec::new();
        for entry in &req.members {
            let mt = proto_member_type(entry.member_type).ok_or_else(|| {
                Status::invalid_argument(format!("invalid member type: {}", entry.member_type))
            })?;
            let role = proto_member_role(entry.role).ok_or_else(|| {
                Status::invalid_argument(format!("invalid member role: {}", entry.role))
            })?;
            entries.push((mt, entry.member_value.clone(), role));
        }
        let members = self
            .identity
            .batch_add_members(&req.group_name, &entries, &user_id)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(BatchAddMembersResponse {
            members: members
                .iter()
                .map(|m| member_to_proto(m, &req.group_name))
                .collect(),
        }))
    }

    async fn resolve_user_groups(
        &self,
        request: Request<ResolveUserGroupsRequest>,
    ) -> Result<Response<ResolveUserGroupsResponse>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.user_id.is_empty() {
            return Err(Status::invalid_argument("user_id is required"));
        }
        let groups = self
            .identity
            .resolve_user_groups(&req.user_id)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(ResolveUserGroupsResponse { groups }))
    }

    async fn is_member(
        &self,
        request: Request<IsMemberRequest>,
    ) -> Result<Response<IsMemberResponse>, Status> {
        let _user_id = permissions::extract_user_id(&request)
            .ok_or_else(|| Status::permission_denied("authentication required"))?;
        let req = request.into_inner();
        if req.user_id.is_empty() || req.group_name.is_empty() {
            return Err(Status::invalid_argument(
                "user_id and group_name are required",
            ));
        }
        let result = self
            .identity
            .is_member(&req.user_id, &req.group_name)
            .await
            .map_err(identity_error_to_status)?;
        Ok(Response::new(IsMemberResponse { is_member: result }))
    }
}
