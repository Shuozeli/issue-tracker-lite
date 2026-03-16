use quiver_driver_core::{Row, Value};

use crate::error::IdentityError;
use crate::types::{Group, GroupMember, MemberRole, MemberType};

fn get_col_index(row: &Row, name: &str) -> Result<usize, IdentityError> {
    row.columns
        .iter()
        .position(|c| c.name == name)
        .ok_or_else(|| IdentityError::Internal(format!("column '{name}' not found in row")))
}

fn get_text(row: &Row, name: &str) -> Result<String, IdentityError> {
    let idx = get_col_index(row, name)?;
    match &row.values[idx] {
        Value::Text(s) => Ok(s.clone()),
        Value::Null => Ok(String::new()),
        other => Err(IdentityError::Internal(format!(
            "expected text for '{name}', got {other:?}"
        ))),
    }
}

fn get_i32(row: &Row, name: &str) -> Result<i32, IdentityError> {
    let idx = get_col_index(row, name)?;
    match &row.values[idx] {
        Value::Int(v) => Ok(*v as i32),
        Value::UInt(v) => Ok(*v as i32),
        Value::Null => Ok(0),
        other => Err(IdentityError::Internal(format!(
            "expected int for '{name}', got {other:?}"
        ))),
    }
}

pub fn group_from_row(row: &Row) -> Result<Group, IdentityError> {
    Ok(Group {
        id: get_i32(row, "id")?,
        name: get_text(row, "name")?,
        display_name: get_text(row, "displayName")?,
        description: get_text(row, "description")?,
        creator: get_text(row, "creator")?,
        created_at: get_text(row, "createdAt")?,
        updated_at: get_text(row, "updatedAt")?,
    })
}

pub fn group_member_from_row(row: &Row) -> Result<GroupMember, IdentityError> {
    let member_type_str = get_text(row, "memberType")?;
    let role_str = get_text(row, "role")?;

    Ok(GroupMember {
        id: get_i32(row, "id")?,
        group_id: get_i32(row, "groupId")?,
        member_type: MemberType::from_str(&member_type_str)
            .map_err(|e| IdentityError::Internal(e))?,
        member_value: get_text(row, "memberValue")?,
        role: MemberRole::from_str(&role_str).map_err(|e| IdentityError::Internal(e))?,
        added_by: get_text(row, "addedBy")?,
        created_at: get_text(row, "createdAt")?,
    })
}
