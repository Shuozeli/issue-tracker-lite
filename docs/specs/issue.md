<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Issue -- Detailed Component Spec

## Overview

The Issue is the core entity of the system. It represents a bug, feature request, task, or any trackable work item. Issues live inside Components and carry a rich set of standard fields, optional custom fields, comments, attachments, and relationships to other issues.

## Resource Name

```
components/{component_id}/issues/{issue_id}
```

Short form for cross-component references: `issues/{issue_id}` (issue IDs are globally unique integers).

---

## Fields

### Required Fields (on creation)

| Field | Type | Constraints |
|---|---|---|
| `component` | `string` (component resource name) | Must reference an existing component. User must have `CREATE_ISSUES` permission. |
| `title` | `string` | Non-empty. Displayed in search results and lists. |
| `priority` | `Priority` enum | One of P0..P4. |
| `type` | `IssueType` enum | One of the type values below. |

### Standard Fields

| Field | Type | Default | Writable | Notes |
|---|---|---|---|---|
| `issue_id` | `int64` | Auto-generated | No | Globally unique. |
| `title` | `string` | -- | Yes | |
| `description` | `string` (Markdown) | `""` | Yes | First comment body. Editable if component enables "Editable comments" AND user has Edit Issues or is creator. |
| `status` | `Status` enum | `NEW` | Yes | See status transition rules. |
| `priority` | `Priority` enum | `P2` | Yes | |
| `severity` | `Severity` enum | `S2` | Yes | |
| `type` | `IssueType` enum | `BUG` | Yes | |
| `component` | `string` | -- | Yes | Moving issue = changing component. |
| `assignee` | `string` (user email) | `null` | Yes | |
| `reporter` | `string` (user email) | Creator | Yes (minor edit) | |
| `verifier` | `string` (user email) | `null` | Yes | |
| `collaborators` | `string[]` (user emails) | `[]` | Yes | |
| `cc` | `string[]` (user/group emails) | `[]` | Yes | |
| `found_in` | `string` | `""` | Yes | Version string. |
| `targeted_to` | `string` | `""` | Yes | Version string. |
| `verified_in` | `string` | `""` | Yes | Version string. |
| `in_prod` | `bool` | `false` | Yes | |
| `archived` | `bool` | `false` | Yes | Only by issue editors. |
| `vote_count` | `int32` | `0` | No | Incremented by starring. |
| `duplicate_count` | `int32` | `0` | No | Auto-calculated. |
| `status_update` | `string` (Markdown) | `""` | Yes | Soft limit 4 lines. Shows days-since-last-update badge. |
| `estimated_effort` | `string` | `""` | Yes | Story points, dev days, or t-shirt sizes per component config. |
| `start_date` | `date` | `null` | Yes | |
| `end_date` | `date` | `null` | Yes | |
| `create_time` | `timestamp` | Auto | No | |
| `modify_time` | `timestamp` | Auto | No | |
| `resolve_time` | `timestamp` | Auto | No | Set when status transitions to closed. |
| `verify_time` | `timestamp` | Auto | No | Set when status transitions to FIXED_VERIFIED. |
| `custom_fields` | `map<string, Value>` | `{}` | Yes | Keyed by custom field ID. |

---

## Enums

### IssueType

```typescript
enum IssueType {
  BUG = 'BUG',
  FEATURE_REQUEST = 'FEATURE_REQUEST',
  CUSTOMER_ISSUE = 'CUSTOMER_ISSUE',
  INTERNAL_CLEANUP = 'INTERNAL_CLEANUP',
  PROCESS = 'PROCESS',
  VULNERABILITY = 'VULNERABILITY',
  PRIVACY_ISSUE = 'PRIVACY_ISSUE',
  PROGRAM = 'PROGRAM',
  PROJECT = 'PROJECT',
  FEATURE = 'FEATURE',
  MILESTONE = 'MILESTONE',
  EPIC = 'EPIC',
  STORY = 'STORY',
  TASK = 'TASK',
}
```

### Priority

```typescript
enum Priority {
  P0 = 'P0', // Immediate action required
  P1 = 'P1', // Quick resolution needed
  P2 = 'P2', // Reasonable timescale (default)
  P3 = 'P3', // When able
  P4 = 'P4', // Eventually
}
```

### Severity

```typescript
enum Severity {
  S0 = 'S0', // Critical
  S1 = 'S1', // Major
  S2 = 'S2', // Moderate (default)
  S3 = 'S3', // Minor
  S4 = 'S4', // Cosmetic
}
```

### Status

```typescript
enum Status {
  // Open statuses
  NEW = 'NEW',
  ASSIGNED = 'ASSIGNED',
  IN_PROGRESS = 'IN_PROGRESS',
  INACTIVE = 'INACTIVE',          // Added June 2025; auto-reopens on major edits

  // Closed statuses
  FIXED = 'FIXED',
  FIXED_VERIFIED = 'FIXED_VERIFIED',
  WONT_FIX_INFEASIBLE = 'WONT_FIX_INFEASIBLE',
  WONT_FIX_NOT_REPRODUCIBLE = 'WONT_FIX_NOT_REPRODUCIBLE',
  WONT_FIX_OBSOLETE = 'WONT_FIX_OBSOLETE',
  WONT_FIX_INTENDED_BEHAVIOR = 'WONT_FIX_INTENDED_BEHAVIOR',
  DUPLICATE = 'DUPLICATE',
}
```

### AccessLevel (per-issue restriction)

```typescript
enum AccessLevel {
  DEFAULT = 'DEFAULT',                    // Normal component ACL rules
  LIMITED_COMMENTING = 'LIMITED_COMMENTING', // Only listed identities + Admin Issues can comment
  LIMITED_VISIBILITY = 'LIMITED_VISIBILITY', // Only listed identities can view
  LIMITED_VISIBILITY_GOOGLE = 'LIMITED_VISIBILITY_GOOGLE', // Listed identities + FTEs
}
```

---

## Status Transition Rules

### Valid Transitions

```
NEW -> ASSIGNED (when assignee is set)
NEW -> IN_PROGRESS
NEW -> any closed status

ASSIGNED -> IN_PROGRESS
ASSIGNED -> NEW (when assignee is cleared)
ASSIGNED -> any closed status

IN_PROGRESS -> any closed status
IN_PROGRESS -> ASSIGNED (pause work)

FIXED -> FIXED_VERIFIED (by verifier only)
any closed -> NEW (reopen, clears assignee)
any closed -> ASSIGNED (reopen, retains assignee)

INACTIVE -> any open status (auto-triggered by major edit)
```

### Automatic Transitions

- Setting assignee on NEW issue -> status becomes ASSIGNED
- Clearing assignee on ASSIGNED issue -> status becomes NEW
- Marking as duplicate -> status becomes DUPLICATE, sets `duplicate_of`
- Unmarking duplicate -> status becomes NEW (no assignee) or ASSIGNED (has assignee)
- Major edit on INACTIVE issue -> status reopens

---

## Relationships

### Parent-Child

- **Cardinality:** N:N (an issue can have multiple parents AND multiple children)
- **Ordering:** Children within a parent have a defined order (sortable)
- **Constraints:**
  - No cycles allowed (system must detect and reject)
  - Max 500 direct children per parent
  - Max 1000 ancestors (transitive parents)
- **Semantics:** Breakdown of work (not dependency)

### Blocking / Blocked By

- **Cardinality:** N:N, reciprocal
- **Semantics:** Advisory timing/sequence dependency. System does NOT enforce (blocked issues can still be closed).
- **Permission:** User must have Edit Issues on BOTH components
- **Display:** `open_count / total_count` format

### Duplicate

- **Cardinality:** N:1 (many issues can be duplicates of one canonical issue)
- **Side effects on mark-duplicate:**
  - Status -> DUPLICATE (closed)
  - Reporter, Assignee, Verifier, Collaborators, CC auto-added to canonical issue's CC
  - Hotlists auto-added to canonical issue
  - `duplicate_count` on canonical issue incremented
- **Side effects on unmark-duplicate:**
  - Status -> NEW (no assignee) or ASSIGNED (has assignee)
  - People previously auto-added to canonical CC remain unless explicitly removed

---

## Comments

| Field | Type | Notes |
|---|---|---|
| `comment_id` | `int64` | Auto-generated, unique within issue. |
| `author` | `string` (user email) | Immutable after creation. |
| `body` | `string` (Markdown) | |
| `create_time` | `timestamp` | |
| `modify_time` | `timestamp` | Null if never edited. |
| `restriction_level` | `RestrictionLevel` enum | UNRESTRICTED, RESTRICTED, RESTRICTED_PLUS |
| `is_description` | `bool` | True for the first comment (the issue description). |

### Comment Rules

- Editable only if component enables "Editable comments" setting AND user has Edit Issues or is the comment author.
- Editing a comment is a **silent edit** (no notification).
- "+1" comments are classified as **minor edits**.
- Comments support `@email` mentions (triggers notification).
- Comment reactions: thumbs up.
- Comments can be copied to another issue (manual action, not auto-submitted).

### Restriction Levels (for comments and attachments)

```typescript
enum RestrictionLevel {
  UNRESTRICTED = 'UNRESTRICTED',   // Anyone with View Issues
  RESTRICTED = 'RESTRICTED',       // Requires View Restricted permission
  RESTRICTED_PLUS = 'RESTRICTED_PLUS', // Requires View Restricted+ permission
}
```

---

## Attachments

| Field | Type | Notes |
|---|---|---|
| `attachment_id` | `int64` | Auto-generated. |
| `filename` | `string` | |
| `content_type` | `string` | MIME type. |
| `size_bytes` | `int64` | |
| `uploader` | `string` (user email) | |
| `create_time` | `timestamp` | |
| `restriction_level` | `RestrictionLevel` enum | |

---

## Edit Classification

Every mutation to an issue is classified for notification purposes:

### Closing Edits
- Status transitions from open to closed

### Major Edits
- Issue creation
- Comment added (except "+1")
- Component change (move)
- Priority, severity, assignee change
- Status close/verify/reopen
- Custom fields marked as "Major"

### Minor Edits
- Title change
- Hotlist membership change
- Attachment added
- Relationship changes (blocking, blocked by, duplicate, parent, child)
- Reporter, type, verifier, found_in, targeted_to, verified_in, in_prod changes
- Non-major status changes
- Custom fields marked as "Minor"

### Silent Edits
- CC or collaborator add/remove (except the affected user)
- Comment edit
- Custom fields marked as "Silent"

---

## API Methods

Per Google AIP:

| Method | Endpoint | Permission Required |
|---|---|---|
| Create | `POST /v1/components/{id}/issues` | CREATE_ISSUES on component |
| Get | `GET /v1/issues/{id}` | VIEW_ISSUES on component (+ restriction check) |
| List | `GET /v1/components/{id}/issues` | VIEW_ISSUES on component |
| Update | `PATCH /v1/issues/{id}` | EDIT_ISSUES on component (+ restriction check) |
| Search | `GET /v1/issues:search` | VIEW_ISSUES (filtered per component) |
| AddComment | `POST /v1/issues/{id}/comments` | COMMENT_ON_ISSUES on component |
| ListComments | `GET /v1/issues/{id}/comments` | VIEW_ISSUES on component |
| UpdateComment | `PATCH /v1/issues/{id}/comments/{id}` | EDIT_ISSUES or comment author |
| AddAttachment | `POST /v1/issues/{id}/attachments` | COMMENT_ON_ISSUES on component |
| BulkUpdate | `POST /v1/issues:bulkUpdate` | EDIT_ISSUES on all affected components |
