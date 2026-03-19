<!-- agent-updated: 2026-03-19T00:00:00Z -->

# Issue Tracker -- High-Level Architecture

This document describes the system architecture for rebuilding Google Issue Tracker. It is derived from the official documentation at `https://developers.google.com/issue-tracker`.

## System Overview

Issue Tracker is an issue/bug tracking system organized around **Components** (hierarchical containers) that hold **Issues** (the core work items). Issues are cross-referenced through **Hotlists**, **Saved Searches**, and **Bookmark Groups**. Access control is component-based with optional per-issue restrictions.

```
+------------------------------------------------------------------+
|                          Issue Tracker                            |
|                                                                   |
|  +------------------+    +-------------------+    +-------------+ |
|  |   Component      |    |   Issue           |    |  Hotlist    | |
|  |   Service        |    |   Service         |    |  Service    | |
|  +--------+---------+    +--------+----------+    +------+------+ |
|           |                       |                      |        |
|  +--------+---------+    +--------+----------+    +------+------+ |
|  |   Access Control  |    |  Search           |    | Bookmark   | |
|  |   Service         |    |  Service          |    | Group Svc  | |
|  +--------+---------+    +--------+----------+    +------+------+ |
|           |                       |                      |        |
|  +--------+---------+    +--------+----------+    +------+------+ |
|  |   Notification    |    |  Saved Search     |    | User       | |
|  |   Service         |    |  Service          |    | Settings   | |
|  +------------------+    +-------------------+    +-------------+ |
|                                                                   |
|  +------------------------------------------------------------+  |
|  |                     Database Layer                          |  |
|  +------------------------------------------------------------+  |
+------------------------------------------------------------------+
```

## Core Components

### 1. Component Service
Manages the hierarchical tree of components. Each component defines ACL rules, custom fields, and templates for its issues. See [component.md](./component.md).

### 2. Issue Service
CRUD and lifecycle management for the core Issue entity. Handles field validation, status transitions, relationships (parent-child, blocking, duplicate), and comment threading. See [issue.md](./issue.md).

### 3. Access Control Service
Evaluates permissions by combining component ACLs, expanded access rules, and per-issue restriction levels. See [access-control.md](./access-control.md).

### 4. Hotlist Service
Manages user-created, manually-curated, ordered lists of issues. See [hotlist.md](./hotlist.md).

### 5. Saved Search Service
Manages stored search queries that produce dynamic result sets. See [saved-search.md](./saved-search.md).

### 6. Bookmark Group Service
Manages composite views containing hotlists and saved searches. See [bookmark-group.md](./bookmark-group.md).

### 7. Search Service
Full-text and structured query engine over all issues. Supports the Issue Tracker Search Query Language. See [search.md](./search.md).

### 8. Notification Service
Processes issue edits, classifies them by significance (closing/major/minor/silent), evaluates per-role notification preferences, and dispatches email notifications. See [notification.md](./notification.md).

### 9. User Settings Service
Per-user preferences: homepage, date/time format, timezone, keyboard shortcuts, notification defaults. See [user-settings.md](./user-settings.md).

## Entity Relationship Overview

```
Component (tree)
  |-- has many --> Issue
  |-- has many --> Template
  |-- has many --> Custom Field Definition
  |-- has --> ACL (per-identity permission set)

Issue
  |-- belongs to --> Component
  |-- has many --> Comment
  |-- has many --> Attachment
  |-- has many <-> many --> Hotlist  (membership)
  |-- has many <-> many --> Issue    (parent-child, N:N)
  |-- has many <-> many --> Issue    (blocking/blocked-by, reciprocal)
  |-- has 0..1 --> Issue             (duplicate-of)
  |-- has --> Assignee, Reporter, Verifier (User)
  |-- has many --> Collaborator, CC (User/Group)

Hotlist
  |-- contains ordered --> Issue (membership)
  |-- has --> ACL (Admin, View+Append, View)

Saved Search
  |-- stores --> Query string
  |-- has --> ACL (Admin, View+Execute)

Bookmark Group
  |-- contains ordered --> Hotlist refs + Saved Search refs
  |-- has --> ACL (Admin, View)

User
  |-- has --> User Settings
  |-- has many --> per-issue notification overrides
  |-- has many --> starred issues
```

## Data Flow: Issue Lifecycle

```
1. User creates issue in Component
     |
     v
2. Component ACL checked (Create Issues permission)
     |
     v
3. Template applied (default field values)
     |
     v
4. Issue created: status=New, reporter=creator
     |
     v
5. Notification dispatched (Major edit: creation)
     |
     v
6. Issue edited (field changes, comments)
     |-- Edit classified as Closing/Major/Minor/Silent
     |-- Notification dispatched per classification
     v
7. Issue resolved (status -> Fixed/Won't Fix/Duplicate)
     |
     v
8. Optional: Verifier marks Fixed (Verified)
```

## Data Flow: Permission Evaluation

```
1. User requests access to Issue I in Component C
     |
     v
2. Check Issue I restriction level
     |-- Default access     -> proceed to step 3
     |-- Limited commenting -> check if user is in issue identity list
     |-- Limited visibility -> check if user is in issue identity list
     v
3. Check Component C ACL for user's identity
     |-- Direct grant?      -> allow
     |-- Group membership?  -> allow
     v
4. Check Expanded Access (if enabled on C)
     |-- Is user Assignee/Verifier/Collaborator? -> Edit access
     |-- Is user CC'd/Reporter?                  -> Comment access
     v
5. Deny
```

## Web UI

The project includes a React SPA frontend in the `ui/` directory. See
[ui/docs/architecture.md](../../ui/docs/architecture.md) for full details.

```
Browser (React 19 + Ant Design 5)
  |  fetch("/api/...")
  v
Express REST Proxy (ui/server/)
  |  @grpc/grpc-js
  v
Rust gRPC Server (tonic, port 50051)
  |
  v
SQLite (Quiver ORM)
```

Key features:
- RTK Query for server state with tag-based cache invalidation
- 8 pages: Dashboard, Issues, IssueDetail, Components, Hotlists, Search, Events, Login
- Built-in demo console panel with automated scenarios and data seeding
- Playwright E2E tests and demo runner
- Browser DevTools console API (`window.api`)

## Tech Stack Decisions (for our rebuild)

| Concern | Choice | Rationale |
|---|---|---|
| Server Language | Rust | Performance, type safety |
| Server Framework | tonic (gRPC) | Native gRPC, async, codegen |
| ORM | Quiver ORM | Schema-driven codegen, connection pooling, typed row deserialization |
| Database | SQLite | Simple deployment, WAL mode |
| Proto Codegen | tonic-build + prost | Stable gRPC codegen with native tonic integration |
| CLI | clap (Rust) | Derive-based, typed arg parsing |
| UI Framework | React 19 + Ant Design 5 | Component library, dark theme |
| UI State | Redux Toolkit (RTK Query) | Server cache, tag invalidation |
| UI Bundler | Vite 6 | Fast HMR, proxy support |
| API Proxy | Express | REST-to-gRPC, @grpc/grpc-js |
| E2E Testing | Playwright | Cross-browser, remote CDP |
| Package Manager | pnpm | Per project rules |
