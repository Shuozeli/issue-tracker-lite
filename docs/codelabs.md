<!-- agent-updated: 2026-03-16T06:00:00Z -->

# Issue Tracker Lite -- Step-by-Step Codelabs

Three hands-on tutorials that walk through realistic usage scenarios.

---

## Codelab 1: Bug Triage Workflow

**Scenario:** Set up a project with a component hierarchy, file bugs, and triage them through the full lifecycle.

### Prerequisites

Ensure the server is running:

```bash
DATABASE_URL="file:./dev.db" cargo run --bin issuetracker-server
```

Set up the CLI environment:

```bash
export IT_SERVER_ADDR=http://localhost:50051
export IT_USER=admin@example.com
```

### Step 1: Create a component hierarchy

Create a top-level application component, then nested sub-components:

```bash
it component create "MyApp" -d "Main application"
```

Note the returned component ID (expected: 1). Use it as the parent for child components:

```bash
it component create "Backend" --parent-id 1 -d "Backend services"
it component create "Auth" --parent-id 2 -d "Authentication and authorization"
```

Verify the hierarchy:

```bash
it component list --parent-id 1
it component list --parent-id 2
```

### Step 2: Set up ACLs for the team

Grant the development team access to the Auth component:

```bash
it acl set-component 3 \
  --identity-type user \
  --identity-value dev@example.com \
  --permissions VIEW_ISSUES,EDIT_ISSUES,CREATE_ISSUES,COMMENT_ON_ISSUES

it acl set-component 3 \
  --identity-type user \
  --identity-value qa@example.com \
  --permissions VIEW_ISSUES,COMMENT_ON_ISSUES
```

Verify:

```bash
it acl get-component 3
```

### Step 3: File 3 bugs with different priorities

```bash
it issue create -c 3 -t "Login fails with expired OAuth token" \
  -p P0 --type BUG -s S0 -d "Users get 500 error when OAuth token expires mid-session"

it issue create -c 3 -t "Password reset email delayed by 10 minutes" \
  -p P1 --type BUG -s S2 -d "Reset emails taking much longer than expected"

it issue create -c 3 -t "Login page missing ARIA labels" \
  -p P3 --type BUG -s S3 -d "Accessibility audit found missing labels on login form"
```

Note the returned issue IDs (expected: 1, 2, 3).

List all open bugs in the Auth component:

```bash
it issue list -c 3
```

### Step 4: Assign the P0 bug to a developer

Assigning an issue automatically transitions its status to ASSIGNED:

```bash
it issue update 1 -a "dev@example.com"
```

Verify the status changed:

```bash
it issue get 1
```

The status field should now show `ASSIGNED`.

### Step 5: Developer starts work

Transition the issue to IN_PROGRESS:

```bash
it issue update 1 -s IN_PROGRESS
```

Add a comment explaining the investigation:

```bash
it comment add 1 -b "Root cause identified: token refresh logic skips expired tokens instead of re-authenticating."
```

### Step 6: Developer fixes the bug

Mark as FIXED (this sets the resolve_time):

```bash
it issue update 1 -s FIXED
```

Add a fix description:

```bash
it comment add 1 -b "Fixed in commit abc123. Added re-auth fallback when token refresh fails."
```

### Step 7: QA verifies the fix

QA verifies the fix and marks it FIXED_VERIFIED (this sets the verify_time):

```bash
it issue update 1 -s FIXED_VERIFIED
```

```bash
it comment add 1 -b "Verified on staging. OAuth expiry now correctly triggers re-authentication."
```

### Step 8: Review the event log

See the full audit trail for issue 1:

```bash
it events --entity-type Issue --entity-id 1
```

This shows every state transition: creation, assignment, status changes, and who performed each action.

List comments to see the discussion thread:

```bash
it comment list 1
```

---

## Codelab 2: Team Access Control

**Scenario:** Set up groups and permissions for a multi-team organization where each team owns different components.

### Prerequisites

Ensure the server is running and the CLI is configured (same as Codelab 1).

### Step 1: Create groups

```bash
it group create backend-team --display-name "Backend Team" \
  --description "Server-side engineers"

it group create frontend-team --display-name "Frontend Team" \
  --description "UI/UX engineers"

it group create qa-team --display-name "QA Team" \
  --description "Quality assurance engineers"
```

### Step 2: Add members to groups

```bash
it group add-member backend-team --member-type user --member-value alice@example.com
it group add-member backend-team --member-type user --member-value bob@example.com
it group add-member backend-team --member-type user --member-value alice@example.com --role manager

it group add-member frontend-team --member-type user --member-value carol@example.com
it group add-member frontend-team --member-type user --member-value dave@example.com

it group add-member qa-team --member-type user --member-value eve@example.com
it group add-member qa-team --member-type user --member-value frank@example.com
```

Verify membership:

```bash
it group list-members backend-team
it group list-members frontend-team
it group list-members qa-team
```

### Step 3: Create the component hierarchy

```bash
it component create "Platform" -d "Top-level platform component"
it component create "API" --parent-id 1 -d "REST and gRPC API layer"
it component create "Web" --parent-id 1 -d "Web frontend application"
```

Expected IDs: Platform=1, API=2, Web=3.

### Step 4: Grant backend-team ADMIN on API component

```bash
it acl set-component 2 \
  --identity-type group \
  --identity-value backend-team \
  --permissions VIEW_ISSUES,EDIT_ISSUES,CREATE_ISSUES,COMMENT_ON_ISSUES,ADMIN_ISSUES
```

### Step 5: Grant frontend-team ADMIN on Web component

```bash
it acl set-component 3 \
  --identity-type group \
  --identity-value frontend-team \
  --permissions VIEW_ISSUES,EDIT_ISSUES,CREATE_ISSUES,COMMENT_ON_ISSUES,ADMIN_ISSUES
```

### Step 6: Grant qa-team VIEW_ISSUES on Platform (inherited by children)

```bash
it acl set-component 1 \
  --identity-type group \
  --identity-value qa-team \
  --permissions VIEW_ISSUES,COMMENT_ON_ISSUES
```

Because permissions on a parent component are inherited, QA can view issues in both API and Web.

### Step 7: Verify -- backend dev can manage API issues

Check Alice's effective permissions on the API component:

```bash
it acl check 2 --user alice@example.com
```

Expected: Alice has VIEW_ISSUES, EDIT_ISSUES, CREATE_ISSUES, COMMENT_ON_ISSUES, ADMIN_ISSUES on the API component (via backend-team group).

Check Alice's permissions on the Web component:

```bash
it acl check 3 --user alice@example.com
```

Expected: Alice should NOT have edit/admin permissions on the Web component (no ACL grant for backend-team on Web).

### Step 8: Verify -- QA can view all issues but not edit

Check Eve's permissions on Platform (and by inheritance, API and Web):

```bash
it acl check 1 --user eve@example.com
it acl check 2 --user eve@example.com
it acl check 3 --user eve@example.com
```

Expected: Eve has VIEW_ISSUES and COMMENT_ON_ISSUES everywhere (via qa-team on Platform), but does NOT have EDIT_ISSUES or ADMIN_ISSUES.

### Step 9: Resolve transitive group membership

Check which groups a user belongs to:

```bash
it group resolve-groups alice@example.com
it group resolve-groups eve@example.com
```

Check direct membership:

```bash
it group is-member alice@example.com backend-team
it group is-member alice@example.com frontend-team
```

---

## Codelab 3: Search and Hotlists

**Scenario:** Create a realistic set of issues, use search to find specific subsets, and organize work with hotlists.

### Prerequisites

Ensure the server is running and the CLI is configured (same as Codelab 1).

### Step 1: Set up components and issues

Create two components:

```bash
it component create "Server" -d "Server-side services"
it component create "Client" -d "Client applications"
```

Expected IDs: Server=1, Client=2.

Create 6 issues across both components with varying priorities and statuses:

```bash
# Server issues
it issue create -c 1 -t "Memory leak in connection pool" \
  -p P0 --type BUG -s S1 -d "Server OOM after 24h uptime"

it issue create -c 1 -t "Add rate limiting to API" \
  -p P1 --type FEATURE_REQUEST -d "Need rate limiting before public launch"

it issue create -c 1 -t "Optimize database query for dashboard" \
  -p P2 --type TASK -d "Dashboard query takes 3s, should be under 500ms"

# Client issues
it issue create -c 2 -t "App crashes on Android 14" \
  -p P0 --type BUG -s S0 -d "Crash on startup for Android 14 devices"

it issue create -c 2 -t "Dark mode support" \
  -p P3 --type FEATURE_REQUEST -d "Users requesting dark mode"

it issue create -c 2 -t "Update localization strings" \
  -p P2 --type TASK -d "New translations for French and German"
```

Expected issue IDs: 1-6.

Assign some issues and update statuses:

```bash
it issue update 1 -a "alice@example.com"
it issue update 2 -a "bob@example.com"
it issue update 4 -a "carol@example.com"
it issue update 4 -s IN_PROGRESS
```

### Step 2: Search for all P0 bugs

```bash
it search "priority:P0 type:BUG"
```

Expected results: Issue 1 (memory leak) and Issue 4 (Android crash).

### Step 3: Search for unassigned open issues

```bash
it search "status:open assignee:none"
```

Expected results: Issue 3 (optimize query), Issue 5 (dark mode), Issue 6 (localization).

### Step 4: Recursive component search

Search for all issues in the Server component and its children (if any):

```bash
it search "componentid:1+"
```

Expected results: Issues 1, 2, 3 (all Server component issues).

### Step 5: Search with keywords

Find issues mentioning specific terms:

```bash
it search "memory leak"
it search "crash -status:closed"
it search "status:open dark mode"
```

### Step 6: Create a hotlist for release blockers

```bash
it hotlist create --name "Release Blockers" -d "Must fix before v2.0 launch"
```

Expected hotlist ID: 1.

### Step 7: Add the P0 issues to the hotlist

```bash
it hotlist add-issue 1 1    # Add issue 1 (memory leak) to hotlist 1
it hotlist add-issue 1 4    # Add issue 4 (Android crash) to hotlist 1
it hotlist add-issue 1 2    # Add issue 2 (rate limiting) to hotlist 1
```

View the hotlist contents:

```bash
it hotlist issues 1
```

### Step 8: Reorder the hotlist by priority

Put the most critical issues first:

```bash
it hotlist reorder 1 --order 4,1,2
```

This sets the order to: Android crash (P0/S0) first, memory leak (P0/S1) second, rate limiting (P1) third.

Verify the new order:

```bash
it hotlist issues 1
```

### Step 9: Search by hotlist

Find all issues in the Release Blockers hotlist:

```bash
it search "hotlistid:1"
```

Expected results: Issues 1, 2, and 4.

### Step 10: Combine search filters

Find open P0 bugs in the hotlist:

```bash
it search "hotlistid:1 priority:P0 type:BUG"
```

Find all open issues NOT in the hotlist:

```bash
it search "status:open -hotlistid:1"
```

Find assigned issues across all components:

```bash
it search "assignee:any status:open"
```

### Step 11: Manage multiple hotlists

Create a second hotlist for a sprint:

```bash
it hotlist create --name "Sprint 42" -d "Current sprint work items"
```

Add the lower-priority items:

```bash
it hotlist add-issue 2 3    # Optimize query
it hotlist add-issue 2 5    # Dark mode
it hotlist add-issue 2 6    # Localization
```

View both hotlists:

```bash
it hotlist list
it hotlist issues 1
it hotlist issues 2
```

Archive a completed hotlist later:

```bash
it hotlist update 2 --archived true
it hotlist list --filter archived
```
