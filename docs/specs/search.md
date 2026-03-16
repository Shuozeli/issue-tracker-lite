<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Search -- Detailed Component Spec

## Overview

The Search Service provides full-text keyword search and structured field-based querying across all issues the user has access to. It implements the Issue Tracker Search Query Language.

---

## Query Syntax

### Operators

| Operator | Syntax | Precedence | Description |
|---|---|---|---|
| AND | whitespace (implicit) | Lowest | Both criteria must match |
| OR | `\|` | Middle | Either criterion matches |
| NOT | `-` prefix | Highest | Criterion must not match |
| Exact phrase | `"..."` | -- | Match exact phrase |
| Grouping | `(...)` | -- | Override precedence |

### Query Forms

1. **Keyword search:** `memory leak crash` -- searches across title, comments, attachment names, user fields, ID fields, version fields, custom fields. Case-insensitive, full-word matching with basic synonym support.

2. **Field:value pairs:** `priority:P0 status:open assignee:me`

3. **Mixed:** `memory leak priority:P0 -status:closed`

---

## Searchable Fields

### ID Fields

| Field | Aliases | Type | Notes |
|---|---|---|---|
| `id` | -- | int | Issue ID |
| `blockingid` | -- | int | Issues this one blocks |
| `blockedbyid` | -- | int | Issues blocking this one |
| `parentid` | -- | int | Parent issue. `parentid:123+` includes transitive children. |
| `canonicalid` | -- | int | Canonical issue (for duplicates) |
| `hotlistid` | `h` | int | Hotlist membership |
| `componentid` | `c` | int | Component. `componentid:123+` includes child components. |
| `trackerid` | -- | int | Tracker membership |

### User Fields

| Field | Aliases | Type | Notes |
|---|---|---|---|
| `reporter` | `r` | user | |
| `assignee` | `a` | user | |
| `collaborator` | -- | user | |
| `cc` | -- | user | |
| `verifier` | `v` | user | |
| `mention` | -- | user | Mentioned in comments |
| `modifier` | -- | user | Any user who edited the issue |
| `lastmodifier` | -- | user | Most recent editor |
| `commenter` | -- | user | Any user who commented |
| `lastcommenter` | -- | user | Most recent commenter |

**Special value:** `me` resolves to the current authenticated user.

### Enum Fields

| Field | Aliases | Values |
|---|---|---|
| `priority` | `p` | P0, P1, P2, P3, P4 |
| `severity` | `s` | S0, S1, S2, S3, S4 |
| `type` | `t` | Bug, Feature_Request, Customer_Issue, etc. |
| `status` | `is` | New, Assigned, In_Progress, Fixed, etc. Special: `open`, `closed` |
| `accesslevel` | -- | Default, Limited_Commenting, Limited_Visibility, etc. |

### Text Fields (Tokenized -- keyword search)

| Field | Notes |
|---|---|
| `title` | Issue title |
| `comment` | All comments |
| `attachment` | Attachment filenames |

### Text Fields (Exact Match)

| Field | Notes |
|---|---|
| `foundin` | Found In version |
| `targetedto` | Targeted To version |
| `verifiedin` | Verified In version |
| `effortlabel` | Estimated effort label |

### Time Fields

| Field | Notes |
|---|---|
| `created` | Issue creation time |
| `modified` | Last modification time |
| `resolved` | Resolution time |
| `verified` | Verification time |

**Time syntax:**
- Absolute: `yyyy-MM-ddTHH:mm:ss` (UTC, any prefix accepted, e.g. `2024-01-15`)
- Range: `2024-01-01..2024-01-31`
- Relative: `7d` (last 7 days)
- Today: `today`, `today+3`, `today-7`
- Relational: `created>2024-01-01`, `modified>=7d`

### Count Fields

| Field | Notes |
|---|---|
| `duplicatecount` | Number of duplicates |
| `votecount` | Number of stars |
| `commentcount` | Number of comments |
| `collaboratorcount` | Number of collaborators |
| `cccount` | Number of CC entries |
| `descendantcount` | Total descendants (children, grandchildren, etc.) |
| `opendescendantcount` | Open descendants |
| `attachmentcount` | Number of attachments |
| `onedayviewcount` | Views in last 1 day |
| `sevendayviewcount` | Views in last 7 days |
| `thirtydayviewcount` | Views in last 30 days |

**Relational operators:** `<`, `<=`, `>`, `>=`, `:` (equals)

### Boolean Fields

| Field | Notes |
|---|---|
| `inprod` | In production |
| `star` | Starred by current user |
| `archived` | Archived. `archived:all` shows both. |
| `mute` | Muted by current user |
| `deleted` / `isdeleted` | Soft-deleted |
| `vote` | Voted by current user |

### Special Fields

| Field | Notes |
|---|---|
| `customfield<id>` | Custom field by numeric ID, e.g. `customfield12345:value` |
| `savedsearchid` | Issues matching a saved search |

### Special Values

| Value | Meaning |
|---|---|
| `me` | Current authenticated user (for user fields) |
| `none` | Field is null/empty |
| `any` | Field is non-null |
| `open` | Any open status (for status field) |
| `closed` | Any closed status (for status field) |
| `all` | Both true and false (for archived field) |

---

## Search Results

### Default Columns

Issues are returned with configurable column sets. Default columns typically include: ID, Priority, Type, Status, Title, Assignee, Modified.

### Sorting

- Syntax: comma-separated fields with optional `desc` suffix.
- Example: `created desc, priority`
- Default: relevance-based for keyword queries, `modified desc` for structured queries.

### Grouping

- Results can be grouped by a field (e.g., group by Status, Priority, Component).
- Each group shows as a collapsible section.

### Pagination

- Cursor-based pagination per AIP-158.
- `page_size` (default configurable by user) and `page_token`.

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Search | `GET /v1/issues:search?query=...` | VIEW_ISSUES (results filtered per component) |

### Request Parameters

| Parameter | Type | Notes |
|---|---|---|
| `query` | `string` | Search query in ITSQL |
| `page_size` | `int32` | Max results per page |
| `page_token` | `string` | Pagination cursor |
| `order_by` | `string` | Sort specification |
| `group_by` | `string` | Grouping field |
| `columns` | `string[]` | Fields to include in response |

### Response

```typescript
interface SearchResponse {
  issues: Issue[];
  next_page_token: string;
  total_size: number;       // Approximate total matches
  group_counts?: Record<string, number>; // When group_by is specified
}
```
