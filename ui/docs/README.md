<!-- agent-updated: 2026-03-14T12:00:00Z -->

# IssueTracker UI

Web frontend for the IssueTracker system. React 19 SPA with Ant Design, backed by a
gRPC-to-REST proxy server.

## Dependencies

- **Runtime:** React 19, React Router 7, Ant Design 5, Redux Toolkit (RTK Query)
- **Dev:** Vite 6, TypeScript 5, Playwright, Express (API proxy)
- **Backend:** Rust gRPC server on port 50051 (see `../server/`)
- **Package manager:** pnpm

## Quick Start

```bash
# Prerequisites: Rust gRPC server running on localhost:50051

# Install dependencies
pnpm install

# Start dev server (Vite + API proxy concurrently)
pnpm dev

# Or start individually:
pnpm dev:ui     # Vite on http://0.0.0.0:5173
pnpm dev:api    # Express proxy on http://0.0.0.0:3001
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `IT_SERVER_ADDR` | `localhost:50051` | gRPC server address for API proxy |
| `API_PORT` | `3001` | Express proxy listen port |
| `BIND_HOST` | `0.0.0.0` | Express proxy bind host |

## Demo System

The UI includes a built-in demo console panel (toggle with `Ctrl+\`` or the toolbar button).
It seeds demo data and auto-drives the UI through scenarios.

```bash
# E2E demo via Playwright (headless)
pnpm demo

# Headed browser
pnpm demo:headed

# Record video
pnpm demo:record

# Remote Chrome via CDP
pnpm demo:remote
```

## Testing

```bash
# E2E tests
pnpm test:e2e
pnpm test:e2e:headed
```
