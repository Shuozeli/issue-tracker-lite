<!-- agent-updated: 2026-03-08T12:00:00Z -->

# Tracker -- Detailed Component Spec

## Overview

A Tracker is a branded, scoped view of the Issue Tracker. It provides a custom URL, logo, color theme, and scoped navigation that only shows issues and components belonging to the tracker. Trackers are used to give external partners a focused experience.

## Resource Name

```
trackers/{tracker_id}
```

---

## Fields

| Field | Type | Default | Writable | Notes |
|---|---|---|---|---|
| `tracker_id` | `int64` | Auto-generated | No | Globally unique. |
| `name` | `string` | -- | Yes | Display name. |
| `domain_url` | `string` | -- | Yes | Custom domain URL for the tracker. |
| `logo_url` | `string` | `""` | Yes | Custom logo. |
| `color_theme` | `string` | `""` | Yes | Custom color theme. |
| `description` | `string` | `""` | Yes | |
| `trusted_groups` | `string[]` | `[]` | Yes | Trusted collaborator groups configured by tracker admins. |
| `create_time` | `timestamp` | Auto | No | |
| `modify_time` | `timestamp` | Auto | No | |

---

## Tracker-Component Association

Components are associated with trackers to define the scope.

| Field | Type |
|---|---|
| `tracker_id` | `int64` |
| `component_id` | `int64` |

---

## Business Rules

1. **Scoped autocomplete:** Within a tracker, issue and component autocomplete only shows tracker-scoped items.
2. **Cross-tracker references:** Dependencies on issues outside the tracker can be added by explicit issue ID.
3. **Redirect:** Viewing a non-tracker issue from within a tracker context redirects the user outside the tracker.
4. **Tracker chips:** Issues belonging to a tracker display a tracker chip/badge when viewed outside the tracker context.
5. **Searchable:** `trackerid:ID` in the search query language.
6. **Access propagation:** Tracker access changes can take up to 12 hours to propagate. Component/issue access changes take up to 1 hour.

---

## API Methods

| Method | Endpoint | Permission Required |
|---|---|---|
| Get | `GET /v1/trackers/{id}` | Tracker member |
| List | `GET /v1/trackers` | Filtered by membership |
| Update | `PATCH /v1/trackers/{id}` | Tracker admin |
| ListComponents | `GET /v1/trackers/{id}/components` | Tracker member |
