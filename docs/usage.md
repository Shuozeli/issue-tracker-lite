<!-- agent-updated: 2026-03-16T06:00:00Z -->

# Issue Tracker Lite -- Usage Guide

A practical guide for using the Issue Tracker Lite system, covering server setup, CLI configuration, and command reference.

## Prerequisites

- **Rust 1.85+** (for building server and CLI)
- **Node.js + pnpm** (for Web UI only)

## Starting the Server

```bash
DATABASE_URL="file:./dev.db" cargo run --bin issuetracker-server
```

### Environment Variables

| Variable       | Required | Description                            |
|----------------|----------|----------------------------------------|
| `DATABASE_URL` | Yes      | SQLite database path (e.g., `file:./dev.db`) |
| `LISTEN_ADDR`  | No       | gRPC listen address (default: `0.0.0.0:50051`) |

If `DATABASE_URL` is not set, the server will fail to start.

## CLI Setup

Build the CLI binary:

```bash
cargo build -p issuetracker-cli
# Binary is at target/debug/it
```

Configure your environment:

```bash
export IT_SERVER_ADDR=http://localhost:50051
export IT_USER=admin@example.com
```

Alternatively, pass these as flags:

```bash
it --server http://localhost:50051 --user admin@example.com ping
```

## CLI Commands Reference

### ping

Check server health.

```bash
it ping
```

Expected output: `pong` (or similar health check message).

---

### component

Manage the component hierarchy (projects/modules).

**Create a root component:**

```bash
it component create "Backend" -d "Backend services"
```

**Create a child component:**

```bash
it component create "API" --parent-id 1
```

**Get a component by ID:**

```bash
it component get 1
```

**List components (optionally filtered by parent):**

```bash
it component list
it component list --parent-id 1
```

**Update a component:**

```bash
it component update 1 --name "Backend Core" --description "Updated description"
```

**Delete a component:**

```bash
it component delete 1
```

---

### issue

Manage issues within components.

**Create an issue:**

```bash
it issue create -c 1 -t "Login broken" -p P1 --type BUG
it issue create -c 1 -t "Add dark mode" -p P2 --type FEATURE_REQUEST -d "Users want dark mode"
```

Priority values: `P0`, `P1`, `P2`, `P3`, `P4`

Type values: `BUG`, `FEATURE_REQUEST`, `CUSTOMER_ISSUE`, `INTERNAL_CLEANUP`, `PROCESS`, `VULNERABILITY`, `PRIVACY_ISSUE`, `PROGRAM`, `PROJECT`, `FEATURE`, `MILESTONE`, `EPIC`, `STORY`, `TASK`

Severity values: `S0`, `S1`, `S2`, `S3`, `S4`

**Get an issue:**

```bash
it issue get 1
```

**List issues in a component:**

```bash
it issue list -c 1
it issue list -c 1 --status all
it issue list -c 1 --status closed
```

**Update an issue:**

```bash
it issue update 1 -s ASSIGNED -a "dev@example.com"
it issue update 1 -s IN_PROGRESS
it issue update 1 -s FIXED
it issue update 1 -s FIXED_VERIFIED
```

Status values: `NEW`, `ASSIGNED`, `IN_PROGRESS`, `INACTIVE`, `FIXED`, `FIXED_VERIFIED`, `WONT_FIX_INFEASIBLE`, `WONT_FIX_NOT_REPRODUCIBLE`, `WONT_FIX_OBSOLETE`, `WONT_FIX_INTENDED_BEHAVIOR`, `DUPLICATE`

**Issue relationships -- blocking:**

```bash
# Issue 2 blocks issue 3
it issue block 2 3

# Remove the blocking relationship
it issue unblock 2 3
```

**Issue relationships -- parent/child:**

```bash
it issue add-parent 2 1     # Issue 2's parent is issue 1
it issue remove-parent 2 1
it issue parents 2           # List parents of issue 2
it issue children 1          # List children of issue 1
```

**Issue relationships -- duplicate:**

```bash
it issue duplicate 4 --of 1  # Mark issue 4 as duplicate of issue 1
it issue unduplicate 4        # Unmark as duplicate
```

---

### comment

Add and manage comments on issues.

**Add a comment:**

```bash
it comment add 1 -b "Reproduced on staging"
```

**List comments on an issue:**

```bash
it comment list 1
```

**Edit a comment:**

```bash
it comment edit 5 -b "Updated: reproduced on staging and production"
```

---

### hotlist

Organize issues into named lists.

**Create a hotlist:**

```bash
it hotlist create --name "Sprint 42"
it hotlist create --name "Release Blockers" -d "Must fix before v2.0"
```

**Get a hotlist:**

```bash
it hotlist get 1
```

**List hotlists:**

```bash
it hotlist list
it hotlist list --filter archived
it hotlist list --filter all
```

**Add an issue to a hotlist:**

```bash
it hotlist add-issue 1 3    # Add issue 3 to hotlist 1
```

**Remove an issue from a hotlist:**

```bash
it hotlist remove-issue 1 3
```

**List issues in a hotlist:**

```bash
it hotlist issues 1
```

**Reorder issues in a hotlist:**

```bash
it hotlist reorder 1 --order 3,1,5,2    # Comma-separated issue IDs in desired order
```

**Update a hotlist:**

```bash
it hotlist update 1 --name "Sprint 43"
it hotlist update 1 --archived true
```

---

### search

Query issues using a structured search language.

**Basic search:**

```bash
it search "status:open priority:P0"
it search "memory leak -status:closed"
it search "componentid:1+ assignee:any"
```

**With ordering:**

```bash
it search "priority:P0" --order-by priority --order-dir asc
it search "status:open" --order-by created --order-dir desc
```

**With pagination:**

```bash
it search "type:BUG" --page-size 10
it search "type:BUG" --page-size 10 --page-token "TOKEN_FROM_PREVIOUS_RESPONSE"
```

---

### events

Query the audit event log for debugging and tracking changes.

```bash
it events --entity-type Issue --entity-id 1
it events --entity-type Component --entity-id 1
it events --event-type ISSUE_CREATED
it events --actor admin@example.com
it events --entity-type Issue --entity-id 1 --page-size 10
```

---

### acl

Manage access control lists for components and hotlists.

**Set component permissions for a user:**

```bash
it acl set-component 1 \
  --identity-type user \
  --identity-value dev@example.com \
  --permissions VIEW_ISSUES,COMMENT_ON_ISSUES
```

**Set component permissions for a group:**

```bash
it acl set-component 1 \
  --identity-type group \
  --identity-value backend-team \
  --permissions VIEW_ISSUES,EDIT_ISSUES,CREATE_ISSUES,ADMIN_ISSUES
```

Component permission values: `VIEW_ISSUES`, `COMMENT_ON_ISSUES`, `EDIT_ISSUES`, `ADMIN_ISSUES`, `CREATE_ISSUES`, `VIEW_COMPONENTS`, `ADMIN_COMPONENTS`, `VIEW_RESTRICTED`, `VIEW_RESTRICTED_PLUS`

**Check effective permissions for a user:**

```bash
it acl check 1 --user dev@example.com
it acl check 1 --user dev@example.com --issue-id 5
```

**Get all ACL entries for a component:**

```bash
it acl get-component 1
```

**Remove a component ACL entry:**

```bash
it acl remove-component 1 --identity-type user --identity-value dev@example.com
```

**Hotlist ACL management:**

```bash
it acl set-hotlist 1 \
  --identity-type user \
  --identity-value dev@example.com \
  --permission HOTLIST_VIEW_APPEND

it acl get-hotlist 1

it acl remove-hotlist 1 --identity-type user --identity-value dev@example.com
```

Hotlist permission values: `HOTLIST_VIEW`, `HOTLIST_VIEW_APPEND`, `HOTLIST_ADMIN`

---

### group

Manage identity groups for team-based access control.

**Create a group:**

```bash
it group create backend-team --display-name "Backend Team"
it group create backend-team --display-name "Backend Team" --description "Server-side engineers"
```

**Get a group:**

```bash
it group get backend-team
```

**List all groups:**

```bash
it group list
```

**Update a group:**

```bash
it group update backend-team --display-name "Backend Engineering"
```

**Delete a group:**

```bash
it group delete backend-team
```

**Add a member to a group:**

```bash
it group add-member backend-team --member-type user --member-value dev@example.com
it group add-member backend-team --member-type user --member-value lead@example.com --role manager
it group add-member all-engineers --member-type group --member-value backend-team
```

Member roles: `member`, `manager`, `owner`

**Remove a member:**

```bash
it group remove-member backend-team --member-type user --member-value dev@example.com
```

**List members of a group:**

```bash
it group list-members backend-team
```

**Update a member's role:**

```bash
it group update-member-role backend-team \
  --member-type user \
  --member-value dev@example.com \
  --role owner
```

**Resolve all groups a user belongs to (transitive):**

```bash
it group resolve-groups dev@example.com
```

**Check if a user is a member of a group:**

```bash
it group is-member dev@example.com backend-team
```

---

## Search Query Language

The search command accepts a query string composed of field filters, keywords, and operators.

### Field Filters

| Filter          | Description                          | Example                    |
|-----------------|--------------------------------------|----------------------------|
| `status:`       | Issue status                         | `status:open`, `status:FIXED` |
| `priority:`     | Issue priority                       | `priority:P0`              |
| `severity:`     | Issue severity                       | `severity:S0`              |
| `type:`         | Issue type                           | `type:BUG`                 |
| `assignee:`     | Assigned user                        | `assignee:dev@example.com` |
| `reporter:`     | Reporting user                       | `reporter:qa@example.com`  |
| `componentid:`  | Component ID                         | `componentid:1`            |
| `componentid:N+`| Component ID (recursive, includes children) | `componentid:1+`   |
| `hotlistid:`    | Hotlist ID                           | `hotlistid:1`              |

### Special Values

| Value    | Meaning                                  |
|----------|------------------------------------------|
| `open`   | All non-closed statuses (for `status:`)  |
| `closed` | All closed statuses (for `status:`)      |
| `none`   | Field is empty/unset                     |
| `any`    | Field has any value                      |

### Operators

- **AND**: Whitespace between terms acts as AND. `status:open priority:P0` means both conditions must match.
- **NOT**: Prefix a term with `-` to negate it. `-status:closed` excludes closed issues.
- **Exact phrase**: Wrap keywords in double quotes. `"memory leak"` matches the exact phrase.

### Keyword Search

Bare words (without a field prefix) search across issue title and description.

```bash
it search "memory leak"                     # Title/description contains "memory leak"
it search "status:open crash"               # Open issues mentioning "crash"
it search "priority:P0 type:BUG -assignee:none"  # P0 bugs that are assigned
```

---

## Web UI

The project includes a web UI built with Node.js.

### Starting the Web UI

```bash
cd ui
pnpm install
pnpm dev
```

### Architecture

The Web UI follows a 3-tier architecture:

1. **Browser** -- Frontend application
2. **Express proxy** -- Translates HTTP requests to gRPC calls
3. **gRPC server** -- The Rust backend

The Express proxy connects to the gRPC server at the address configured via environment variables and exposes a REST-like HTTP API for the browser frontend.
