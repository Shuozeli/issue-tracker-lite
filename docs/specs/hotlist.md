<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Hotlist -- Detailed Component Spec

## Overview

A Hotlist is a manually-curated, ordered list of issues that cuts across components. Unlike saved searches, hotlist membership is explicit -- issues are manually added and removed.

## Resource Name

```
hotlists/{hotlist_id}
```

---

## Fields

| Field | Type | Default | Writable | Notes |
|---|---|---|---|---|
| `hotlist_id` | `int64` | Auto-generated | No | Globally unique. Name is NOT unique. |
| `name` | `string` | -- | Yes | Display name. NOT unique across system. |
| `description` | `string` | `""` | Yes | |
| `owner` | `string` (user email) | Creator | No | |
| `archived` | `bool` | `false` | Yes | Only by HOTLIST_ADMIN. |
| `create_time` | `timestamp` | Auto | No | |
| `modify_time` | `timestamp` | Auto | No | |

---

## Hotlist Membership

The join table between hotlists and issues.

| Field | Type | Notes |
|---|---|---|
| `hotlist_id` | `int64` | |
| `issue_id` | `int64` | |
| `position` | `int32` | Ordering within the hotlist. Drag-and-drop reordering when sorted by position. |
| `add_time` | `timestamp` | When the issue was added. |
| `added_by` | `string` (user email) | Who added the issue. |

---

## Permissions

See [access-control.md](./access-control.md) for the full hotlist ACL spec.

| Permission | What It Allows |
|---|---|
| HOTLIST_ADMIN | Edit name/description, manage ACL, archive/unarchive, add/remove issues |
| HOTLIST_VIEW_APPEND | Add/remove/reorder issues in the hotlist |
| HOTLIST_VIEW | View the hotlist and its issue list |

---

## Business Rules

1. **Private by default.** On creation, only the creator gets HOTLIST_ADMIN.
2. **Names are NOT unique.** Hotlists are identified by ID, not name.
3. **Issue membership is independent of issue visibility.** If a user has HOTLIST_VIEW but not VIEW_ISSUES on a component, those issues are hidden from their hotlist view.
4. **Issue visibility does NOT reveal hotlist membership.** Viewing an issue does not tell you which hotlists it belongs to unless you have HOTLIST_VIEW on those hotlists.
5. **Archived hotlists** are hidden from navigation for all subscribers. Only HOTLIST_ADMIN users can see them in the Archived section.
6. **Duplicate cascade:** When an issue is marked as duplicate, its hotlists are auto-added to the canonical issue.
7. **Inline creation:** The hotlist picker allows creating a new hotlist inline if the search query doesn't exactly match an existing one.
8. **Issues can belong to multiple hotlists.**

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Create | `POST /v1/hotlists` | Authenticated user |
| Get | `GET /v1/hotlists/{id}` | HOTLIST_VIEW |
| List | `GET /v1/hotlists` | HOTLIST_VIEW (filtered) |
| Update | `PATCH /v1/hotlists/{id}` | HOTLIST_ADMIN |
| Archive | `POST /v1/hotlists/{id}:archive` | HOTLIST_ADMIN |
| Unarchive | `POST /v1/hotlists/{id}:unarchive` | HOTLIST_ADMIN |
| AddIssue | `POST /v1/hotlists/{id}/issues` | HOTLIST_VIEW_APPEND |
| RemoveIssue | `DELETE /v1/hotlists/{id}/issues/{issue_id}` | HOTLIST_VIEW_APPEND |
| ReorderIssues | `POST /v1/hotlists/{id}/issues:reorder` | HOTLIST_VIEW_APPEND |
| ListIssues | `GET /v1/hotlists/{id}/issues` | HOTLIST_VIEW |
| GetACL | `GET /v1/hotlists/{id}/acl` | HOTLIST_VIEW |
| UpdateACL | `PATCH /v1/hotlists/{id}/acl` | HOTLIST_ADMIN |
