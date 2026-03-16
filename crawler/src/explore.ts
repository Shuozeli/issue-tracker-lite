/**
 * Phase 1: Exploration script for Google Issue Tracker docs.
 * Connects to CDP, navigates to the docs site, dumps HTML and structure
 * for offline analysis before building the actual crawler.
 */

import { chromium } from 'playwright';
import * as fs from 'fs/promises';
import * as path from 'path';

const CDP_ENDPOINT = process.env.CDP_ENDPOINT || 'http://localhost:9222';
const BASE_URL = 'https://developers.google.com/issue-tracker';
const EXPLORATION_DIR = path.resolve(process.cwd(), 'exploration');

async function ensureDir(dir: string): Promise<void> {
  await fs.mkdir(dir, { recursive: true });
}

async function explore(): Promise<void> {
  console.log(`[explore] Launching local browser (CDP at ${CDP_ENDPOINT} not reachable)`);
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();

  await ensureDir(EXPLORATION_DIR);

  // Step 1: Navigate to main docs page
  const page = await context.newPage();
  console.log(`[explore] Navigating to ${BASE_URL}`);
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 60000 });
  await page.waitForTimeout(3000);

  const finalUrl = page.url();
  console.log(`[explore] Final URL: ${finalUrl}`);

  // Step 2: Dump full HTML
  const html = await page.content();
  await fs.writeFile(path.join(EXPLORATION_DIR, 'main-page.html'), html);
  console.log('[explore] Saved main-page.html');

  // Step 3: Extract text content
  const textContent = await page.evaluate(() => {
    const main = document.querySelector('main') ?? document.querySelector('article') ?? document.body;
    return (main as HTMLElement).innerText;
  });
  await fs.writeFile(path.join(EXPLORATION_DIR, 'main-page-text.txt'), textContent);
  console.log('[explore] Saved main-page-text.txt');

  // Step 4: Extract page structure and all navigation links
  const structure = await page.evaluate((baseUrl: string) => {
    // Find all internal links that point to issue-tracker docs
    const allLinks = Array.from(document.querySelectorAll('a[href]')).map(a => ({
      href: a.getAttribute('href') ?? '',
      text: a.textContent?.trim().substring(0, 200) ?? '',
    }));

    const issueTrackerLinks = allLinks.filter(l =>
      l.href.includes('/issue-tracker') || l.href.startsWith('/issue-tracker')
    );

    // Find sidebar/navigation elements
    const navElements = Array.from(document.querySelectorAll('nav, [role="navigation"], .devsite-nav, devsite-toc, .devsite-section-nav'))
      .map(el => ({
        tag: el.tagName,
        className: el.className?.toString().substring(0, 100),
        childCount: el.children.length,
        innerHTML: el.innerHTML.substring(0, 2000),
      }));

    // Find data-test-id attributes
    const dataTestIds = Array.from(document.querySelectorAll('[data-test-id]'))
      .map(el => el.getAttribute('data-test-id'))
      .filter((v, i, a) => a.indexOf(v) === i);

    // Find main content containers
    const containers = Array.from(document.querySelectorAll('main, article, section, [role="main"], .devsite-article-body'))
      .map(el => ({
        tag: el.tagName,
        className: el.className?.toString().substring(0, 100),
        id: el.id,
        childCount: el.children.length,
      }));

    return {
      title: document.title,
      url: window.location.href,
      allLinksCount: allLinks.length,
      issueTrackerLinks,
      navElements,
      dataTestIds,
      containers,
    };
  }, BASE_URL);

  await fs.writeFile(
    path.join(EXPLORATION_DIR, 'structure.json'),
    JSON.stringify(structure, null, 2)
  );
  console.log('[explore] Saved structure.json');

  // Step 5: Extract the sidebar navigation specifically - this contains the doc tree
  const sidebarHtml = await page.evaluate(() => {
    const sidebar = document.querySelector('devsite-toc')
      ?? document.querySelector('.devsite-section-nav')
      ?? document.querySelector('nav.devsite-nav');
    return sidebar?.innerHTML ?? 'NO SIDEBAR FOUND';
  });
  await fs.writeFile(path.join(EXPLORATION_DIR, 'sidebar.html'), sidebarHtml);
  console.log('[explore] Saved sidebar.html');

  // Step 6: Deduplicate and collect all unique doc URLs
  const uniqueDocUrls = new Set<string>();
  for (const link of structure.issueTrackerLinks) {
    let href = link.href;
    // Normalize relative URLs
    if (href.startsWith('/')) {
      href = `https://developers.google.com${href}`;
    }
    // Remove fragments and query params for dedup
    const clean = href.split('#')[0]?.split('?')[0];
    if (clean && clean.includes('/issue-tracker')) {
      uniqueDocUrls.add(clean);
    }
  }

  const sortedUrls = Array.from(uniqueDocUrls).sort();
  await fs.writeFile(
    path.join(EXPLORATION_DIR, 'doc-urls.json'),
    JSON.stringify(sortedUrls, null, 2)
  );
  console.log(`[explore] Found ${sortedUrls.length} unique doc URLs, saved to doc-urls.json`);

  // Print summary
  console.log('\n--- Exploration Summary ---');
  console.log(`Title: ${structure.title}`);
  console.log(`Final URL: ${structure.url}`);
  console.log(`Total links on page: ${structure.allLinksCount}`);
  console.log(`Issue tracker doc links: ${structure.issueTrackerLinks.length}`);
  console.log(`Unique doc URLs: ${sortedUrls.length}`);
  console.log(`Nav elements found: ${structure.navElements.length}`);
  console.log(`Containers found: ${structure.containers.length}`);
  console.log('\nDoc URLs:');
  for (const url of sortedUrls) {
    console.log(`  ${url}`);
  }

  await page.close();
  await browser.close();
  console.log('\n[explore] Done. Review files in exploration/ directory.');
}

explore().catch(err => {
  console.error('[explore] Fatal error:', err);
  process.exit(1);
});
