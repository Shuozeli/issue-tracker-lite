<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Notification -- Detailed Component Spec

## Overview

The Notification Service processes issue mutations, classifies them by significance, evaluates per-user notification preferences, and dispatches email notifications. It also handles per-issue subscription overrides and the read/unread system.

---

## Edit Classification

Every issue mutation is classified into one of four levels. This classification drives both notification dispatch and read/unread status.

### Closing Edits
- Status transitions from any open status to any closed status.

### Major Edits
- Issue creation
- Comment added (except "+1" which is minor)
- Component change (moving an issue)
- Priority change
- Severity change
- Assignee change
- Status: close, verify, reopen
- Custom field changes marked as "Major"

### Minor Edits
- Title change
- Hotlist membership change
- Attachment added
- Relationship changes: blocking, blocked by, duplicate, parent, child
- Reporter, type, verifier, found_in, targeted_to, verified_in, in_prod changes
- Non-major status changes
- Custom field changes marked as "Minor"

### Silent Edits
- CC or collaborator add/remove (but the affected user IS notified of their own addition/removal)
- Comment editing
- Custom field changes marked as "Silent"

---

## Notification Roles

A user's relationship to an issue determines their notification role:

| Role | Condition |
|---|---|
| Assignee | User is in the `assignee` field |
| Reporter | User is in the `reporter` field |
| Verifier | User is in the `verifier` field |
| Collaborator | User is in the `collaborators` list |
| CC | User is in the `cc` list |
| Starred | User has starred the issue |

---

## Notification Levels

Each role has a configurable notification level (set in user settings):

| Level | Receives Notifications For |
|---|---|
| ALL_UPDATES | Closing + major + minor edits |
| MAJOR_UPDATES | Closing + major edits |
| CLOSURE_ONLY | Closing edits only |
| MINIMAL | Only if the issue is marked fixed and the user is the verifier |

### Resolution Rules

1. **Multiple roles:** If a user has multiple roles on an issue (e.g., assignee AND CC), the **highest** notification level wins.
2. **Own edits:** By default, a user's own edits do NOT generate notifications for themselves. This is configurable via the "Exclude edits made by you" setting.
3. **Per-issue override:** On each issue, a user can override their role-based notification level. This includes a `NONE` option to completely mute notifications for that issue.
4. **Always notified:** Users are always notified when they are added to or removed from an issue, regardless of notification level.

---

## Group Notifications

When a Google Group is in the CC field:
- Individual members receive notifications if:
  1. Their CC notification settings allow it, AND
  2. The group's email subscription is set to "Each email"

---

## Read/Unread Status

| Event | Effect |
|---|---|
| User opens issue detail page (finishes loading) | Marked as **read** |
| Another user makes a major edit | Marked as **unread** |
| Minor edits by another user | No change to read status |
| User's own edits | No change to read status |

### Display
- Unread issues appear in **bold** in lists.
- Closed issues have darker background and lighter font.

### Bulk Operations
- Mark multiple issues as read or unread from list view.
- Keyboard shortcuts: `Shift+i` (read), `Shift+u` (unread).

---

## Per-Issue Subscription Mechanisms

Users can subscribe to an issue through multiple channels:

| Mechanism | Visibility | How |
|---|---|---|
| CC field | Visible to all | Added by anyone with EDIT_ISSUES |
| CC Me (on comment) | Visible (adds to CC) | Opt-in checkbox when posting a comment |
| Star | Private | Toggle; also counts as upvote for `vote_count` |
| Notification override | Private | Per-issue dropdown overriding role-based level; includes NONE to mute |

---

## Custom Email Alerts

Users can configure email alerts based on search queries:
- When an issue matching the query is created or updated, an alert email is sent.
- Limited to queries returning at most 1000 results.
- Added February 2025.

---

## Database Schema Sketch

```sql
-- Per-user notification preferences (role-based defaults)
CREATE TABLE user_notification_settings (
  user_email TEXT NOT NULL,
  role TEXT NOT NULL,  -- 'ASSIGNEE', 'REPORTER', 'VERIFIER', 'COLLABORATOR', 'CC', 'STARRED'
  level TEXT NOT NULL, -- 'ALL_UPDATES', 'MAJOR_UPDATES', 'CLOSURE_ONLY', 'MINIMAL'
  PRIMARY KEY (user_email, role)
);

-- Per-issue notification overrides
CREATE TABLE issue_notification_overrides (
  user_email TEXT NOT NULL,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id),
  level TEXT NOT NULL, -- 'ALL_UPDATES', 'MAJOR_UPDATES', 'CLOSURE_ONLY', 'MINIMAL', 'NONE'
  PRIMARY KEY (user_email, issue_id)
);

-- Read/unread tracking
CREATE TABLE issue_read_status (
  user_email TEXT NOT NULL,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id),
  is_read BOOLEAN NOT NULL DEFAULT FALSE,
  last_read_time TIMESTAMP,
  PRIMARY KEY (user_email, issue_id)
);

-- Stars (private subscriptions + upvotes)
CREATE TABLE issue_stars (
  user_email TEXT NOT NULL,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id),
  create_time TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (user_email, issue_id)
);
```
