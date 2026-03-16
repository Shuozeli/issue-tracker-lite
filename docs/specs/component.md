<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Component -- Detailed Component Spec

## Overview

Components are the hierarchical organizational containers for issues. They define access control, custom fields, and templates. Components form a tree from most general to most specific.

## Resource Name

```
components/{component_id}
```

---

## Fields

| Field | Type | Default | Writable | Notes |
|---|---|---|---|---|
| `component_id` | `int64` | Auto-generated | No | Globally unique. |
| `name` | `string` | -- | Yes | NOT unique across the system. |
| `description` | `string` | `""` | Yes | |
| `parent_component` | `string` (component resource name) | `null` | Yes | Null for root components. |
| `create_time` | `timestamp` | Auto | No | |
| `modify_time` | `timestamp` | Auto | No | |
| `expanded_access_enabled` | `bool` | `true` | Yes | Controls role-based permission expansion. |
| `editable_comments_enabled` | `bool` | `false` | Yes | Allows comment editing. |

---

## Hierarchy

- Components form a **tree** (each component has at most one parent).
- Depth is unbounded in the docs, but practically limited.
- Searching with `componentid:ID+` includes all descendant components.
- Best practice: create issues in the **most specific** matching component.

### Hierarchy Operations

| Operation | Notes |
|---|---|
| Create child component | Requires ADMIN_COMPONENTS on parent. |
| Move component (reparent) | Requires ADMIN_COMPONENTS on both old and new parent. |
| Delete component | Only if no issues exist in it or its descendants. |

---

## Component ACL

Each component maintains a permission set per identity (user, group, or Public).

### Permissions (ordered from most to least powerful)

| Permission | Code | Implies |
|---|---|---|
| Admin Components | `ADMIN_COMPONENTS` | CREATE_ISSUES |
| Admin Issues | `ADMIN_ISSUES` | EDIT_ISSUES, COMMENT_ON_ISSUES, VIEW_ISSUES |
| Edit Issues | `EDIT_ISSUES` | COMMENT_ON_ISSUES, VIEW_ISSUES |
| Comment on Issues | `COMMENT_ON_ISSUES` | VIEW_ISSUES |
| View Issues | `VIEW_ISSUES` | -- |
| Create Issues | `CREATE_ISSUES` | -- (does NOT imply Comment or Edit) |
| View Components | `VIEW_COMPONENTS` | -- (auto-populated) |
| View Restricted | `VIEW_RESTRICTED` | -- |
| View Restricted+ | `VIEW_RESTRICTED_PLUS` | VIEW_RESTRICTED |

### Permission Implication Graph

```
ADMIN_COMPONENTS --> CREATE_ISSUES

ADMIN_ISSUES --> EDIT_ISSUES --> COMMENT_ON_ISSUES --> VIEW_ISSUES

VIEW_RESTRICTED_PLUS --> VIEW_RESTRICTED
```

**Key rule:** CREATE_ISSUES does NOT imply COMMENT_ON_ISSUES or VIEW_ISSUES. A user can create an issue and then not be able to view it (unless Expanded Access adds them via Reporter->CC->Comment).

### ACL Inheritance

- ACLs are NOT inherited from parent components by default.
- Each component has its own independent ACL.
- Evaluating access: check the issue's component ACL only, not ancestors.

---

## Expanded Access

A per-component toggle (`expanded_access_enabled`) that grants permissions based on issue role:

| Issue Role | Granted Permission |
|---|---|
| Assignee | EDIT_ISSUES |
| Verifier | EDIT_ISSUES |
| Collaborator | EDIT_ISSUES |
| CC | COMMENT_ON_ISSUES |
| Reporter | Auto-added to CC -> COMMENT_ON_ISSUES |

**When disabled:**
- None of the above role-based expansions apply.
- The system shows a warning when assigning a user who lacks sufficient permissions.

---

## Custom Field Definitions

Custom fields are defined per component.

| Field | Type | Notes |
|---|---|---|
| `custom_field_id` | `int64` | Auto-generated. |
| `name` | `string` | Display name. |
| `field_type` | `CustomFieldType` enum | TEXT, NUMBER, ENUM, DATE, USER, etc. |
| `enum_values` | `string[]` | Only for ENUM type. |
| `required` | `bool` | If true, must be set on issue creation. |
| `hidden` | `bool` | If true, collapsed in edit view, hidden in create view. |
| `edit_significance` | `EditSignificance` enum | MAJOR, MINOR, SILENT. Determines notification behavior when this field changes. |

### Searchability

Custom fields are searchable via `customfield<id>:value` in the search query language. They can be added as columns in search result views (requires specifying the component first).

---

## Templates

Templates provide default field values for new issues within a component.

| Field | Type | Notes |
|---|---|---|
| `template_id` | `int64` | Auto-generated. |
| `name` | `string` | Display name. |
| `is_default` | `bool` | Each component has exactly one default template. |
| `is_inherited` | `bool` | If true, available to all child components. |
| `field_defaults` | `map<string, Value>` | Default values for issue fields. |

### Template Rules

- Each component has exactly **one** default template.
- Additional non-default templates can be created.
- Inherited templates from parent components appear in child components.
- Users select a template when creating an issue; defaults are pre-filled but editable.

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Create | `POST /v1/components` (root) or `POST /v1/components/{parent_id}/children` | ADMIN_COMPONENTS on parent |
| Get | `GET /v1/components/{id}` | VIEW_COMPONENTS |
| List | `GET /v1/components` or `GET /v1/components/{parent_id}/children` | VIEW_COMPONENTS |
| Update | `PATCH /v1/components/{id}` | ADMIN_COMPONENTS |
| Delete | `DELETE /v1/components/{id}` | ADMIN_COMPONENTS |
| GetACL | `GET /v1/components/{id}/acl` | VIEW_COMPONENTS |
| UpdateACL | `PATCH /v1/components/{id}/acl` | ADMIN_COMPONENTS |
| ListCustomFields | `GET /v1/components/{id}/customFields` | VIEW_COMPONENTS |
| CreateCustomField | `POST /v1/components/{id}/customFields` | ADMIN_COMPONENTS |
| ListTemplates | `GET /v1/components/{id}/templates` | VIEW_COMPONENTS |
| CreateTemplate | `POST /v1/components/{id}/templates` | ADMIN_COMPONENTS |
