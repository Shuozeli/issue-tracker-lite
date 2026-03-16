use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub creator: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    pub id: i32,
    pub group_id: i32,
    pub member_type: MemberType,
    pub member_value: String,
    pub role: MemberRole,
    pub added_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberType {
    User,
    Group,
}

impl MemberType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberType::User => "USER",
            MemberType::Group => "GROUP",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "USER" => Ok(MemberType::User),
            "GROUP" => Ok(MemberType::Group),
            other => Err(format!("unknown member type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    Member,
    Manager,
    Owner,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Member => "MEMBER",
            MemberRole::Manager => "MANAGER",
            MemberRole::Owner => "OWNER",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "MEMBER" => Ok(MemberRole::Member),
            "MANAGER" => Ok(MemberRole::Manager),
            "OWNER" => Ok(MemberRole::Owner),
            other => Err(format!("unknown member role: {other}")),
        }
    }

    /// Returns true if this role can add/remove members of the given role.
    pub fn can_manage(&self, target_role: MemberRole) -> bool {
        match self {
            MemberRole::Owner => true,
            MemberRole::Manager => matches!(target_role, MemberRole::Member),
            MemberRole::Member => false,
        }
    }

    /// Returns true if this role can promote a member to the given role.
    pub fn can_promote_to(&self, target_role: MemberRole) -> bool {
        match self {
            MemberRole::Owner => matches!(target_role, MemberRole::Manager | MemberRole::Owner),
            MemberRole::Manager => matches!(target_role, MemberRole::Member),
            MemberRole::Member => false,
        }
    }
}
