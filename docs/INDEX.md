<!-- agent-updated: 2026-03-19T00:00:00Z -->

# Issue Tracker Lite -- Documentation Index

A rebuild of Google Issue Tracker. Rust gRPC server + CLI client + React web UI,
backed by SQLite via Quiver ORM.

Use this index to navigate the project documentation. Start with the architecture
overview, then drill into the area you need.

---

## 1. Getting Started

| Document | Path | Description |
|---|---|---|
| **Usage Guide** | [`docs/usage.md`](usage.md) | Server setup, CLI commands, search query language, web UI |
| **Codelabs** | [`docs/codelabs.md`](codelabs.md) | Step-by-step tutorials: bug triage, team ACLs, search + hotlists |
| **API Reference** | [`docs/API.md`](API.md) | All 9 gRPC services, RPC signatures, enums, error codes |

## 2. Architecture & Design

| Document | Path | Description |
|---|---|---|
| **Architecture Overview** | [`docs/specs/architecture.md`](specs/architecture.md) | System diagram, entity relationships, data flows, tech stack |
| **Implementation Phases** | [`docs/specs/phases.md`](specs/phases.md) | All 10 phases with deliverables, verification notes, and implementation details |
| **UI Architecture** | [`ui/docs/architecture.md`](../ui/docs/architecture.md) | Three-tier browser architecture, directory structure, data flow, design decisions |

## 3. Project Status

| Document | Path | Description |
|---|---|---|
| **Tasks** | [`docs/tasks.md`](tasks.md) | Roadmap: completed items and pending work |
| **Changelog** | [`docs/CHANGELOG.md`](CHANGELOG.md) | Version history |

## 4. Domain Specifications

Detailed specs for each domain entity, derived from the official Google Issue Tracker
documentation (crawled to `crawler/docs-output/markdown/`).

| Document | Path | Description |
|---|---|---|
| Issue | [`docs/specs/issue.md`](specs/issue.md) | Core work item: fields, status lifecycle, relationships, field masks |
| Component | [`docs/specs/component.md`](specs/component.md) | Hierarchical container: tree structure, ACLs, templates, custom fields |
| Access Control | [`docs/specs/access-control.md`](specs/access-control.md) | Permission model: component ACLs, expanded access, per-issue restrictions |
| Hotlist | [`docs/specs/hotlist.md`](specs/hotlist.md) | Curated issue lists: membership, ordering, ACLs |
| Search | [`docs/specs/search.md`](specs/search.md) | Query language: field filters, operators, pagination, sorting |
| Notification | [`docs/specs/notification.md`](specs/notification.md) | Edit classification, per-role notification levels, dispatch |
| Saved Search | [`docs/specs/saved-search.md`](specs/saved-search.md) | Stored queries with dynamic results |
| Bookmark Group | [`docs/specs/bookmark-group.md`](specs/bookmark-group.md) | Composite views of hotlists and saved searches |
| Tracker | [`docs/specs/tracker.md`](specs/tracker.md) | Top-level organizational unit |
| User Settings | [`docs/specs/user-settings.md`](specs/user-settings.md) | Per-user preferences |
| Database Schema | [`docs/specs/database-schema.md`](specs/database-schema.md) | Reference schema design |

## 5. Server (Rust gRPC)

The server lives in `server/` within the workspace root. Key files:

| Area | Path | Description |
|---|---|---|
| Entry point | `server/src/main.rs` | Tonic server startup, SqlitePool init, service registration |
| Proto definitions | `proto/issuetracker/v1/*.proto` | gRPC service definitions (component, issue, comment, hotlist, search, acl, common) |
| Quiver schema | `schema.quiver` | Database models (13 tables), used by `quiver generate` for codegen |
| Generated models | `server/src/db/row_mapping.rs` | Quiver-generated model structs + `TryFrom<&Row>` |
| Services | `server/src/service/*.rs` | gRPC handlers: component, issue, comment, hotlist, search, event_log, acl |
| Domain logic | `server/src/domain/*.rs` | Status machine, permissions engine, query parser |
| Integration tests | `server/tests/` | 180 tests split across 10 files by service, each with isolated temp SQLite DB |

## 6. CLI (Rust gRPC Client)

The CLI lives in `cli/`. Key files:

| Area | Path | Description |
|---|---|---|
| Entry point | `cli/src/main.rs` | Clap arg parsing, gRPC client setup |
| Commands | `cli/src/commands/*.rs` | Subcommands: component, issue, comment, hotlist, search, events, acl |
| Output formatting | `cli/src/output.rs` | Table and JSON output |

## 7. Web UI (React SPA)

The UI lives in `ui/`. It has its own documentation set:

| Document | Path | Description |
|---|---|---|
| **README** | [`ui/docs/README.md`](../ui/docs/README.md) | Quick start, env vars, scripts |
| **Architecture** | [`ui/docs/architecture.md`](../ui/docs/architecture.md) | Three-tier design, directory layout, data flow |
| **API Reference** | [`ui/docs/API.md`](../ui/docs/API.md) | All REST endpoints, request/response format, error codes |
| **Tasks** | [`ui/docs/tasks.md`](../ui/docs/tasks.md) | Completed work, pending items, known bugs |
| **Changelog** | [`ui/docs/CHANGELOG.md`](../ui/docs/CHANGELOG.md) | Version history |

Key directories:

```
ui/
  src/
    api/          # Types, browser console bindings, demo executor
    components/   # Shared components (DemoConsole, format helpers)
    pages/        # Route pages (Dashboard, Issues, Components, etc.)
    store/        # Redux store, RTK Query API slice, auth slice
  server/         # Express REST-to-gRPC proxy
  e2e/            # Playwright tests and demo runner
```

## 8. Demo System

Demo systems:

| System | Path | Description |
|---|---|---|
| **CLI Demo** | `demo/src/` | Rust binary that drives the gRPC server through scenarios via CLI-like calls |
| **Browser Demo** | `ui/src/api/demoConsole.ts` | In-browser executor that seeds data and auto-drives the UI |
| **Playwright Demo** | `ui/e2e/demo-runner.ts` | Headless/headed browser automation via Playwright |
| **Demo Console UI** | `ui/src/components/DemoConsole.tsx` | Built-in panel with toolbar buttons, log viewer (toggle: `Ctrl+\``) |

Browser demo scenarios: quickstart, triage, lifecycle, comments, search, full.
CLI demo pipelines: quickstart, hierarchy, hotlists, access_control, search, full_lifecycle, groups, security.

## 9. Crawler (Reference Data)

| Area | Path | Description |
|---|---|---|
| Crawler code | `crawler/` | Playwright-based crawler for Google Issue Tracker docs |
| Crawled output | `crawler/docs-output/markdown/` | 47 markdown files from developers.google.com/issue-tracker |
| Crawler docs | `crawler/docs/README.md` | Crawler setup and usage |

## 10. Build & Run

### Prerequisites
- Rust toolchain (cargo)
- Node.js + pnpm

### Server + CLI
```bash
# From workspace root
DATABASE_URL="file:./dev.db" cargo run --bin issuetracker-server
cargo run --bin it -- ping
```

### Web UI
```bash
cd ui
pnpm install
pnpm dev          # Starts Vite (5173) + Express proxy (3001)
```

### Tests
```bash
cargo test -p issuetracker-server  # 180 gRPC integration tests (10 files)
cd ui && pnpm test:e2e            # Playwright E2E tests
```

## 11. Key Patterns

| Pattern | Where | Description |
|---|---|---|
| All DB ops in transactions | All services | Every read and write wrapped in transaction via `pool.acquire() -> begin()` |
| Event logging | All mutations | Every state change appends to `event_log` table within the same transaction |
| Status machine | `domain/status_machine.rs` | Enforces valid status transitions, auto-transitions on assignee set/clear |
| Permission hierarchy | `domain/permissions.rs` | Component ACL inheritance up parent chain, expanded access, role implication graph |
| RTK Query cache | `store/api.ts` | Tag-based invalidation: mutations invalidate tags, queries refetch automatically |
| Centralized test IDs | `ui/src/testIds.ts` | Structured object shared between React components and Playwright tests |
| REST-to-gRPC proxy | `ui/server/` | Express translates REST calls to gRPC, maps error codes to HTTP status |

## 12. Configuration

| Variable | Default | Used By | Description |
|---|---|---|---|
| `DATABASE_URL` | (required) | Server | SQLite path, e.g. `file:./dev.db` |
| `LISTEN_ADDR` | `0.0.0.0:50051` | Server | gRPC listen address |
| `IT_SERVER_ADDR` | `localhost:50051` | CLI, UI proxy | gRPC server address |
| `API_PORT` | `3001` | UI proxy | Express proxy port |
| `BIND_HOST` | `0.0.0.0` | UI proxy | Express proxy bind host |
