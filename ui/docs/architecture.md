<!-- agent-updated: 2026-03-14T12:00:00Z -->

# UI Architecture

## Overview

Three-tier browser application: React SPA -> Express REST proxy -> Rust gRPC server.

```
Browser (React 19 + Ant Design 5)
  |
  | fetch("/api/...")
  v
Express API Proxy (server/index.ts, port 3001)
  |
  | @grpc/grpc-js
  v
Rust gRPC Server (tonic, port 50051)
  |
  v
SQLite (prisma-rs)
```

Vite dev server proxies `/api` requests to the Express server, so the browser only
talks to one origin.

## Directory Structure

```
ui/
  src/
    App.tsx                  # Root layout: sidebar nav, header, routes, demo console toggle
    main.tsx                 # React DOM entry, Redux provider, BrowserRouter
    testIds.ts               # Centralized data-testid constants for E2E tests
    vite-env.d.ts

    api/
      types.ts               # Shared TypeScript types (Component, Issue, Comment, etc.)
      consoleBindings.ts     # Browser DevTools console API (window.api)
      demoConsole.ts         # In-browser demo executor: seeding, UI automation, scenarios

    components/
      DemoConsole.tsx         # Built-in demo console panel (toolbar + log viewer)
      formatHelpers.ts        # Display formatting (dates, enums, priorities)

    pages/
      LoginPage.tsx           # User ID input (no real auth -- identity header only)
      DashboardPage.tsx       # Overview: recent issues, component counts
      IssuesPage.tsx          # Issue list with create modal
      IssueDetailPage.tsx     # Single issue view: metadata, comments, status updates
      ComponentsPage.tsx      # Component CRUD with hierarchy display
      HotlistsPage.tsx        # Hotlist management, issue membership
      SearchPage.tsx          # Query-based issue search
      EventsPage.tsx          # Event log viewer with filters

    store/
      index.ts                # Redux store configuration
      api.ts                  # RTK Query API slice (all endpoints)
      authSlice.ts            # Auth state (userId from login)

  server/
    index.ts                  # Express API proxy: REST -> gRPC translation
    grpcClient.ts             # @grpc/grpc-js client setup, proto loading

  e2e/
    issuetracker.spec.ts      # Playwright E2E test suite
    demo-runner.ts            # Playwright-based demo runner (headless/headed/remote)

  vite.config.ts              # Vite: React plugin, /api proxy to :3001
  playwright.config.ts        # Playwright configuration
  package.json                # Scripts, dependencies
```

## Data Flow

### Read Path

```
Page Component
  -> useListXxxQuery() / useGetXxxQuery()   (RTK Query hook)
  -> fetchBaseQuery("/api/...")               (auto x-user-id header)
  -> Vite proxy -> Express proxy
  -> grpc-js client.methodName()
  -> gRPC server
  -> Response flows back, RTK Query caches + provides tags
```

### Write Path

```
User action (form submit, button click)
  -> useCreateXxxMutation() / useUpdateXxxMutation()
  -> fetchBaseQuery POST/PATCH/DELETE
  -> Express proxy -> gRPC
  -> Response: RTK Query invalidates tags -> dependent queries refetch
```

### Authentication

No real authentication. The LoginPage captures a user ID string, stored in Redux
(`authSlice`). RTK Query's `prepareHeaders` attaches it as `x-user-id` header on
every request. The Express proxy forwards it as gRPC metadata. The gRPC server uses
it for permission checks (gradual rollout: missing header = anonymous = allowed).

## Key Design Decisions

### RTK Query for Server State

All server data is managed through RTK Query's cache with tag-based invalidation.
No manual state management for server data. Tags: `Component`, `Issue`, `Comment`,
`Hotlist`, `Event`.

### Express Proxy (not direct gRPC-Web)

The browser cannot speak native gRPC. Instead of gRPC-Web (which requires Envoy or
similar), a thin Express server translates REST to gRPC using `@grpc/grpc-js` with
dynamic proto loading. This keeps the setup simple and avoids extra infrastructure.

### Built-in Demo Console

The demo system runs entirely in-browser (no Playwright needed for casual demos).
`demoConsole.ts` seeds data via `fetch()` API calls, then manipulates the DOM to
simulate user interactions (clicking nav items, filling forms, selecting dropdowns).
The `DemoConsole.tsx` panel provides toolbar buttons and a log viewer. Toggle with
`Ctrl+\`` or the header button.

### Centralized Test IDs

`testIds.ts` exports a structured object of `data-testid` values used by both
React components and E2E tests. This prevents string drift between the two.
