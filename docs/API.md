<!-- agent-updated: 2026-03-19T00:00:00Z -->

# gRPC API Reference

This document is the complete API reference for the MyIssueTracker gRPC services.

---

## Table of Contents

- [Authentication](#authentication)
- [Error Codes](#error-codes)
- [Pagination](#pagination)
- [Services](#services)
  - [HealthService](#healthservice)
  - [ComponentService](#componentservice)
  - [IssueService](#issueservice)
  - [CommentService](#commentservice)
  - [HotlistService](#hotlistservice)
  - [SearchService](#searchservice)
  - [EventLogService](#eventlogservice)
  - [AclService](#aclservice)
  - [GroupService](#groupservice)
- [Enum Reference](#enum-reference)

---

## Authentication

All RPCs require an `x-user-id` metadata header identifying the caller.

- **Header:** `x-user-id`
- **Format:** Alphanumeric string plus `@`, `-`, `_`, `.`, `+` characters. Maximum 256 characters.
- **Missing header:** Returns `PERMISSION_DENIED`.
- **Invalid format:** Returns `PERMISSION_DENIED`.

The `x-user-id` value is used as the actor for event logging and ACL permission checks.

---

## Error Codes

| gRPC Code | When Used |
|---|---|
| `NOT_FOUND` | Resource does not exist (component, issue, comment, hotlist, group). |
| `INVALID_ARGUMENT` | Missing required fields, invalid field values, malformed page tokens. |
| `ALREADY_EXISTS` | Duplicate resource creation (e.g., duplicate group name, duplicate ACL entry). |
| `FAILED_PRECONDITION` | Operation not allowed in current state (e.g., deleting a component with child components). |
| `PERMISSION_DENIED` | Missing or invalid `x-user-id` header, or insufficient ACL permissions. |
| `INTERNAL` | Unexpected server error (database failure, etc.). |

Error messages are redacted to avoid leaking internal identifiers.

---

## Pagination

List RPCs follow a cursor-based pagination pattern:

- **Request fields:**
  - `page_size` (int32): Maximum number of results to return. Capped at 100 server-side. Default varies by service.
  - `page_token` (string): Opaque token from a previous response's `next_page_token`.
- **Response fields:**
  - `next_page_token` (string): Token to pass in the next request. Empty string means no more pages.

Invalid or malformed `page_token` values return `INVALID_ARGUMENT`.

---

## Services

### HealthService

**Package:** `issuetracker.v1`

Simple health check endpoint.

#### Ping

```
rpc Ping(PingRequest) returns (PingResponse)
```

**PingRequest:** (empty)

**PingResponse:**

| Field | Type | Description |
|---|---|---|
| `message` | string | Server health message. |

---

### ComponentService

**Package:** `issuetracker.v1`

Manages issue tracker components (project containers).

#### CreateComponent

```
rpc CreateComponent(CreateComponentRequest) returns (Component)
```

**CreateComponentRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Display name of the component. |
| `description` | string | No | Description text. |
| `parent_id` | int64 | No | Parent component ID for hierarchical nesting. |

**Returns:** The created `Component`.

**Errors:** `INVALID_ARGUMENT` (missing name), `NOT_FOUND` (parent does not exist).

#### GetComponent

```
rpc GetComponent(GetComponentRequest) returns (Component)
```

**GetComponentRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | ID of the component to retrieve. |

**Returns:** The `Component`.

**Errors:** `NOT_FOUND`.

#### ListComponents

```
rpc ListComponents(ListComponentsRequest) returns (ListComponentsResponse)
```

**ListComponentsRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `parent_id` | int64 | No | Filter to children of this parent. Omit for root components. |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListComponentsResponse:**

| Field | Type | Description |
|---|---|---|
| `components` | repeated Component | List of components. |
| `next_page_token` | string | Pagination cursor for next page. |

#### UpdateComponent

```
rpc UpdateComponent(UpdateComponentRequest) returns (Component)
```

**UpdateComponentRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | ID of the component to update. |
| `name` | string | No | New name. |
| `description` | string | No | New description. |
| `parent_id` | int64 | No | New parent component ID. |
| `expanded_access_enabled` | bool | No | Enable expanded access. |
| `editable_comments_enabled` | bool | No | Enable editable comments. |
| `update_mask` | FieldMask | No | Fields to update. |

**Returns:** The updated `Component`.

**Errors:** `NOT_FOUND`, `INVALID_ARGUMENT`.

#### DeleteComponent

```
rpc DeleteComponent(DeleteComponentRequest) returns (DeleteComponentResponse)
```

**DeleteComponentRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | ID of the component to delete. |

**Returns:** Empty `DeleteComponentResponse`.

**Errors:** `NOT_FOUND`, `FAILED_PRECONDITION` (has child components or issues).

---

#### Component Resource

| Field | Type | Description |
|---|---|---|
| `component_id` | int64 | Unique identifier. |
| `name` | string | Display name. |
| `description` | string | Description text. |
| `parent_id` | int64 | Parent component ID (optional). |
| `expanded_access_enabled` | bool | Whether expanded access is enabled. |
| `editable_comments_enabled` | bool | Whether comment editing is enabled. |
| `create_time` | Timestamp | Creation time. |
| `update_time` | Timestamp | Last modification time. |
| `child_count` | int32 | Number of child components. |

---

### IssueService

**Package:** `issuetracker.v1`

Manages issues and issue relationships (parent/child, blocking, duplicate).

#### CreateIssue

```
rpc CreateIssue(CreateIssueRequest) returns (Issue)
```

**CreateIssueRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | Component this issue belongs to. |
| `title` | string | Yes | Issue title. |
| `description` | string | No | Issue description. |
| `priority` | Priority | Yes | Priority level. |
| `type` | IssueType | Yes | Issue type. |
| `severity` | Severity | No | Severity level. |
| `assignee` | string | No | Assignee user ID. |
| `reporter` | string | No | Reporter user ID (defaults to `x-user-id`). |
| `verifier` | string | No | Verifier user ID. |
| `found_in` | string | No | Version where issue was found. |
| `targeted_to` | string | No | Target version for fix. |

**Returns:** The created `Issue`.

**Errors:** `INVALID_ARGUMENT` (missing required fields), `NOT_FOUND` (component does not exist).

#### GetIssue

```
rpc GetIssue(GetIssueRequest) returns (Issue)
```

**GetIssueRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | ID of the issue. |

**Errors:** `NOT_FOUND`.

#### ListIssues

```
rpc ListIssues(ListIssuesRequest) returns (ListIssuesResponse)
```

**ListIssuesRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | Component to list issues from. |
| `status_filter` | string | No | Filter: `"open"`, `"closed"`, or `"all"` (default: `"open"`). |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListIssuesResponse:**

| Field | Type | Description |
|---|---|---|
| `issues` | repeated Issue | List of issues. |
| `next_page_token` | string | Pagination cursor. |

#### UpdateIssue

```
rpc UpdateIssue(UpdateIssueRequest) returns (Issue)
```

**UpdateIssueRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | ID of the issue to update. |
| `title` | string | No | New title. |
| `description` | string | No | New description. |
| `status` | Status | No | New status. |
| `priority` | Priority | No | New priority. |
| `severity` | Severity | No | New severity. |
| `type` | IssueType | No | New type. |
| `component_id` | int64 | No | Move to different component. |
| `assignee` | string | No | New assignee. |
| `reporter` | string | No | New reporter. |
| `verifier` | string | No | New verifier. |
| `found_in` | string | No | New found-in version. |
| `targeted_to` | string | No | New target version. |
| `verified_in` | string | No | New verified-in version. |
| `in_prod` | bool | No | In-production flag. |
| `archived` | bool | No | Archive flag. |
| `update_mask` | FieldMask | No | Fields to update. |

**Returns:** The updated `Issue`.

**Errors:** `NOT_FOUND`, `INVALID_ARGUMENT`.

#### AddParent

```
rpc AddParent(AddParentRequest) returns (RelationshipResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `child_id` | int64 | Yes | The child issue ID. |
| `parent_id` | int64 | Yes | The parent issue ID. |

**Errors:** `NOT_FOUND`, `ALREADY_EXISTS`, `INVALID_ARGUMENT` (self-reference).

#### RemoveParent

```
rpc RemoveParent(RemoveParentRequest) returns (RelationshipResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `child_id` | int64 | Yes | The child issue ID. |
| `parent_id` | int64 | Yes | The parent issue ID. |

**Errors:** `NOT_FOUND`.

#### ListParents

```
rpc ListParents(ListRelatedIssuesRequest) returns (ListRelatedIssuesResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | Issue to list parents for. |

**Returns:** `ListRelatedIssuesResponse` with `repeated Issue issues`.

#### ListChildren

```
rpc ListChildren(ListRelatedIssuesRequest) returns (ListRelatedIssuesResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | Issue to list children for. |

**Returns:** `ListRelatedIssuesResponse` with `repeated Issue issues`.

#### AddBlocking

```
rpc AddBlocking(AddBlockingRequest) returns (RelationshipResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `blocking_id` | int64 | Yes | The blocking issue ID. |
| `blocked_id` | int64 | Yes | The blocked issue ID. |

**Errors:** `NOT_FOUND`, `ALREADY_EXISTS`, `INVALID_ARGUMENT` (self-reference).

#### RemoveBlocking

```
rpc RemoveBlocking(RemoveBlockingRequest) returns (RelationshipResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `blocking_id` | int64 | Yes | The blocking issue ID. |
| `blocked_id` | int64 | Yes | The blocked issue ID. |

**Errors:** `NOT_FOUND`.

#### MarkDuplicate

```
rpc MarkDuplicate(MarkDuplicateRequest) returns (Issue)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | The duplicate issue ID. |
| `canonical_id` | int64 | Yes | The canonical (original) issue ID. |

**Returns:** The updated `Issue` (status set to `DUPLICATE`).

**Errors:** `NOT_FOUND`, `INVALID_ARGUMENT` (self-reference).

#### UnmarkDuplicate

```
rpc UnmarkDuplicate(UnmarkDuplicateRequest) returns (Issue)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | The issue to unmark. |

**Returns:** The updated `Issue` (status reverted from `DUPLICATE`).

**Errors:** `NOT_FOUND`, `FAILED_PRECONDITION` (not currently marked as duplicate).

---

#### Issue Resource

| Field | Type | Description |
|---|---|---|
| `issue_id` | int64 | Unique identifier. |
| `title` | string | Issue title. |
| `description` | string | Issue description. |
| `status` | Status | Current status. |
| `priority` | Priority | Priority level. |
| `severity` | Severity | Severity level. |
| `type` | IssueType | Issue type classification. |
| `component_id` | int64 | Owning component ID. |
| `assignee` | string | Assigned user ID. |
| `reporter` | string | Reporter user ID. |
| `verifier` | string | Verifier user ID. |
| `create_time` | Timestamp | Creation time. |
| `modify_time` | Timestamp | Last modification time. |
| `resolve_time` | Timestamp | When the issue was resolved (optional). |
| `verify_time` | Timestamp | When the issue was verified (optional). |
| `vote_count` | int32 | Number of votes. |
| `duplicate_count` | int32 | Number of duplicates pointing to this issue. |
| `found_in` | string | Version where issue was found. |
| `targeted_to` | string | Target version for fix. |
| `verified_in` | string | Version where fix was verified. |
| `in_prod` | bool | Whether the issue is in production. |
| `archived` | bool | Whether the issue is archived. |
| `access_level` | string | Access level: `"DEFAULT"`, `"LIMITED_COMMENTING"`, `"LIMITED_VISIBILITY"`. |

---

### CommentService

**Package:** `issuetracker.v1`

Manages comments on issues, including edit history and moderation.

#### CreateComment

```
rpc CreateComment(CreateCommentRequest) returns (Comment)
```

**CreateCommentRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | Issue to comment on. |
| `body` | string | Yes | Comment body text. |
| `author` | string | Yes | Author user ID. |

**Errors:** `NOT_FOUND` (issue), `INVALID_ARGUMENT` (empty body).

#### ListComments

```
rpc ListComments(ListCommentsRequest) returns (ListCommentsResponse)
```

**ListCommentsRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `issue_id` | int64 | Yes | Issue to list comments for. |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListCommentsResponse:**

| Field | Type | Description |
|---|---|---|
| `comments` | repeated Comment | List of comments. |
| `next_page_token` | string | Pagination cursor. |

#### UpdateComment

```
rpc UpdateComment(UpdateCommentRequest) returns (Comment)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `comment_id` | int64 | Yes | Comment to update. |
| `body` | string | Yes | New comment body. |

Creates a revision of the previous body. Returns the updated `Comment` with incremented `revision_count`.

**Errors:** `NOT_FOUND`, `INVALID_ARGUMENT` (empty body).

#### HideComment

```
rpc HideComment(HideCommentRequest) returns (Comment)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `comment_id` | int64 | Yes | Comment to hide/unhide. |
| `hidden` | bool | Yes | `true` to hide, `false` to unhide. |

**Errors:** `NOT_FOUND`.

#### ListCommentRevisions

```
rpc ListCommentRevisions(ListCommentRevisionsRequest) returns (ListCommentRevisionsResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `comment_id` | int64 | Yes | Comment to list revisions for. |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListCommentRevisionsResponse:**

| Field | Type | Description |
|---|---|---|
| `revisions` | repeated CommentRevision | Edit history entries. |
| `next_page_token` | string | Pagination cursor. |

---

#### Comment Resource

| Field | Type | Description |
|---|---|---|
| `comment_id` | int64 | Unique identifier. |
| `issue_id` | int64 | Parent issue ID. |
| `author` | string | Author user ID. |
| `body` | string | Comment body text. |
| `is_description` | bool | Whether this is the issue description comment. |
| `create_time` | Timestamp | Creation time. |
| `modify_time` | Timestamp | Last edit time (optional). |
| `hidden` | bool | Whether the comment is hidden. |
| `hidden_by` | string | User who hid the comment. |
| `hidden_time` | Timestamp | When the comment was hidden (optional). |
| `revision_count` | int32 | Number of edits. |

#### CommentRevision Resource

| Field | Type | Description |
|---|---|---|
| `revision_id` | int64 | Unique identifier. |
| `comment_id` | int64 | Parent comment ID. |
| `body` | string | Body text at this revision. |
| `edited_by` | string | User who made the edit. |
| `create_time` | Timestamp | When the revision was created. |

---

### HotlistService

**Package:** `issuetracker.v1`

Manages hotlists (curated collections of issues with ordering).

#### CreateHotlist

```
rpc CreateHotlist(CreateHotlistRequest) returns (Hotlist)
```

**CreateHotlistRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Hotlist name. |
| `description` | string | No | Description text. |
| `owner` | string | Yes | Owner user ID. |

**Errors:** `INVALID_ARGUMENT` (missing name or owner).

#### GetHotlist

```
rpc GetHotlist(GetHotlistRequest) returns (Hotlist)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Hotlist ID. |

**Errors:** `NOT_FOUND`.

#### ListHotlists

```
rpc ListHotlists(ListHotlistsRequest) returns (ListHotlistsResponse)
```

**ListHotlistsRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `filter` | string | No | Filter: `"all"`, `"active"`, `"archived"` (default: `"active"`). |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListHotlistsResponse:**

| Field | Type | Description |
|---|---|---|
| `hotlists` | repeated Hotlist | List of hotlists. |
| `next_page_token` | string | Pagination cursor. |

#### UpdateHotlist

```
rpc UpdateHotlist(UpdateHotlistRequest) returns (Hotlist)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Hotlist to update. |
| `name` | string | No | New name. |
| `description` | string | No | New description. |
| `archived` | bool | No | Archive/unarchive. |

**Errors:** `NOT_FOUND`.

#### AddIssue

```
rpc AddIssue(AddIssueToHotlistRequest) returns (HotlistIssue)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Target hotlist. |
| `issue_id` | int64 | Yes | Issue to add. |
| `added_by` | string | Yes | User performing the action. |

**Errors:** `NOT_FOUND` (hotlist or issue), `ALREADY_EXISTS` (issue already in hotlist).

#### RemoveIssue

```
rpc RemoveIssue(RemoveIssueFromHotlistRequest) returns (RemoveIssueFromHotlistResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Target hotlist. |
| `issue_id` | int64 | Yes | Issue to remove. |

**Errors:** `NOT_FOUND`.

#### ListIssues (Hotlist)

```
rpc ListIssues(ListHotlistIssuesRequest) returns (ListHotlistIssuesResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Hotlist to list issues from. |

**ListHotlistIssuesResponse:**

| Field | Type | Description |
|---|---|---|
| `issues` | repeated HotlistIssue | Ordered list of hotlist issue entries. |

#### ReorderIssues

```
rpc ReorderIssues(ReorderHotlistIssuesRequest) returns (ReorderHotlistIssuesResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Hotlist to reorder. |
| `issue_ids` | repeated int64 | Yes | Ordered list of issue IDs in the new desired order. |

**Errors:** `NOT_FOUND`, `INVALID_ARGUMENT` (issue IDs do not match hotlist contents).

---

#### Hotlist Resource

| Field | Type | Description |
|---|---|---|
| `hotlist_id` | int64 | Unique identifier. |
| `name` | string | Display name. |
| `description` | string | Description text. |
| `owner` | string | Owner user ID. |
| `archived` | bool | Whether the hotlist is archived. |
| `create_time` | Timestamp | Creation time. |
| `modify_time` | Timestamp | Last modification time. |
| `issue_count` | int32 | Number of issues in the hotlist. |

#### HotlistIssue Resource

| Field | Type | Description |
|---|---|---|
| `hotlist_id` | int64 | Parent hotlist ID. |
| `issue_id` | int64 | Issue ID. |
| `position` | int32 | Sort position within the hotlist. |
| `add_time` | Timestamp | When the issue was added. |
| `added_by` | string | User who added the issue. |

---

### SearchService

**Package:** `issuetracker.v1`

Full-text and structured search across issues.

#### SearchIssues

```
rpc SearchIssues(SearchIssuesRequest) returns (SearchIssuesResponse)
```

**SearchIssuesRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `query` | string | Yes | Query string. Supports structured filters (e.g., `"status:open priority:P0 memory leak"`). |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |
| `order_by` | string | No | Sort field: `"created"`, `"modified"`, `"priority"` (default: `"modified"`). |
| `order_direction` | string | No | Sort direction: `"asc"` or `"desc"` (default: `"desc"`). |

**SearchIssuesResponse:**

| Field | Type | Description |
|---|---|---|
| `issues` | repeated Issue | Matching issues. |
| `next_page_token` | string | Pagination cursor. |
| `total_count` | int32 | Total number of matching issues (across all pages). |

LIKE wildcards (`%`, `_`) in keyword segments are escaped server-side.

---

### EventLogService

**Package:** `issuetracker.v1`

Read-only access to the system event log for auditing and debugging.

#### ListEvents

```
rpc ListEvents(ListEventsRequest) returns (ListEventsResponse)
```

**ListEventsRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `entity_type` | string | No | Filter by entity: `"Issue"`, `"Component"`, `"Hotlist"`, `"Comment"`, or `""` for all. |
| `entity_id` | int64 | No | Filter by entity ID (0 = all). |
| `event_type` | string | No | Filter by event type (e.g., `"ISSUE_CREATED"`, `"ISSUE_UPDATED"`). `""` for all. |
| `actor` | string | No | Filter by actor user ID. `""` for all. |
| `since` | Timestamp | No | Start of time range filter. |
| `until` | Timestamp | No | End of time range filter. |
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListEventsResponse:**

| Field | Type | Description |
|---|---|---|
| `events` | repeated Event | List of events. |
| `next_page_token` | string | Pagination cursor. |

#### Event Resource

| Field | Type | Description |
|---|---|---|
| `event_id` | int64 | Unique identifier. |
| `event_time` | Timestamp | When the event occurred. |
| `event_type` | string | Event type (e.g., `"ISSUE_CREATED"`, `"COMPONENT_UPDATED"`). |
| `actor` | string | User who triggered the event. |
| `entity_type` | string | Entity type (e.g., `"Issue"`, `"Component"`). |
| `entity_id` | int64 | Entity ID. |
| `payload` | string | JSON payload with event details. |

---

### AclService

**Package:** `issuetracker.v1`

Access control list management for components and hotlists.

#### SetComponentAcl

```
rpc SetComponentAcl(SetComponentAclRequest) returns (ComponentAclEntry)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | Target component. |
| `identity_type` | IdentityType | Yes | `USER`, `GROUP`, or `PUBLIC`. |
| `identity_value` | string | Yes | User ID, group name, or `""` for PUBLIC. |
| `permissions` | repeated ComponentPermission | Yes | Permissions to grant. |

**Returns:** The created/updated `ComponentAclEntry`.

#### GetComponentAcl

```
rpc GetComponentAcl(GetComponentAclRequest) returns (GetComponentAclResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | Component to query. |

**GetComponentAclResponse:**

| Field | Type | Description |
|---|---|---|
| `entries` | repeated ComponentAclEntry | All ACL entries for the component. |

#### RemoveComponentAcl

```
rpc RemoveComponentAcl(RemoveComponentAclRequest) returns (RemoveComponentAclResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | Target component. |
| `identity_type` | IdentityType | Yes | Identity type to remove. |
| `identity_value` | string | Yes | Identity value to remove. |

**Errors:** `NOT_FOUND`.

#### SetHotlistAcl

```
rpc SetHotlistAcl(SetHotlistAclRequest) returns (HotlistAclEntry)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Target hotlist. |
| `identity_type` | IdentityType | Yes | `USER`, `GROUP`, or `PUBLIC`. |
| `identity_value` | string | Yes | User ID, group name, or `""` for PUBLIC. |
| `permission` | HotlistPermission | Yes | Permission to grant. |

**Returns:** The created/updated `HotlistAclEntry`.

#### GetHotlistAcl

```
rpc GetHotlistAcl(GetHotlistAclRequest) returns (GetHotlistAclResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Hotlist to query. |

**GetHotlistAclResponse:**

| Field | Type | Description |
|---|---|---|
| `entries` | repeated HotlistAclEntry | All ACL entries for the hotlist. |

#### RemoveHotlistAcl

```
rpc RemoveHotlistAcl(RemoveHotlistAclRequest) returns (RemoveHotlistAclResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `hotlist_id` | int64 | Yes | Target hotlist. |
| `identity_type` | IdentityType | Yes | Identity type to remove. |
| `identity_value` | string | Yes | Identity value to remove. |

**Errors:** `NOT_FOUND`.

#### CheckComponentPermission

```
rpc CheckComponentPermission(CheckComponentPermissionRequest) returns (CheckComponentPermissionResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | int64 | Yes | Component to check. |
| `user_id` | string | Yes | User to check permissions for. |
| `issue_id` | int64 | No | Optional issue ID for expanded access evaluation. |

**CheckComponentPermissionResponse:**

| Field | Type | Description |
|---|---|---|
| `permissions` | repeated ComponentPermission | Effective permissions for the user. |
| `grant_source` | string | How permissions were granted: `"ACL"`, `"EXPANDED_ACCESS"`, or `"DENIED"`. |

---

#### ComponentAclEntry Resource

| Field | Type | Description |
|---|---|---|
| `component_id` | int64 | Component ID. |
| `identity_type` | IdentityType | Identity type. |
| `identity_value` | string | Identity value. |
| `permissions` | repeated ComponentPermission | Granted permissions. |
| `create_time` | Timestamp | When the ACL entry was created. |

#### HotlistAclEntry Resource

| Field | Type | Description |
|---|---|---|
| `hotlist_id` | int64 | Hotlist ID. |
| `identity_type` | IdentityType | Identity type. |
| `identity_value` | string | Identity value. |
| `permission` | HotlistPermission | Granted permission. |
| `create_time` | Timestamp | When the ACL entry was created. |

---

### GroupService

**Package:** `identity.v1`

Manages identity groups and group membership. Groups are used as identity principals in ACL entries.

#### CreateGroup

```
rpc CreateGroup(CreateGroupRequest) returns (Group)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Unique group name (e.g., `"eng-team"`). |
| `display_name` | string | No | Human-readable display name. |
| `description` | string | No | Description text. |

**Errors:** `ALREADY_EXISTS` (name taken), `INVALID_ARGUMENT` (missing name).

#### GetGroup

```
rpc GetGroup(GetGroupRequest) returns (Group)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Group name. |

**Errors:** `NOT_FOUND`.

#### ListGroups

```
rpc ListGroups(ListGroupsRequest) returns (ListGroupsResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `page_size` | int32 | No | Max results (capped at 100). |
| `page_token` | string | No | Pagination cursor. |

**ListGroupsResponse:**

| Field | Type | Description |
|---|---|---|
| `groups` | repeated Group | List of groups. |
| `next_page_token` | string | Pagination cursor. |

#### UpdateGroup

```
rpc UpdateGroup(UpdateGroupRequest) returns (Group)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Group name to update. |
| `display_name` | string | No | New display name. |
| `description` | string | No | New description. |

**Errors:** `NOT_FOUND`.

#### DeleteGroup

```
rpc DeleteGroup(DeleteGroupRequest) returns (DeleteGroupResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Group name to delete. |

**Errors:** `NOT_FOUND`.

#### AddMember

```
rpc AddMember(AddMemberRequest) returns (GroupMember)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `group_name` | string | Yes | Target group. |
| `member_type` | MemberType | Yes | `MEMBER_TYPE_USER` or `MEMBER_TYPE_GROUP`. |
| `member_value` | string | Yes | User ID or nested group name. |
| `role` | MemberRole | Yes | `MEMBER`, `MANAGER`, or `OWNER`. |

**Errors:** `NOT_FOUND` (group), `ALREADY_EXISTS` (already a member).

#### RemoveMember

```
rpc RemoveMember(RemoveMemberRequest) returns (RemoveMemberResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `group_name` | string | Yes | Target group. |
| `member_type` | MemberType | Yes | Member type. |
| `member_value` | string | Yes | Member value to remove. |

**Errors:** `NOT_FOUND`.

#### ListMembers

```
rpc ListMembers(ListMembersRequest) returns (ListMembersResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `group_name` | string | Yes | Group to list members of. |

**ListMembersResponse:**

| Field | Type | Description |
|---|---|---|
| `members` | repeated GroupMember | List of group members. |

#### UpdateMemberRole

```
rpc UpdateMemberRole(UpdateMemberRoleRequest) returns (GroupMember)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `group_name` | string | Yes | Target group. |
| `member_type` | MemberType | Yes | Member type. |
| `member_value` | string | Yes | Member value. |
| `role` | MemberRole | Yes | New role. |

**Errors:** `NOT_FOUND`.

#### BatchAddMembers

```
rpc BatchAddMembers(BatchAddMembersRequest) returns (BatchAddMembersResponse)
```

**BatchAddMembersRequest:**

| Field | Type | Required | Description |
|---|---|---|---|
| `group_name` | string | Yes | Target group. |
| `members` | repeated BatchMemberEntry | Yes | Members to add. |

**BatchMemberEntry:**

| Field | Type | Description |
|---|---|---|
| `member_type` | MemberType | `MEMBER_TYPE_USER` or `MEMBER_TYPE_GROUP`. |
| `member_value` | string | User ID or group name. |
| `role` | MemberRole | Role for this member. |

**BatchAddMembersResponse:**

| Field | Type | Description |
|---|---|---|
| `members` | repeated GroupMember | Successfully added members. |

#### ResolveUserGroups

```
rpc ResolveUserGroups(ResolveUserGroupsRequest) returns (ResolveUserGroupsResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `user_id` | string | Yes | User ID to resolve groups for. |

**ResolveUserGroupsResponse:**

| Field | Type | Description |
|---|---|---|
| `groups` | repeated string | Group names the user belongs to (including transitive membership). |

#### IsMember

```
rpc IsMember(IsMemberRequest) returns (IsMemberResponse)
```

| Field | Type | Required | Description |
|---|---|---|---|
| `user_id` | string | Yes | User ID to check. |
| `group_name` | string | Yes | Group to check membership in. |

**IsMemberResponse:**

| Field | Type | Description |
|---|---|---|
| `is_member` | bool | Whether the user is a member (directly or transitively). |

---

#### Group Resource

| Field | Type | Description |
|---|---|---|
| `name` | string | Unique group name. |
| `display_name` | string | Human-readable name. |
| `description` | string | Description text. |
| `creator` | string | User who created the group. |
| `create_time` | Timestamp | Creation time. |
| `update_time` | Timestamp | Last modification time. |

#### GroupMember Resource

| Field | Type | Description |
|---|---|---|
| `group_name` | string | Parent group name. |
| `member_type` | MemberType | `MEMBER_TYPE_USER` or `MEMBER_TYPE_GROUP`. |
| `member_value` | string | User ID or nested group name. |
| `role` | MemberRole | `MEMBER`, `MANAGER`, or `OWNER`. |
| `added_by` | string | User who added this member. |
| `create_time` | Timestamp | When the member was added. |

---

## Enum Reference

### IssueType

| Value | Number | Description |
|---|---|---|
| `ISSUE_TYPE_UNSPECIFIED` | 0 | Default/unspecified. |
| `BUG` | 1 | Software defect. |
| `FEATURE_REQUEST` | 2 | Feature request. |
| `CUSTOMER_ISSUE` | 3 | Customer-reported issue. |
| `INTERNAL_CLEANUP` | 4 | Internal cleanup/tech debt. |
| `PROCESS` | 5 | Process issue. |
| `VULNERABILITY` | 6 | Security vulnerability. |
| `PRIVACY_ISSUE` | 7 | Privacy concern. |
| `PROGRAM` | 8 | Program-level tracking. |
| `PROJECT` | 9 | Project-level tracking. |
| `FEATURE` | 10 | Feature tracking. |
| `MILESTONE` | 11 | Milestone tracking. |
| `EPIC` | 12 | Epic (large work item). |
| `STORY` | 13 | User story. |
| `TASK` | 14 | Task. |

### Priority

| Value | Number | Description |
|---|---|---|
| `PRIORITY_UNSPECIFIED` | 0 | Default/unspecified. |
| `P0` | 1 | Critical -- requires immediate action. |
| `P1` | 2 | High priority. |
| `P2` | 3 | Medium priority. |
| `P3` | 4 | Low priority. |
| `P4` | 5 | Minimal priority. |

### Severity

| Value | Number | Description |
|---|---|---|
| `SEVERITY_UNSPECIFIED` | 0 | Default/unspecified. |
| `S0` | 1 | Critical severity. |
| `S1` | 2 | High severity. |
| `S2` | 3 | Medium severity. |
| `S3` | 4 | Low severity. |
| `S4` | 5 | Minimal severity. |

### Status

| Value | Number | Category | Description |
|---|---|---|---|
| `STATUS_UNSPECIFIED` | 0 | -- | Default/unspecified. |
| `NEW` | 1 | Open | Newly created, not yet triaged. |
| `ASSIGNED` | 2 | Open | Assigned to someone. |
| `IN_PROGRESS` | 3 | Open | Actively being worked on. |
| `INACTIVE` | 4 | Open | Temporarily inactive. |
| `FIXED` | 5 | Closed | Fixed but not yet verified. |
| `FIXED_VERIFIED` | 6 | Closed | Fixed and verified. |
| `WONT_FIX_INFEASIBLE` | 7 | Closed | Will not fix -- infeasible. |
| `WONT_FIX_NOT_REPRODUCIBLE` | 8 | Closed | Will not fix -- not reproducible. |
| `WONT_FIX_OBSOLETE` | 9 | Closed | Will not fix -- obsolete. |
| `WONT_FIX_INTENDED_BEHAVIOR` | 10 | Closed | Will not fix -- intended behavior. |
| `DUPLICATE` | 11 | Closed | Duplicate of another issue. |

### IdentityType (ACL)

| Value | Number | Description |
|---|---|---|
| `IDENTITY_TYPE_UNSPECIFIED` | 0 | Default/unspecified. |
| `USER` | 1 | Individual user. |
| `GROUP` | 2 | Identity group. |
| `PUBLIC` | 3 | Public access. |

### ComponentPermission

| Value | Number | Description |
|---|---|---|
| `COMPONENT_PERMISSION_UNSPECIFIED` | 0 | Default/unspecified. |
| `VIEW_ISSUES` | 1 | View issues in the component. |
| `COMMENT_ON_ISSUES` | 2 | Add comments to issues. |
| `EDIT_ISSUES` | 3 | Edit issue fields. |
| `ADMIN_ISSUES` | 4 | Full admin over issues. |
| `CREATE_ISSUES` | 5 | Create new issues. |
| `VIEW_COMPONENTS` | 6 | View the component itself. |
| `ADMIN_COMPONENTS` | 7 | Full admin over the component. |
| `VIEW_RESTRICTED` | 8 | View restricted issues. |
| `VIEW_RESTRICTED_PLUS` | 9 | View restricted-plus issues. |

### HotlistPermission

| Value | Number | Description |
|---|---|---|
| `HOTLIST_PERMISSION_UNSPECIFIED` | 0 | Default/unspecified. |
| `HOTLIST_VIEW` | 1 | View the hotlist. |
| `HOTLIST_VIEW_APPEND` | 2 | View and add issues. |
| `HOTLIST_ADMIN` | 3 | Full admin over the hotlist. |

### AccessLevel

| Value | Number | Description |
|---|---|---|
| `ACCESS_LEVEL_UNSPECIFIED` | 0 | Default/unspecified. |
| `DEFAULT` | 1 | Standard access. |
| `LIMITED_COMMENTING` | 2 | Commenting restricted. |
| `LIMITED_VISIBILITY` | 3 | Visibility restricted. |

### MemberType (Groups)

| Value | Number | Description |
|---|---|---|
| `MEMBER_TYPE_UNSPECIFIED` | 0 | Default/unspecified. |
| `MEMBER_TYPE_USER` | 1 | Individual user member. |
| `MEMBER_TYPE_GROUP` | 2 | Nested group member. |

### MemberRole (Groups)

| Value | Number | Description |
|---|---|---|
| `MEMBER_ROLE_UNSPECIFIED` | 0 | Default/unspecified. |
| `MEMBER` | 1 | Regular member. |
| `MANAGER` | 2 | Group manager. |
| `OWNER` | 3 | Group owner. |
