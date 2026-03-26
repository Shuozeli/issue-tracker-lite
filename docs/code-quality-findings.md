<!-- agent-updated: 2026-03-26T08:30:00Z -->

# Code Quality Findings

## 1. Duplication (High Impact)

### 1.1 `parse_timestamp` duplicated across 6 files -- FIXED (previous session)
- **Resolution:** Extracted to `server/src/domain/timestamp.rs`.

### 1.2 `log_event` helper duplicated across 3 files -- FIXED (previous session)
- **Resolution:** Extracted to `server/src/domain/event_log.rs`.

### 1.3 `user_groups` resolution boilerplate repeated in every service method
- **Status:** FIXED
- **Location:** Every RPC handler across all service files (25+ occurrences)
- **Problem:** The 7-line `match user_id.as_deref()` block for resolving groups was copy-pasted into every handler.
- **Fix:** Extracted `resolve_user_groups` free function in `domain/permissions.rs`.

### 1.4 Transaction acquisition boilerplate repeated in every handler
- **Status:** FIXED
- **Location:** Every RPC handler across all service files (30+ occurrences)
- **Problem:** The `acquire().await.map_err()?; conn.begin().await.map_err()?;` pattern repeated identically.
- **Fix:** Extracted `begin_tx` helper function in `db/mod.rs`.

### 1.5 Status list duplication between `status_machine.rs`, `issue_service.rs`, and `query_parser.rs`
- **Status:** FIXED
- **Location:** Three separate places defined open/closed status lists.
- **Fix:** Made `status_machine::OPEN_STATUSES` and `CLOSED_STATUSES` public; `issue_service.rs` and `query_parser.rs` now reference them.

### 1.6 Row mapping duplication between `identity` crate and `server` crate
- **Status:** Open (deferred - cross-crate refactor, low ROI)
- **Location:** `identity/src/row_mapping.rs` and `server/src/db/row_mapping.rs`

## 2. Silent Failures (High Impact)

### 2.1 JSON permission deserialization silently defaults to empty -- FIXED (previous session)

### 2.2 `identity_type_to_proto` returns 0 for unknown types -- FIXED (previous session)

### 2.3 `resolve_user_groups` errors silently swallowed across all services
- **Status:** FIXED
- **Location:** Every `user_groups` resolution block
- **Problem:** `.unwrap_or_default()` on `resolve_user_groups` silently treated DB failures as "no groups".
- **Fix:** The new `resolve_user_groups` helper propagates errors as `DomainError::Internal`.

### 2.4 `DateTime::from_timestamp` silently defaults to epoch
- **Status:** FIXED
- **Location:** `server/src/service/event_log_service.rs:66,73`
- **Problem:** `unwrap_or_default()` silently converted invalid timestamps to Unix epoch.
- **Fix:** Returns `DomainError::InvalidArgument` when `from_timestamp` returns `None`.

## 3. Stringly-Typed APIs (Medium Impact)

### 3.1 Hotlist permissions use raw strings instead of an enum
- **Status:** FIXED
- **Location:** `server/src/domain/permissions.rs`, all hotlist permission checks
- **Problem:** Component permissions had a proper enum, hotlist permissions were raw string literals.
- **Fix:** Created `HotlistPermission` enum with `parse`, `as_str`, `from_proto`, `to_proto` methods. All call sites updated.

### 3.2 Issue status, priority, severity, and type stored as strings in DB model
- **Status:** Open (low priority - proto enums already validate at the service boundary)

## 4. Missing Abstractions (Medium Impact)

### 4.1 `validate_entity_exists` pattern repeated for every entity type
- **Status:** Open (deferred - each entity has slightly different error messages)

### 4.2 `count_by_query` pattern: fetches all rows to count
- **Status:** Open (deferred - quiver ORM limitation; would require raw SQL COUNT(*))

### 4.3 Page size clamping boilerplate
- **Status:** FIXED
- **Fix:** Extracted `fn clamp_page_size(requested: i32) -> i32` helper in `domain/types.rs`.

## 5. Data Integrity Issues (Medium Impact)

### 5.1 `unmark_duplicate` does not clear `duplicateOfId`
- **Status:** Open (deferred - quiver ORM does not support `Value::Null` in update `.set()`)
- **Location:** `server/src/service/issue_service.rs`
- **Note:** The status change is sufficient for correctness since `duplicateOfId` is only meaningful when status is DUPLICATE.

### 5.2 `mark_duplicate` does not validate status transition
- **Status:** FIXED
- **Location:** `server/src/service/issue_service.rs`
- **Fix:** Added `status_machine::validate_transition(&issue.status, "DUPLICATE")?` before setting status.

## 6. Dead / Unnecessary Code (Low Impact)

### 6.1 Unused `UpdateInput` structs in row_mapping
- **Status:** Open (deferred - generated code, removing would break codegen contract)

### 6.2 `crawler` and `ui` directories outside Rust workspace
- **Status:** SKIPPED -- separate TypeScript projects.

## 7. Unsafe Patterns

### 7.1 `unwrap()` in non-test code in identity crate
- **Status:** FIXED
- **Location:** `identity/src/sqlite_provider.rs:851`
- **Problem:** `.unwrap()` after an `is_some()` check could panic if racing conditions occurred.
- **Fix:** Changed to use the already-verified `Some` value from the preceding `if let` check.

## 8. Consistency Issues (Low Impact)

### 8.1 Inconsistent pagination (N+1 overfetch vs len==N)
- **Status:** Open (deferred - functional, not a bug, would touch many files)

### 8.2 `list_issues` on hotlist does not paginate
- **Status:** Open (deferred - functional limitation, not a bug)

### 8.3 `get_accessible_component_ids` fetches ALL ACL entries
- **Status:** Open (deferred - performance optimization, not a correctness issue)
