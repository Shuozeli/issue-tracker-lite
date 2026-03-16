<!-- agent-updated: 2026-03-14T12:00:00Z -->

# UI Tasks

## Completed

- [x] React SPA with Ant Design dark theme (2026-03-12)
- [x] Express API proxy: REST -> gRPC translation (2026-03-12)
- [x] RTK Query API slice with tag-based cache invalidation (2026-03-12)
- [x] All pages: Dashboard, Issues, IssueDetail, Components, Hotlists, Search, Events, Login (2026-03-12)
- [x] Playwright E2E test suite (2026-03-13)
- [x] Playwright demo runner (headless/headed/record/remote CDP) (2026-03-13)
- [x] Browser DevTools console API (`window.api`) (2026-03-13)
- [x] Built-in demo console panel with toolbar buttons (2026-03-14)
- [x] Demo data seeding (6 components, 12 issues, comments, hotlists) (2026-03-14)
- [x] Demo scenarios: quickstart, triage, lifecycle, comments, search, full (2026-03-14)
- [x] Step verification gates (waitForCondition, waitForModalClose, waitForPageReady) (2026-03-14)
- [x] Comment edit, revision history, hide/unhide (2026-03-13)
- [x] Centralized testIds for E2E stability (2026-03-13)

## Pending

- [ ] Responsive layout for mobile/tablet
- [ ] Component tree view (hierarchical display with expand/collapse)
- [ ] Issue relationship UI (parent-child, blocking, duplicate visualization)
- [ ] Hotlist drag-and-drop reordering
- [ ] ACL management UI (component/hotlist permissions)
- [ ] Real-time updates (WebSocket or polling for live issue changes)
- [ ] Dark/light theme toggle
- [ ] Keyboard shortcuts for common actions (beyond Ctrl+` for console)
- [ ] Error boundary with user-friendly error pages

## Known Issues

- [ ] **Bug**: Demo overlay can get stuck if a modal close is not detected within timeout
- [ ] **Bug**: Demo console log filter `useMemo` re-renders on every keystroke (minor perf)
