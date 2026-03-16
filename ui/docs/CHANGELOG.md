<!-- agent-updated: 2026-03-14T12:00:00Z -->

# Changelog

## [0.1.0] - 2026-03-14

### Added
- Built-in demo console panel with toolbar buttons for 6 scenarios
- Demo data seeding: 6 components, 12 issues, comments, hotlists via direct API calls
- Step verification gates: waitForCondition, waitForModalClose, waitForPageReady, waitForUrlChange
- Overlay dismissal recovery for stuck demos
- Pause/Resume/Stop controls for demo playback
- Log filtering and clear in demo console
- Console toggle via `Ctrl+\`` keyboard shortcut
- Console accessible on login page (floating button)

### Fixed
- `useLog` hook now returns array snapshot (not mutable reference) for correct `useMemo` behavior
- Unused import cleanup in demoConsole.ts

## [0.0.2] - 2026-03-13

### Added
- Playwright E2E test suite (`e2e/issuetracker.spec.ts`)
- Playwright demo runner with headless, headed, record, and remote CDP modes
- Browser DevTools console API (`window.api` bindings)
- Comment editing with revision history
- Comment hide/unhide functionality
- Centralized `testIds.ts` for E2E test stability

## [0.0.1] - 2026-03-12

### Added
- Initial React 19 SPA with Ant Design 5 dark theme
- Express API proxy: REST-to-gRPC translation for all services
- RTK Query API slice with tag-based cache invalidation
- Pages: Dashboard, Issues, IssueDetail, Components, Hotlists, Search, Events, Login
- Sidebar navigation with route-based active state
- Login page with user ID input (identity header, no real auth)
- Vite dev server with `/api` proxy to Express
