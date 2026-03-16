/**
 * Phase 2: Crawl all Google Issue Tracker docs.
 *
 * Strategy:
 * 1. Start from known section entry pages
 * 2. On each page, discover new /issue-tracker/* links from left nav
 * 3. Extract content from each unique page
 * 4. Save as individual markdown files
 */

import { chromium, type Browser, type BrowserContext, type Page } from 'playwright';
import * as fs from 'fs/promises';
import * as path from 'path';

const BASE_URL = 'https://developers.google.com';
const DOCS_PREFIX = '/issue-tracker';
const OUTPUT_DIR = path.resolve(process.cwd(), 'docs-output');
const DELAY_MS = 2000;

interface DocPage {
  url: string;
  title: string;
  breadcrumbs: string[];
  content: string;
  htmlContent: string;
}

// Known seed URLs from exploration
const SEED_URLS = [
  '/issue-tracker',
  '/issue-tracker/guides/access-ui',
  '/issue-tracker/concepts/components',
  '/issue-tracker/concepts/access-control',
  '/issue-tracker/guides/partner-access',
  '/issue-tracker/guides/partner-domains',
  '/issue-tracker/references/glossary-of-fields',
  '/issue-tracker/references/faq',
];

async function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function discoverLinks(page: Page): Promise<string[]> {
  return page.evaluate((prefix: string) => {
    const links = new Set<string>();
    document.querySelectorAll('a[href]').forEach(a => {
      let href = a.getAttribute('href') ?? '';
      // Remove fragments and query params
      href = href.split('#')[0]?.split('?')[0] ?? '';
      if (href.startsWith(prefix) && href.length > prefix.length) {
        links.add(href);
      }
    });
    return Array.from(links);
  }, DOCS_PREFIX);
}

async function extractContent(page: Page): Promise<DocPage> {
  return page.evaluate(() => {
    const title = document.querySelector('h1')?.textContent?.trim()
      ?? document.title.split('|')[0]?.trim()
      ?? '';

    // Get breadcrumbs
    const breadcrumbs = Array.from(document.querySelectorAll('.devsite-breadcrumb-link'))
      .map(el => el.textContent?.trim() ?? '')
      .filter(t => t.length > 0);

    // Get main article content
    const articleBody = document.querySelector('.devsite-article-body');
    const content = (articleBody as HTMLElement)?.innerText ?? '';
    const htmlContent = articleBody?.innerHTML ?? '';

    return {
      url: window.location.pathname,
      title,
      breadcrumbs,
      content,
      htmlContent,
    };
  });
}

function htmlToMarkdown(html: string, title: string): string {
  // Simple HTML to markdown conversion
  let md = `# ${title}\n\n`;

  // Process the HTML content
  let text = html;

  // Replace headers
  text = text.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
  text = text.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
  text = text.replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n');
  text = text.replace(/<h4[^>]*>(.*?)<\/h4>/gi, '#### $1\n\n');

  // Replace code blocks
  text = text.replace(/<pre[^>]*><code[^>]*>(.*?)<\/code><\/pre>/gis, '```\n$1\n```\n\n');
  text = text.replace(/<code[^>]*>(.*?)<\/code>/gi, '`$1`');

  // Replace links
  text = text.replace(/<a[^>]*href="([^"]*)"[^>]*>(.*?)<\/a>/gi, '[$2]($1)');

  // Replace lists
  text = text.replace(/<li[^>]*>(.*?)<\/li>/gi, '- $1\n');
  text = text.replace(/<\/?[uo]l[^>]*>/gi, '\n');

  // Replace paragraphs and breaks
  text = text.replace(/<p[^>]*>(.*?)<\/p>/gis, '$1\n\n');
  text = text.replace(/<br\s*\/?>/gi, '\n');

  // Replace strong/em
  text = text.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**');
  text = text.replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*');

  // Replace tables (basic)
  text = text.replace(/<table[^>]*>(.*?)<\/table>/gis, (_, tableContent: string) => {
    const rows = tableContent.match(/<tr[^>]*>(.*?)<\/tr>/gis) ?? [];
    return rows.map((row: string) => {
      const cells = (row.match(/<t[hd][^>]*>(.*?)<\/t[hd]>/gis) ?? [])
        .map((cell: string) => cell.replace(/<\/?t[hd][^>]*>/gi, '').trim());
      return `| ${cells.join(' | ')} |`;
    }).join('\n') + '\n\n';
  });

  // Strip remaining HTML tags
  text = text.replace(/<[^>]+>/g, '');

  // Decode HTML entities
  text = text.replace(/&amp;/g, '&');
  text = text.replace(/&lt;/g, '<');
  text = text.replace(/&gt;/g, '>');
  text = text.replace(/&quot;/g, '"');
  text = text.replace(/&#39;/g, "'");
  text = text.replace(/&nbsp;/g, ' ');

  // Clean up whitespace
  text = text.replace(/\n{3,}/g, '\n\n');
  text = text.trim();

  md += text;
  return md;
}

async function crawl(): Promise<void> {
  console.log('[crawl] Launching browser');
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();

  await fs.mkdir(OUTPUT_DIR, { recursive: true });
  await fs.mkdir(path.join(OUTPUT_DIR, 'html'), { recursive: true });
  await fs.mkdir(path.join(OUTPUT_DIR, 'markdown'), { recursive: true });

  const visited = new Set<string>();
  const toVisit = new Set<string>(SEED_URLS);
  const allPages: DocPage[] = [];

  while (toVisit.size > 0) {
    const urlPath = toVisit.values().next().value;
    if (!urlPath) break;
    toVisit.delete(urlPath);

    if (visited.has(urlPath)) continue;
    visited.add(urlPath);

    const fullUrl = `${BASE_URL}${urlPath}`;
    console.log(`[crawl] (${visited.size}/${visited.size + toVisit.size}) ${fullUrl}`);

    const page = await context.newPage();
    try {
      await page.goto(fullUrl, { waitUntil: 'networkidle', timeout: 60000 });
      await page.waitForTimeout(2000);

      // Discover new links
      const newLinks = await discoverLinks(page);
      for (const link of newLinks) {
        if (!visited.has(link)) {
          toVisit.add(link);
        }
      }

      // Extract content
      const docPage = await extractContent(page);
      allPages.push(docPage);

      // Save HTML
      const slug = urlPath.replace(/^\/issue-tracker\/?/, '').replace(/\//g, '_') || 'index';
      await fs.writeFile(
        path.join(OUTPUT_DIR, 'html', `${slug}.html`),
        docPage.htmlContent
      );

      // Save markdown
      const markdown = htmlToMarkdown(docPage.htmlContent, docPage.title);
      await fs.writeFile(
        path.join(OUTPUT_DIR, 'markdown', `${slug}.md`),
        markdown
      );

      console.log(`  -> "${docPage.title}" (${docPage.content.length} chars, ${newLinks.length} new links)`);
    } catch (err) {
      console.error(`  -> ERROR: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      await page.close();
    }

    await delay(DELAY_MS);
  }

  // Save index
  const index = allPages.map(p => ({
    url: `${BASE_URL}${p.url}`,
    title: p.title,
    breadcrumbs: p.breadcrumbs,
    contentLength: p.content.length,
  }));

  await fs.writeFile(
    path.join(OUTPUT_DIR, 'index.json'),
    JSON.stringify(index, null, 2)
  );

  // Save combined text content
  const combined = allPages.map(p =>
    `${'='.repeat(80)}\n${p.title}\nURL: ${BASE_URL}${p.url}\n${'='.repeat(80)}\n\n${p.content}`
  ).join('\n\n\n');

  await fs.writeFile(path.join(OUTPUT_DIR, 'all-docs.txt'), combined);

  console.log(`\n[crawl] Done. Crawled ${allPages.length} pages.`);
  console.log(`[crawl] Output in: ${OUTPUT_DIR}`);

  await browser.close();
}

crawl().catch(err => {
  console.error('[crawl] Fatal error:', err);
  process.exit(1);
});
