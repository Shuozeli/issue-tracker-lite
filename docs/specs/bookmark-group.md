<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Bookmark Group -- Detailed Component Spec

## Overview

A Bookmark Group is an ordered collection of hotlists and saved searches, providing a composite dashboard view. It acts as a personalized navigation container.

## Resource Name

```
bookmarkGroups/{bookmark_group_id}
```

---

## Fields

| Field | Type | Default | Writable | Notes |
|---|---|---|---|---|
| `bookmark_group_id` | `int64` | Auto-generated | No | Globally unique. |
| `name` | `string` | -- | Yes | NOT unique. |
| `description` | `string` | `""` | Yes | |
| `archived` | `bool` | `false` | Yes | Only by BOOKMARK_ADMIN. |
| `create_time` | `timestamp` | Auto | No | |
| `modify_time` | `timestamp` | Auto | No | |

---

## Bookmark Group Items

Ordered entries within a bookmark group.

| Field | Type | Notes |
|---|---|---|
| `bookmark_group_id` | `int64` | |
| `position` | `int32` | Order within the group. |
| `item_type` | `enum` | `HOTLIST` or `SAVED_SEARCH` |
| `item_id` | `int64` | ID of the referenced hotlist or saved search. |

---

## Permissions

| Permission | What It Allows |
|---|---|
| BOOKMARK_ADMIN | Edit name/description, add/remove items, archive/unarchive, manage ACL |
| BOOKMARK_VIEW | View the group and its contents |

---

## Business Rules

1. **Private by default** on creation.
2. **At least one BOOKMARK_ADMIN must exist at all times.** Cannot remove the last admin.
3. **Archiving** hides the group from all subscribers' navigation.
4. **Visibility is independent:** Bookmark group visibility does NOT grant visibility to contained hotlists or saved searches. Users see only the items they independently have access to.
5. **Adding items requires permissions on those items:**
   - Adding a saved search: must have SEARCH_ADMIN on it.
   - Adding a hotlist: must have HOTLIST_VIEW on it.
6. **Dashboard view:** Each contained item renders as a section showing its issues or search results.

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Create | `POST /v1/bookmarkGroups` | Authenticated user |
| Get | `GET /v1/bookmarkGroups/{id}` | BOOKMARK_VIEW |
| List | `GET /v1/bookmarkGroups` | BOOKMARK_VIEW (filtered) |
| Update | `PATCH /v1/bookmarkGroups/{id}` | BOOKMARK_ADMIN |
| Archive | `POST /v1/bookmarkGroups/{id}:archive` | BOOKMARK_ADMIN |
| Unarchive | `POST /v1/bookmarkGroups/{id}:unarchive` | BOOKMARK_ADMIN |
| AddItem | `POST /v1/bookmarkGroups/{id}/items` | BOOKMARK_ADMIN |
| RemoveItem | `DELETE /v1/bookmarkGroups/{id}/items/{item_id}` | BOOKMARK_ADMIN |
| ReorderItems | `POST /v1/bookmarkGroups/{id}/items:reorder` | BOOKMARK_ADMIN |
| ListItems | `GET /v1/bookmarkGroups/{id}/items` | BOOKMARK_VIEW |
| GetACL | `GET /v1/bookmarkGroups/{id}/acl` | BOOKMARK_VIEW |
| UpdateACL | `PATCH /v1/bookmarkGroups/{id}/acl` | BOOKMARK_ADMIN |
