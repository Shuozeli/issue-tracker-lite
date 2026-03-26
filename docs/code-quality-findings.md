<!-- agent-updated: 2026-03-26T00:00:00Z -->

# Code Quality Findings

## 1. Duplication (High Impact)

### 1.1 `parse_timestamp` duplicated across 6 files -- FIXED
- **Resolution:** Extracted to `server/src/domain/timestamp.rs`. All 7 service files now import `crate::domain::timestamp::parse_timestamp` instead of defining their own copy.

### 1.2 `log_event` helper duplicated across 3 files with slight variations -- FIXED
- **Resolution:** Extracted to `server/src/domain/event_log.rs` with a unified signature that takes `actor` and `entity_type` as parameters. All three services (issue, comment, hotlist) now use the shared function.

### 1.3 `user_groups` resolution boilerplate repeated in every service method
- **Status:** Open
- **Location:** Every RPC handler across all service files (at least 25 occurrences total)
- **Problem:** The 7-line `match user_id.as_deref() { Some(uid) => self.identity.resolve_user_groups(uid).await.unwrap_or_default(), None => vec![] }` block is copy-pasted into every single RPC handler.
- **Fix:** Extract into a helper method on a shared trait or a free function: `async fn resolve_groups(identity: &dyn IdentityProvider, user_id: &Option<String>) -> Vec<String>`. Each handler calls this one-liner.

### 1.4 Transaction acquisition boilerplate repeated in every handler
- **Status:** Open
- **Location:** Every RPC handler across all service files (30+ occurrences)
- **Problem:** The `let mut conn = self.db.acquire().await.map_err(...)?; let tx = conn.begin().await.map_err(...)?;` pattern (6 lines) is repeated identically in every single handler.
- **Fix:** Create a helper function `async fn begin_tx(db: &DbConn) -> Result<impl Transaction, DomainError>` that encapsulates the acquire-and-begin pattern.

### 1.5 Status list duplication between `status_machine.rs` and `issue_service.rs`
- **Status:** Open
- **Location:** `server/src/domain/status_machine.rs:3-4`, `server/src/service/issue_service.rs`, `server/src/domain/query_parser.rs:197-214`
- **Problem:** The open/closed status lists are defined in 3 separate places. Adding a new status requires updating all 3 locations.
- **Fix:** Make `status_machine::OPEN_STATUSES` and `status_machine::CLOSED_STATUSES` the single source of truth.

### 1.6 Row mapping duplication between `identity` crate and `server` crate
- **Status:** Open
- **Location:** `identity/src/row_mapping.rs` and `server/src/db/row_mapping.rs` (Group and GroupMember mappings)
- **Problem:** Both crates define mapping for Group and GroupMember rows using different APIs. These must be kept in sync manually.
- **Fix:** The identity crate should use the same quiver `Row` trait methods as the server, or re-export its types so the server does not need its own mapping.

## 2. Silent Failures (High Impact)

### 2.1 JSON permission deserialization silently defaults to empty -- FIXED
- **Resolution:** Changed `unwrap_or_default()` to `map_err(|e| DomainError::Internal(...))` with descriptive error messages in `permissions.rs` (2 sites), `acl_service.rs` (`component_acl_to_proto`, `hotlist_acl_to_proto`, and `check_access`). Corrupt JSON now surfaces as an Internal error.

### 2.2 `identity_type_to_proto` returns 0 for unknown types instead of erroring -- FIXED
- **Resolution:** Changed return type of `identity_type_to_proto` from `i32` to `Result<i32, DomainError>`. Unknown identity types now return `DomainError::Internal`. All call sites in `acl_service.rs` updated to propagate the error.

### 2.3 `resolve_user_groups` errors silently swallowed across all services
- **Status:** Open
- **Location:** Every `user_groups` resolution block across all services
- **Problem:** `.unwrap_or_default()` on `resolve_user_groups` means a transient DB failure silently treats the user as having no group memberships, causing hard-to-debug permission denials.
- **Fix:** Propagate the error: `self.identity.resolve_user_groups(uid).await.map_err(|e| DomainError::Internal(format!("failed to resolve groups: {e}")))?`.

### 2.4 `DateTime::from_timestamp` silently defaults to epoch
- **Status:** Open
- **Location:** `server/src/service/event_log_service.rs`
- **Problem:** `DateTime::from_timestamp(since.seconds, since.nanos as u32).unwrap_or_default()` silently converts invalid timestamps to the Unix epoch.
- **Fix:** Return `DomainError::InvalidArgument("invalid timestamp")` when `from_timestamp` returns `None`.

## 3. Stringly-Typed APIs (Medium Impact)

### 3.1 Hotlist permissions use raw strings instead of an enum
- **Status:** Open
- **Location:** `server/src/domain/permissions.rs`, all hotlist permission checks in `hotlist_service.rs`
- **Problem:** Component permissions have a proper `ComponentPermission` enum. Hotlist permissions are raw strings scattered as literals throughout the codebase. A typo would compile fine but silently fail.
- **Fix:** Create a `HotlistPermission` enum parallel to `ComponentPermission`.

### 3.2 Issue status, priority, severity, and type stored as strings in DB model
- **Status:** Open (low priority)
- **Location:** `server/src/db/row_mapping.rs`
- **Problem:** These fields have well-defined enum values in the proto but are stored as plain `String` in the DB row model.
- **Fix:** Consider creating Rust enums for these fields in the domain layer.

## 4. Missing Abstractions (Medium Impact)

### 4.1 `validate_entity_exists` pattern repeated for every entity type
- **Status:** Open
- **Location:** 6 near-identical "find by ID or return NotFound" helpers across services
- **Fix:** Create a generic helper: `async fn find_by_id<T: TryFrom<&Row, Error=RowError>>(conn: &C, table: &str, id: i64) -> Result<T, DomainError>`.

### 4.2 `count_by_query` pattern repeated for counting
- **Status:** Open
- **Location:** 4 count functions that fetch all rows and return `rows.len()`
- **Fix:** Use `COUNT(*)` if quiver supports it, or extract a generic `count_rows` helper.

### 4.3 Page size clamping boilerplate repeated in every List handler
- **Status:** Open
- **Location:** 6 List handlers
- **Fix:** Extract a `fn clamp_page_size(requested: i32) -> i32` helper.

## 5. Hardcoded Actor in Event Logging (Medium Impact)

### 5.1 `log_event` in issue_service.rs and hotlist_service.rs hardcodes actor to "system" -- FIXED
- **Resolution:** The shared `log_event` in `domain/event_log.rs` now takes `actor` as a parameter. All call sites in `issue_service.rs`, `hotlist_service.rs`, and `comment_service.rs` now pass the actual `user_id` (falling back to `"system"` only when `user_id` is `None`).

## 6. Dead / Unnecessary Code (Low Impact)

### 6.1 Unused `UpdateInput` structs in row_mapping
- **Status:** Open
- **Location:** `server/src/db/row_mapping.rs` (9 unused `*UpdateInput` structs)
- **Fix:** Remove unused structs. If generated by quiver-codegen, consider filtering the codegen output.

### 6.2 Unused imports in `acl_service.rs`
- **Status:** SKIPPED -- marginal finding; the imports are used within the file.

### 6.3 `crawler` and `ui` directories are outside the Rust workspace
- **Status:** SKIPPED -- intentionally separate TypeScript projects, not part of the Rust audit scope.

## 7. Consistency Issues (Low Impact)

### 7.1 Inconsistent pagination between `list_hotlists` and other List handlers
- **Status:** Open
- **Problem:** `list_hotlists` uses the correct N+1 over-fetch pattern while others use the "fetch N, check if len == N" pattern which may produce an incorrect `next_page_token` on the exact last page.
- **Fix:** Adopt the N+1 over-fetch pattern consistently across all List handlers.

### 7.2 `list_issues` on hotlist does not paginate
- **Status:** Open
- **Problem:** Unlike all other list endpoints, `HotlistService::list_issues` fetches all issues without pagination.
- **Fix:** Add `page_size` and `page_token` support.

### 7.3 `unmark_duplicate` does not clear `duplicateOfId`
- **Status:** Open
- **Location:** `server/src/service/issue_service.rs`
- **Problem:** When unmarking a duplicate, the `duplicateOfId` field is not set back to NULL.
- **Fix:** Add `.set("duplicateOfId", Value::Null)` to the update query.

## 8. Potential Data Integrity Issues (Medium Impact)

### 8.1 `mark_duplicate` does not validate status transition
- **Status:** Open
- **Location:** `server/src/service/issue_service.rs`
- **Problem:** `mark_duplicate` directly sets status to "DUPLICATE" without calling `status_machine::validate_transition()`.
- **Fix:** Add `status_machine::validate_transition(&issue.status, "DUPLICATE")?` before setting the status.

### 8.2 `get_accessible_component_ids` fetches ALL ACL entries
- **Status:** Open
- **Location:** `server/src/domain/permissions.rs`
- **Problem:** Fetches the entire `ComponentAcl`/`HotlistAcl` table to determine accessible IDs. Performance bottleneck at scale.
- **Fix:** Filter by identity match at the SQL level.

### 8.3 `list_components` filters accessible IDs in application code after fetching
- **Status:** Open
- **Location:** `server/src/service/component_service.rs`
- **Problem:** Fetches `page_size` components then filters in application code, potentially returning far fewer items than requested.
- **Fix:** Push the component ID filter into the SQL query.
