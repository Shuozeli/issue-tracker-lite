<!-- agent-updated: 2026-03-19T00:00:00Z -->

# Project Tasks / Roadmap

## Completed

- [x] Core server with 9 gRPC services (Component, Issue, Comment, Hotlist, Search, EventLog, ACL, Group, Health) (2026-03-14)
- [x] CLI client (`it`) with full command coverage (2026-03-14)
- [x] React 19 web UI with Ant Design dark theme and demo console (2026-03-14)
- [x] Security hardening: auth validation, input sanitization, page size caps (2026-03-14)
- [x] Quiver ORM migration with connection pooling (SqlitePool, 4 connections) and codegen (2026-03-15)
- [x] Row deserialization codegen (TryFrom<&Row> generation from schema.quiver) (2026-03-15)
- [x] Proto enum conversion via prost as_str_name/from_str_name (2026-03-15)
- [x] Integration tests split into 9 files (167 tests passing) (2026-03-15)
- [x] Documentation refresh (API.md, tasks.md, CHANGELOG.md) (2026-03-16)
- [x] Security hardening: ACL RPCs require auth + ADMIN permission, hotlist create requires auth, owner override, bootstrap logic, 13 new ACL tests, e2e_security_hardening test, security demo pipeline (2026-03-16)

## Pending

- [ ] Rate limiting and request size limits on gRPC server
- [ ] Identity crate row mapping migration to codegen
- [ ] Codegen build step (auto-regenerate row_mapping.rs from schema.quiver on build)
- [ ] Saved Searches and Bookmark Groups implementation
- [ ] Custom Fields and Templates
- [ ] Notification system (edit classification, dispatch)
- [ ] PostgreSQL migration option
