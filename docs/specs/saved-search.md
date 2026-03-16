<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Saved Search -- Detailed Component Spec

## Overview

A Saved Search stores a search query that produces a dynamic result set. Unlike hotlists, results change automatically as issues are created/modified. Saved searches are the primary way to create reusable views of issues.

## Resource Name

```
savedSearches/{saved_search_id}
```

---

## Fields

| Field | Type | Default | Writable | Notes |
|---|---|---|---|---|
| `saved_search_id` | `int64` | Auto-generated | No | Globally unique. Name is NOT unique. |
| `name` | `string` | -- | Yes | Display name. NOT unique across system. |
| `description` | `string` | `""` | Yes | |
| `query` | `string` | -- | Yes | Search query in Issue Tracker Search Query Language. |
| `owner` | `string` (user email) | Creator | No | |
| `create_time` | `timestamp` | Auto | No | |
| `modify_time` | `timestamp` | Auto | No | |

---

## Permissions

| Permission | What It Allows |
|---|---|
| SEARCH_ADMIN | Edit name/description/query, delete, manage ACL |
| SEARCH_VIEW_EXECUTE | Run the search, make a copy |

---

## Business Rules

1. **On creation:** SEARCH_ADMIN is granted to the creator.
2. **Dynamic results:** Executing a saved search runs the stored query against current data. Results change as issues change.
3. **Results are filtered by access:** Only issues the executing user has VIEW_ISSUES on are returned.
4. **Modify and re-run:** Users with SEARCH_ADMIN can modify the query and save. Users with SEARCH_VIEW_EXECUTE can modify and run temporarily but must Save As (creates a copy) or Discard.
5. **Names are NOT unique.** Identified by ID only.

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Create | `POST /v1/savedSearches` | Authenticated user |
| Get | `GET /v1/savedSearches/{id}` | SEARCH_VIEW_EXECUTE |
| List | `GET /v1/savedSearches` | SEARCH_VIEW_EXECUTE (filtered) |
| Update | `PATCH /v1/savedSearches/{id}` | SEARCH_ADMIN |
| Delete | `DELETE /v1/savedSearches/{id}` | SEARCH_ADMIN |
| Execute | `GET /v1/savedSearches/{id}:execute` | SEARCH_VIEW_EXECUTE |
| GetACL | `GET /v1/savedSearches/{id}/acl` | SEARCH_VIEW_EXECUTE |
| UpdateACL | `PATCH /v1/savedSearches/{id}/acl` | SEARCH_ADMIN |
