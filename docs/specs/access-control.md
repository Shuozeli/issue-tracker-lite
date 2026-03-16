<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Access Control -- Detailed Component Spec

## Overview

Access control determines what a user can do with components, issues, hotlists, saved searches, and bookmark groups. It combines component-level ACLs, expanded access rules, and per-issue restriction levels.

---

## Identity Types

```typescript
enum IdentityType {
  USER = 'USER',     // Individual Google Account (identified by email)
  GROUP = 'GROUP',   // Google Group (google.com or googlegroups.com domain)
  PUBLIC = 'PUBLIC', // Special: all users including unauthenticated
}
```

---

## Component Permissions

Permissions are granted per-identity on each component. See [component.md](./component.md) for the full permission table and implication graph.

### Permission Evaluation Order

```
1. Is the user's identity (or a group they belong to) in the component ACL?
   |-- Yes -> return the granted permission set
   |-- No  -> continue

2. Is Expanded Access enabled on the component?
   |-- Yes -> check the user's role on the specific issue:
   |   |-- Assignee/Verifier/Collaborator -> EDIT_ISSUES
   |   |-- CC/Reporter                    -> COMMENT_ON_ISSUES
   |-- No  -> continue

3. Deny access
```

### Critical Rule: CREATE_ISSUES is Isolated

`CREATE_ISSUES` does NOT imply `VIEW_ISSUES` or `COMMENT_ON_ISSUES`. A user with only `CREATE_ISSUES` permission:
- Can create issues in the component
- Cannot view the issues they created (unless Expanded Access is on, which auto-adds Reporter to CC -> grants COMMENT_ON_ISSUES -> implies VIEW_ISSUES)

---

## Per-Issue Access Levels

Each issue has an `access_level` field that restricts access beyond the component ACL:

| Level | Behavior |
|---|---|
| `DEFAULT` | Normal component ACL rules apply. |
| `LIMITED_COMMENTING` | Only identities explicitly listed on the issue (assignee, reporter, verifier, collaborators, CC) AND users with ADMIN_ISSUES can comment. View access remains governed by component ACL. |
| `LIMITED_VISIBILITY` | Only identities explicitly listed on the issue can view. Component ACL is overridden for view. |
| `LIMITED_VISIBILITY_GOOGLE` | Listed identities + full-time Google employees + internal automation can view. |

### Who Can Change Access Level

- Users with `ADMIN_ISSUES` permission on the component.

### Interaction with Expanded Access

When Expanded Access is **disabled**, the issue identity list (assignee, CC, etc.) does NOT grant any permissions -- only the component ACL applies. The `LIMITED_COMMENTING` and `LIMITED_VISIBILITY` levels still work, but they restrict rather than expand.

---

## Restricted Content (Comments & Attachments)

Individual comments and attachments have their own restriction level:

```typescript
enum RestrictionLevel {
  UNRESTRICTED = 'UNRESTRICTED',       // Anyone with VIEW_ISSUES
  RESTRICTED = 'RESTRICTED',           // Requires VIEW_RESTRICTED
  RESTRICTED_PLUS = 'RESTRICTED_PLUS', // Requires VIEW_RESTRICTED_PLUS
}
```

### Who Can Restrict Content

- Set at creation time by the author/uploader.
- Changed after creation by: ADMIN_ISSUES holder, comment author, or attachment uploader.

### Audit

- All access attempts (successful and unsuccessful) to restricted content are logged.

---

## Hotlist Permissions

| Permission | Code | Implies | Notes |
|---|---|---|---|
| Admin | `HOTLIST_ADMIN` | VIEW_APPEND, VIEW | Edit title/description, manage ACL, archive/unarchive |
| View and Append | `HOTLIST_VIEW_APPEND` | VIEW | Add/remove/reorder issues |
| View Only | `HOTLIST_VIEW` | -- | View hotlist and its issue list |

### Key Rules

- Private by default on creation (only creator gets HOTLIST_ADMIN).
- Hotlist visibility does NOT grant issue visibility. If a user can see a hotlist but not an issue in it, that issue is hidden from their view.
- Issue visibility does NOT reveal hotlist membership unless the user has hotlist VIEW permission.

---

## Saved Search Permissions

| Permission | Code | Implies | Notes |
|---|---|---|---|
| Admin | `SEARCH_ADMIN` | VIEW_EXECUTE | Edit/delete criteria, manage ACL |
| View and Execute | `SEARCH_VIEW_EXECUTE` | -- | Run the search, make a copy |

### Key Rules

- On creation, SEARCH_ADMIN is granted to the creator.
- Search results are filtered: only issues the user has VIEW_ISSUES on are returned.

---

## Bookmark Group Permissions

| Permission | Code | Implies | Notes |
|---|---|---|---|
| Admin | `BOOKMARK_ADMIN` | VIEW | Edit, add/remove items, archive, manage ACL |
| View Only | `BOOKMARK_VIEW` | -- | View group and its contents |

### Key Rules

- Private by default on creation.
- At least one BOOKMARK_ADMIN must exist at all times.
- Bookmark group visibility does NOT grant visibility to contained hotlists/saved searches.

---

## Permission Check Summary Table

| Action | Required Permission | Scope |
|---|---|---|
| View issue | VIEW_ISSUES | Component ACL + issue access level |
| Comment on issue | COMMENT_ON_ISSUES | Component ACL + issue access level |
| Edit issue fields | EDIT_ISSUES | Component ACL + issue access level |
| Create issue | CREATE_ISSUES | Component ACL |
| Delete issue | ADMIN_ISSUES | Component ACL |
| Change issue access level | ADMIN_ISSUES | Component ACL |
| View restricted comment | VIEW_RESTRICTED | Component ACL |
| View restricted+ comment | VIEW_RESTRICTED_PLUS | Component ACL |
| Manage component ACL | ADMIN_COMPONENTS | Component ACL |
| Create custom fields | ADMIN_COMPONENTS | Component ACL |
| View hotlist | HOTLIST_VIEW | Hotlist ACL |
| Add issue to hotlist | HOTLIST_VIEW_APPEND | Hotlist ACL |
| Manage hotlist | HOTLIST_ADMIN | Hotlist ACL |
| Execute saved search | SEARCH_VIEW_EXECUTE | Saved Search ACL |
| Manage saved search | SEARCH_ADMIN | Saved Search ACL |
| View bookmark group | BOOKMARK_VIEW | Bookmark Group ACL |
| Manage bookmark group | BOOKMARK_ADMIN | Bookmark Group ACL |

---

## Database Schema Sketch

```sql
-- Component ACL entries
CREATE TABLE component_acl (
  component_id BIGINT NOT NULL REFERENCES components(component_id),
  identity_type TEXT NOT NULL,  -- 'USER', 'GROUP', 'PUBLIC'
  identity_value TEXT NOT NULL, -- email or group name or '*'
  permissions TEXT[] NOT NULL,  -- array of permission codes
  PRIMARY KEY (component_id, identity_type, identity_value)
);

-- Hotlist ACL entries
CREATE TABLE hotlist_acl (
  hotlist_id BIGINT NOT NULL REFERENCES hotlists(hotlist_id),
  identity_type TEXT NOT NULL,
  identity_value TEXT NOT NULL,
  permission TEXT NOT NULL,  -- 'HOTLIST_ADMIN', 'HOTLIST_VIEW_APPEND', 'HOTLIST_VIEW'
  PRIMARY KEY (hotlist_id, identity_type, identity_value)
);

-- Saved Search ACL entries
CREATE TABLE saved_search_acl (
  saved_search_id BIGINT NOT NULL REFERENCES saved_searches(saved_search_id),
  identity_type TEXT NOT NULL,
  identity_value TEXT NOT NULL,
  permission TEXT NOT NULL,
  PRIMARY KEY (saved_search_id, identity_type, identity_value)
);

-- Bookmark Group ACL entries
CREATE TABLE bookmark_group_acl (
  bookmark_group_id BIGINT NOT NULL REFERENCES bookmark_groups(bookmark_group_id),
  identity_type TEXT NOT NULL,
  identity_value TEXT NOT NULL,
  permission TEXT NOT NULL,
  PRIMARY KEY (bookmark_group_id, identity_type, identity_value)
);
```
