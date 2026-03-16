<!-- agent-updated: 2026-03-08T00:00:00Z -->

# Google Issue Tracker Docs Crawler

Crawls all documentation from `https://developers.google.com/issue-tracker` for offline analysis and reference when building the issue tracker project.

## Dependencies

- Node.js 22+
- pnpm
- Playwright (with Chromium)

## Quick Start

```bash
pnpm install
npx playwright install chromium

# Phase 1: Explore the site structure
npx tsx src/explore.ts

# Phase 2: Crawl all docs
npx tsx src/crawl.ts
```

## Output

- `exploration/` - Raw HTML dumps and structural analysis from phase 1
- `docs-output/markdown/` - Individual markdown files per doc page
- `docs-output/html/` - Raw HTML article bodies
- `docs-output/all-docs.txt` - Combined plain text of all pages
- `docs-output/index.json` - Page index with metadata
