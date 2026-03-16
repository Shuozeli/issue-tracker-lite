use async_trait::async_trait;

use crate::error::IdentityError;
use crate::types::{Group, GroupMember, MemberRole, MemberType};

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    /// Resolve all groups a user belongs to (including nested groups up to max depth).
    async fn resolve_user_groups(&self, user_id: &str) -> Result<Vec<String>, IdentityError>;

    /// Check if a user is a member of a specific group (including nested membership).
    async fn is_member(&self, user_id: &str, group_name: &str) -> Result<bool, IdentityError>;

    /// Create a new group.
    async fn create_group(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        creator: &str,
    ) -> Result<Group, IdentityError>;

    /// Get a group by name.
    async fn get_group(&self, name: &str) -> Result<Group, IdentityError>;

    /// List groups with pagination.
    async fn list_groups(
        &self,
        page_size: i32,
        page_token: &str,
    ) -> Result<(Vec<Group>, String), IdentityError>;

    /// Update a group's display name and/or description.
    async fn update_group(
        &self,
        name: &str,
        display_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Group, IdentityError>;

    /// Delete a group. Fails if the group is referenced elsewhere.
    async fn delete_group(&self, name: &str) -> Result<(), IdentityError>;

    /// Add a member to a group.
    async fn add_member(
        &self,
        group_name: &str,
        member_type: MemberType,
        member_value: &str,
        role: MemberRole,
        added_by: &str,
    ) -> Result<GroupMember, IdentityError>;

    /// Remove a member from a group.
    async fn remove_member(
        &self,
        group_name: &str,
        member_type: MemberType,
        member_value: &str,
    ) -> Result<(), IdentityError>;

    /// List all members of a group.
    async fn list_members(&self, group_name: &str) -> Result<Vec<GroupMember>, IdentityError>;

    /// Update a member's role within a group.
    async fn update_member_role(
        &self,
        group_name: &str,
        member_type: MemberType,
        member_value: &str,
        new_role: MemberRole,
    ) -> Result<GroupMember, IdentityError>;

    /// Add multiple members to a group in a single transaction.
    async fn batch_add_members(
        &self,
        group_name: &str,
        members: &[(MemberType, String, MemberRole)],
        added_by: &str,
    ) -> Result<Vec<GroupMember>, IdentityError>;
}
