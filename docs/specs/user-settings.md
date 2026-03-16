<!-- agent-updated: 2026-03-08T12:00:00Z -->

# User Settings -- Detailed Component Spec

## Overview

Per-user preferences controlling display, notifications, and navigation defaults. This is a singleton resource per user.

## Resource Name

```
users/{user_email}/settings
```

Per AIP-156 (Singleton Resources): only Get and Update, no Create or Delete.

---

## Fields

| Field | Type | Default | Options |
|---|---|---|---|
| `homepage` | `string` | `"assigned_to_me"` | `"assigned_to_me"`, or resource name of any hotlist, saved search, or bookmark group |
| `date_format` | `DateFormat` enum | `MMM_DD_YYYY` | See enum below |
| `time_format` | `TimeFormat` enum | `HH_MM_AMPM` | See enum below |
| `timezone` | `string` | `"LOCAL"` | `"LOCAL"`, `"UTC"`, or IANA timezone (e.g., `"America/Los_Angeles"`) |
| `keyboard_shortcuts_enabled` | `bool` | `true` | |
| `force_plain_text_comments` | `bool` | `false` | Render all comments as plain text |
| `force_code_font_comments` | `bool` | `false` | Render all comments in monospace font |
| `exclude_own_edits` | `bool` | `true` | Do not send notifications for own edits |

---

## Enums

### DateFormat

```typescript
enum DateFormat {
  MMM_DD_YYYY = 'MMM_DD_YYYY',         // Dec 31, 2015
  DDD_MMM_DD_YYYY = 'DDD_MMM_DD_YYYY', // Thu Dec 31, 2015
  YYYY_MM_DD = 'YYYY_MM_DD',           // 2015-12-31
  MM_DD_YYYY = 'MM_DD_YYYY',           // 12/31/2015
  DD_MM_YYYY_DASH = 'DD_MM_YYYY_DASH', // 31-12-2015
  DD_MM_YYYY_DOT = 'DD_MM_YYYY_DOT',   // 31.12.2015
}
```

### TimeFormat

```typescript
enum TimeFormat {
  HH_MM_AMPM = 'HH_MM_AMPM',             // 01:00PM
  H_MM_AMPM = 'H_MM_AMPM',               // 1:00PM
  HH24_MM = 'HH24_MM',                    // 13:00
  HH24_MM_SS = 'HH24_MM_SS',             // 13:00:01
}
```

---

## Notification Defaults

The notification settings are per-role defaults stored separately (see [notification.md](./notification.md)). They are logically part of user settings but stored in the `user_notification_settings` table.

### Default Notification Levels Per Role

| Role | Default Level |
|---|---|
| Assignee | ALL_UPDATES |
| Reporter | MAJOR_UPDATES |
| Verifier | CLOSURE_ONLY |
| Collaborator | ALL_UPDATES |
| CC | MAJOR_UPDATES |
| Starred | MAJOR_UPDATES |

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Get | `GET /v1/users/{email}/settings` | Self only |
| Update | `PATCH /v1/users/{email}/settings` | Self only |
