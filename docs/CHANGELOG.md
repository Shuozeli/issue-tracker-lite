<!-- agent-updated: 2026-03-19T00:00:00Z -->

# Changelog

All notable changes to this project are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.1.1] - 2026-03-16

### Added

- ACL RPCs now require authentication and ADMIN permission on the resource
- Bootstrap logic: first ACL entry on a resource allowed without permission check (still requires auth)
- Hotlist create requires auth, overrides owner to authenticated user, auto-grants HOTLIST_ADMIN to creator
- Hotlist add_issue overrides added_by to authenticated user
- CheckComponentPermission now resolves target user's groups (bug fix)
- Error messages redacted in RemoveComponentAcl/RemoveHotlistAcl
- 13 new integration tests for auth enforcement and error redaction
- E2E test: e2e_security_hardening (unauthenticated denial, privilege escalation, owner override, permission enforcement)
- Demo pipeline: `it-demo security` (18 steps)

## [0.1.0] - 2026-03-16

### Added

- 9 gRPC services: Component, Issue, Comment, Hotlist, Search, EventLog, ACL, Group, Health
- CLI client (`it`) with full command coverage for all services
- React 19 web UI with Ant Design dark theme and demo console
- Quiver ORM with SqlitePool (4 connections) and TryFrom<&Row> codegen from schema.quiver
- 180 integration tests across 10 test files (component, issue, comment, hotlist, search, event_log, acl, group, validation, e2e)
- Security hardening: x-user-id auth validation (max 256 chars, restricted charset), page size caps (100), LIKE wildcard escaping, strict page_token parsing, redacted error messages
- Proto enum conversion via prost as_str_name/from_str_name (no manual match arms)
- Google AIP-compliant API design: pagination (page_size/page_token), field masks for partial updates, standard method patterns (Get, List, Create, Update, Delete)
- Issue relationships: parent/child hierarchy, blocking/blocked-by, duplicate marking
- Comment edit history with revision tracking
- Hotlist issue ordering with reorder support
- Component and hotlist ACL with user, group, and public identity types
- Identity group service with nested group membership and transitive resolution
- Event log for all mutations (structured JSON payloads, time-series queryable)
