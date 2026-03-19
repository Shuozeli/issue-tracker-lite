<!-- agent-updated: 2026-03-19T00:00:00Z -->

# Issue Tracker Lite

A rebuild of [Google Issue Tracker](https://developers.google.com/issue-tracker) as a Rust gRPC server + CLI client + React web UI, backed by SQLite via [Quiver ORM](https://github.com/Shuozeli/quiver-orm).

## Features

- **9 gRPC services**: Component, Issue, Comment, Hotlist, Search, EventLog, ACL, Group, Health
- **Hierarchical components** with inherited ACLs and expanded access
- **Full issue lifecycle**: 11 statuses, auto-transitions, parent/child/blocking/duplicate relationships
- **Search query language**: structured filters (`status:open priority:P0`), keyword search, pagination
- **Group-based access control**: nested groups, permission inheritance up component tree
- **Auth enforcement**: all RPCs require `x-user-id` header; ACL RPCs require ADMIN permission
- **Event log**: append-only audit trail for all mutations
- **CLI client**: full-featured `it` command for all operations
- **React SPA**: 8 pages with RTK Query, Ant Design dark theme, built-in demo console
- **180 integration tests** across 10 test files, each with isolated temp SQLite DB

## Quick Start

### Prerequisites

- Rust toolchain (1.85+)
- Node.js + pnpm (for web UI only)

### Server + CLI

```bash
# Build everything
cargo build

# Start the server
DATABASE_URL="file:./dev.db" cargo run --bin issuetracker-server

# Use the CLI (in another terminal)
cargo run --bin it -- ping
cargo run --bin it -- component create "My Project"
cargo run --bin it -- issue create -c 1 -t "First bug" -p P2 --type BUG
```

### Web UI

```bash
cd ui
pnpm install
pnpm dev    # Vite (5173) + Express proxy (3001)
```

### Tests

```bash
cargo test -p issuetracker-server --tests   # 180 integration tests
cargo test -p issuetracker-server --test e2e # E2E tests (requires CLI binary built)
cd ui && pnpm test:e2e                       # Playwright tests
```

## Architecture

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
SQLite (Quiver ORM, 4-connection pool)
```

### Workspace Layout

```
issue-tracker-lite/
  schema.quiver              # Quiver ORM schema (13 models)
  proto/
    issuetracker/v1/         # gRPC service definitions (8 .proto files)
    identity/v1/             # Group service proto (1 .proto file)
    identity/v1/             # Group service proto
  server/
    src/
      main.rs                # Server entry point (pool init, service registration)
      lib.rs                 # Module exports, proto includes
      db/
        mod.rs               # SqlitePool initialization, DDL generation
        row_mapping.rs       # Generated model structs + TryFrom<&Row>
      service/               # gRPC service implementations (9 services)
      domain/                # Business logic (status machine, permissions, query parser)
    tests/
      common/mod.rs          # Shared test fixtures (TestFixture, helpers)
      component_tests.rs     # 11 tests
      issue_tests.rs         # 38 tests
      comment_tests.rs       # 5 tests
      hotlist_tests.rs       # 9 tests
      search_tests.rs        # 13 tests
      event_log_tests.rs     # 6 tests
      acl_tests.rs           # 44 tests
      group_tests.rs         # 43 tests
      validation_tests.rs    # 11 tests
      e2e.rs                 # 8 E2E tests
  cli/                       # CLI client (clap-based)
  demo/                      # Demo runner (in-process server + scenarios)
  identity/                  # Group/member management library
  test-utils/                # Shared test fixtures (TestFixture, helpers)
  ui/                        # React SPA frontend
  crawler/                   # Google Issue Tracker docs crawler
  docs/
    INDEX.md                 # Documentation navigation
    specs/                   # Domain specifications
```

## Configuration

| Variable | Default | Used By | Description |
|----------|---------|---------|-------------|
| `DATABASE_URL` | (required) | Server | SQLite path, e.g. `file:./dev.db` |
| `LISTEN_ADDR` | `0.0.0.0:50051` | Server | gRPC listen address |
| `IT_SERVER_ADDR` | `localhost:50051` | CLI | gRPC server address |

## Tech Stack

| Concern | Choice |
|---------|--------|
| Server | Rust, tonic (gRPC), prost |
| ORM | [Quiver ORM](https://github.com/Shuozeli/quiver-orm) (SQLite, connection pooling, codegen) |
| CLI | clap (derive) |
| UI | React 19, Ant Design 5, RTK Query, Vite 6 |
| API Proxy | Express (REST-to-gRPC) |
| E2E Testing | Playwright |

## Documentation

See [`docs/INDEX.md`](docs/INDEX.md) for the full documentation index, including:
- [Architecture Overview](docs/specs/architecture.md)
- [Implementation Phases](docs/specs/phases.md)
- Domain specs: [Issue](docs/specs/issue.md), [Component](docs/specs/component.md), [Access Control](docs/specs/access-control.md), [Hotlist](docs/specs/hotlist.md), [Search](docs/specs/search.md)
