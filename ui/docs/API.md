<!-- agent-updated: 2026-03-19T00:00:00Z -->

# UI API Reference

The Express proxy (`server/index.ts`) exposes REST endpoints that translate to gRPC
calls on the Rust server.

## Base URL

Development: `http://localhost:3001/api` (proxied via Vite at `http://localhost:5173/api`)

## Headers

| Header | Required | Description |
|---|---|---|
| `x-user-id` | Yes | User identity for permission checks. Required by the gRPC server (returns PERMISSION_DENIED if missing). |
| `Content-Type` | Yes (POST/PATCH) | `application/json` |

## Endpoints

### Components

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| GET | `/api/components` | `ListComponents` | List all components (page_size=100) |
| POST | `/api/components` | `CreateComponent` | Create component |
| GET | `/api/components/:id` | `GetComponent` | Get component by ID |
| PATCH | `/api/components/:id` | `UpdateComponent` | Update component name/description |
| DELETE | `/api/components/:id` | `DeleteComponent` | Delete component (fails if has children/issues) |

### Issues

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| GET | `/api/issues?componentId=N` | `ListIssues` | List issues by component |
| POST | `/api/issues` | `CreateIssue` | Create issue |
| GET | `/api/issues/:id` | `GetIssue` | Get issue by ID |
| PATCH | `/api/issues/:id` | `UpdateIssue` | Update issue fields |

### Issue Relationships

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| POST | `/api/issues/:id/parent` | `AddParent` | Add parent relationship |
| GET | `/api/issues/:id/parents` | `ListParents` | List parent issues |
| GET | `/api/issues/:id/children` | `ListChildren` | List child issues |
| POST | `/api/issues/:id/blocking` | `AddBlocking` | Add blocking relationship |
| POST | `/api/issues/:id/duplicate` | `MarkDuplicate` | Mark as duplicate |

### Comments

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| GET | `/api/issues/:issueId/comments` | `ListComments` | List comments for issue |
| POST | `/api/issues/:issueId/comments` | `CreateComment` | Add comment |
| PATCH | `/api/comments/:commentId` | `UpdateComment` | Edit comment body |
| POST | `/api/comments/:commentId/hide` | `HideComment` | Hide/unhide comment |
| GET | `/api/comments/:commentId/revisions` | `ListCommentRevisions` | List edit history |

### Hotlists

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| GET | `/api/hotlists` | `ListHotlists` | List all hotlists |
| POST | `/api/hotlists` | `CreateHotlist` | Create hotlist |
| GET | `/api/hotlists/:id` | `GetHotlist` | Get hotlist by ID |
| GET | `/api/hotlists/:id/issues` | `ListIssues` | List issues in hotlist |
| POST | `/api/hotlists/:id/issues` | `AddIssue` | Add issue to hotlist |

### Search

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| GET | `/api/search?q=QUERY` | `SearchIssues` | Search issues by query string |

Query params: `q` (query), `orderBy`, `orderDir`, `pageSize`

### Events

| Method | Path | gRPC Method | Description |
|---|---|---|---|
| GET | `/api/events` | `ListEvents` | List event log entries |

Query params: `entityType`, `entityId`, `pageSize`

## Error Response Format

```json
{
  "error": {
    "code": 5,
    "message": "Component not found"
  }
}
```

gRPC to HTTP status mapping:

| gRPC Code | HTTP Status | Meaning |
|---|---|---|
| 0 (OK) | 200 | Success |
| 3 (INVALID_ARGUMENT) | 400 | Bad request |
| 5 (NOT_FOUND) | 404 | Resource not found |
| 6 (ALREADY_EXISTS) | 409 | Conflict |
| 9 (FAILED_PRECONDITION) | 400 | Precondition failed |
| 13 (INTERNAL) | 500 | Internal error |
| 16 (UNAUTHENTICATED) | 401 | Not authenticated |
