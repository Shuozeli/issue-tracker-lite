import { test, expect, type Page } from "@playwright/test";

// These tests run sequentially against a live server.
// Prerequisites: gRPC server on :50051, API proxy on :3001, Vite on :5173

// Use unique prefix to avoid collisions with existing data
const RUN_ID = `e2e-${Date.now()}`;
const COMP_A = `${RUN_ID}-Payments`;
const COMP_B = `${RUN_ID}-Auth`;
const ISSUE_BUG = `${RUN_ID} Payment fails for large amounts`;
const ISSUE_FEAT = `${RUN_ID} Add OAuth2 support`;
const ISSUE_TASK = `${RUN_ID} Migrate DB column to BIGINT`;
const HOTLIST_NAME = `${RUN_ID}-Sprint1`;

const TEST_USER = "e2e-tester@example.com";

// Track created issue IDs for later reference
let bugIssueId: number;

async function waitForApi(page: Page): Promise<void> {
  await page.waitForLoadState("networkidle");
}

async function loginAs(page: Page, email: string): Promise<void> {
  await page.addInitScript((user) => {
    localStorage.setItem("it_user_id", user);
  }, email);
}

async function selectComponent(page: Page, modal: ReturnType<Page["locator"]>, compName: string): Promise<void> {
  const compSelect = modal.locator(".ant-select").first();

  // Retry: open dropdown, wait for options, type to filter, click target
  await expect(async () => {
    // Close any open dropdown first
    await page.keyboard.press("Escape");
    await page.waitForTimeout(100);

    await compSelect.click();
    // Wait for at least one option to be rendered in the dropdown
    const firstOption = page.locator(".ant-select-item-option").first();
    await expect(firstOption).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 20000, intervals: [500, 1000, 2000, 3000] });

  // Type to filter
  await compSelect.locator("input").pressSequentially(compName.slice(-12), { delay: 20 });
  await page.locator(".ant-select-item-option").filter({ hasText: compName }).first().click({ timeout: 10000 });
}

test.describe("Issue Tracker E2E", () => {
  test.describe.configure({ mode: "serial" });

  test.beforeEach(async ({ page }) => {
    await loginAs(page, TEST_USER);
  });

  // --- Login ---

  test("login page shows when not authenticated", async ({ browser }) => {
    // Use a fresh context without the beforeEach loginAs init script
    const context = await browser.newContext();
    const page = await context.newPage();
    await page.goto("/");
    await expect(page.getByText("Sign in with your email to continue")).toBeVisible();
    await expect(page.getByPlaceholder("you@example.com")).toBeVisible();

    // Log in
    await page.getByPlaceholder("you@example.com").fill(TEST_USER);
    await page.getByRole("button", { name: "Sign In" }).click();

    // Should see dashboard (stat card)
    await expect(page.locator(".ant-statistic-title").filter({ hasText: "Components" })).toBeVisible();
    await context.close();
  });

  // --- Dashboard ---

  test("dashboard loads with stats", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".ant-statistic-title").filter({ hasText: "Components" })).toBeVisible();
    await expect(page.locator(".ant-statistic-title").filter({ hasText: "Open Issues" })).toBeVisible();
    await expect(page.locator(".ant-statistic-title").filter({ hasText: "Closed" })).toBeVisible();
    await expect(page.locator(".ant-statistic-title").filter({ hasText: "P0 Open" })).toBeVisible();
  });

  // --- Components ---

  test("navigate to components page", async ({ page }) => {
    await page.goto("/components");
    await expect(page.locator("text=Components").first()).toBeVisible();
    await expect(page.getByRole("button", { name: "New Component" })).toBeVisible();
  });

  test("create component A", async ({ page }) => {
    await page.goto("/components");
    await page.getByRole("button", { name: "New Component" }).click();

    const modal = page.locator(".ant-modal");
    await expect(modal).toBeVisible();

    await modal.getByLabel("Name").fill(COMP_A);
    await modal.getByLabel("Description").fill("Payment processing service");
    await modal.getByRole("button", { name: "OK" }).click();

    await expect(modal).not.toBeVisible();
    await expect(page.getByText(COMP_A).first()).toBeVisible();
  });

  test("create component B", async ({ page }) => {
    await page.goto("/components");
    await page.getByRole("button", { name: "New Component" }).click();

    const modal = page.locator(".ant-modal");
    await modal.getByLabel("Name").fill(COMP_B);
    await modal.getByLabel("Description").fill("Authentication service");
    await modal.getByRole("button", { name: "OK" }).click();

    await expect(modal).not.toBeVisible();
    await expect(page.getByText(COMP_B).first()).toBeVisible();
  });

  // --- Issues ---

  test("navigate to issues page", async ({ page }) => {
    await page.goto("/issues");
    await expect(page.getByRole("button", { name: "New Issue" })).toBeVisible();
  });

  test("create a P0 BUG issue", async ({ page }) => {
    await page.goto("/issues");
    await waitForApi(page);
    await page.getByRole("button", { name: "New Issue" }).click();

    const modal = page.locator(".ant-modal");
    await expect(modal).toBeVisible();

    // Select component
    await selectComponent(page, modal, COMP_A);

    await modal.getByLabel("Title").fill(ISSUE_BUG);
    await modal.getByLabel("Description").fill("Transactions above $10,000 return a 422 error");

    // Change priority to P0
    const prioritySelect = modal.locator(".ant-form-item").filter({ hasText: "Priority" }).locator(".ant-select");
    await prioritySelect.click();
    await page.locator(".ant-select-item-option").filter({ hasText: /^P0$/ }).click();

    await modal.getByLabel("Assignee").fill(TEST_USER);
    // Reporter defaults to the logged-in user (e2e-tester@example.com)

    const responsePromise = page.waitForResponse((r) => r.url().includes("/api/issues") && r.request().method() === "POST");
    await modal.getByRole("button", { name: "OK" }).click();
    const response = await responsePromise;
    const body = await response.json() as { issueId: number };
    bugIssueId = body.issueId;

    await expect(modal).not.toBeVisible();
    // Wait for the list to refetch and the issue to appear in the table
    await waitForApi(page);
    await expect(page.locator(`.ant-table-cell a`).filter({ hasText: ISSUE_BUG }).first()).toBeAttached({ timeout: 10000 });
  });

  test("create a P1 FEATURE_REQUEST issue", async ({ page }) => {
    await page.goto("/issues");
    await waitForApi(page);
    await page.getByRole("button", { name: "New Issue" }).click();

    const modal = page.locator(".ant-modal");

    // Select component
    await selectComponent(page, modal, COMP_B);

    await modal.getByLabel("Title").fill(ISSUE_FEAT);
    await modal.getByLabel("Description").fill("Support Google and GitHub OAuth2 providers");

    // Change type to FEATURE_REQUEST
    const typeSelect = modal.locator(".ant-form-item").filter({ hasText: "Type" }).locator(".ant-select");
    await typeSelect.click();
    await page.locator(".ant-select-item-option").filter({ hasText: /^FEATURE_REQUEST$/ }).click();

    // Change priority to P1
    const prioritySelect = modal.locator(".ant-form-item").filter({ hasText: "Priority" }).locator(".ant-select");
    await prioritySelect.click();
    await page.locator(".ant-select-item-option").filter({ hasText: /^P1$/ }).click();

    await modal.getByLabel("Assignee").fill(TEST_USER);

    await modal.getByRole("button", { name: "OK" }).click();
    await expect(modal).not.toBeVisible();
    await waitForApi(page);
    await expect(page.locator(`.ant-table-cell a`).filter({ hasText: ISSUE_FEAT }).first()).toBeAttached({ timeout: 10000 });
  });

  test("create a P2 TASK issue", async ({ page }) => {
    await page.goto("/issues");
    await waitForApi(page);

    await page.getByRole("button", { name: "New Issue" }).click();

    const modal = page.locator(".ant-modal");

    await selectComponent(page, modal, COMP_A);

    await modal.getByLabel("Title").fill(ISSUE_TASK);

    const typeSelect = modal.locator(".ant-form-item").filter({ hasText: "Type" }).locator(".ant-select");
    await typeSelect.click();
    await page.locator(".ant-select-item-option").filter({ hasText: /^TASK$/ }).click();

    await modal.getByRole("button", { name: "OK" }).click();
    await expect(modal).not.toBeVisible();
    await waitForApi(page);
    await expect(page.locator(`.ant-table-cell a`).filter({ hasText: ISSUE_TASK }).first()).toBeAttached({ timeout: 10000 });
  });

  // --- Issue Detail ---

  test("view issue detail page", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    await expect(page.getByText(ISSUE_BUG).first()).toBeVisible();
    await expect(page.getByText("Transactions above").first()).toBeVisible();
    await expect(page.getByText(TEST_USER).first()).toBeVisible();
  });

  test("update issue status to IN_PROGRESS", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    const statusSelect = page.locator("tr").filter({ hasText: "Status" }).locator(".ant-select").first();
    await statusSelect.click();
    const patchResponse = page.waitForResponse((resp) => resp.url().includes("/api/issues/") && resp.request().method() === "PATCH");
    await page.locator(".ant-select-item-option").filter({ hasText: /^IN_PROGRESS$/ }).click();
    await patchResponse;

    await page.reload();
    await waitForApi(page);
    await expect(page.getByText("IN_PROGRESS").first()).toBeVisible();
  });

  test("update issue priority", async ({ page }) => {
    // Navigate to the FEAT issue by searching for it
    await page.goto("/issues");
    await waitForApi(page);
    // Find the FEAT issue row and click it
    await page.locator(".ant-table-cell a").filter({ hasText: ISSUE_FEAT }).first().click();
    await waitForApi(page);

    const prioritySelect = page.locator("tr").filter({ hasText: "Priority" }).locator(".ant-select").nth(1);
    await prioritySelect.click();
    const priPatch = page.waitForResponse((resp) => resp.url().includes("/api/issues/") && resp.request().method() === "PATCH");
    await page.locator(".ant-select-item-option").filter({ hasText: /^P0$/ }).click();
    await priPatch;

    await page.reload();
    await waitForApi(page);
    await expect(page.locator("tr").filter({ hasText: "Priority" }).locator(".ant-select").nth(1).getByText("P0")).toBeVisible();
  });

  // --- Comments ---

  test("add a comment to an issue", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    await page.getByPlaceholder("Write a comment...").fill("Root cause found: integer overflow in amount field.");

    const commentResponse = page.waitForResponse((resp) => resp.url().includes("/comments") && resp.request().method() === "POST");
    await page.getByRole("button", { name: "Send" }).click();
    await commentResponse;
    await waitForApi(page);

    await expect(page.getByText("Root cause found")).toBeVisible();
  });

  test("add a second comment", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    await expect(page.getByText("Root cause found")).toBeVisible();

    await page.getByPlaceholder("Write a comment...").fill("Fix deployed to staging. Needs QA verification.");

    const comment2Response = page.waitForResponse((resp) => resp.url().includes("/comments") && resp.request().method() === "POST");
    await page.getByRole("button", { name: "Send" }).click();
    await comment2Response;
    await waitForApi(page);

    await expect(page.getByText("Fix deployed to staging")).toBeVisible();
    await expect(page.getByText("Comments (3)")).toBeVisible();
  });

  test("edit own comment", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    // Find the comment by the test user and click edit
    const commentItem = page.locator(".ant-list-item").filter({ hasText: "Root cause found" });
    await commentItem.locator("button").filter({ has: page.locator("[aria-label='edit'],.anticon-edit") }).click();

    // Edit the comment
    const textarea = commentItem.locator("textarea");
    await textarea.fill("Root cause found: integer overflow in amount field. Fix in PR #42.");

    const patchResponse = page.waitForResponse((resp) => resp.url().includes("/comments/") && resp.request().method() === "PATCH");
    await commentItem.getByRole("button", { name: "Save" }).click();
    await patchResponse;
    await waitForApi(page);

    await expect(page.getByText("Fix in PR #42")).toBeVisible();
    await expect(page.getByText("(edited)").first()).toBeVisible();
  });

  test("view revision history", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    // The edited comment should show a history button
    const commentItem = page.locator(".ant-list-item").filter({ hasText: "Fix in PR #42" });
    await commentItem.locator("button").filter({ has: page.locator("[aria-label='history'],.anticon-history") }).click();

    // Revision modal should open showing the old text
    const modal = page.locator(".ant-modal").filter({ hasText: "Revision History" });
    await expect(modal).toBeVisible();
    await expect(modal.getByText("integer overflow in amount field.")).toBeVisible();

    await modal.locator("button.ant-modal-close").click();
  });

  test("hide (remove) a comment", async ({ page }) => {
    await page.goto(`/issues/${bugIssueId}`);
    await waitForApi(page);

    // Find the second comment and hide it
    const commentItem = page.locator(".ant-list-item").filter({ hasText: "Fix deployed to staging" });
    await commentItem.locator("button").filter({ has: page.locator("[aria-label='eye-invisible'],.anticon-eye-invisible") }).click();

    // Confirm the popconfirm
    const hideResponse = page.waitForResponse((resp) => resp.url().includes("/hide") && resp.request().method() === "POST");
    await page.getByRole("button", { name: "Remove" }).click();
    await hideResponse;
    await waitForApi(page);

    // Should show the removed placeholder
    await expect(page.getByText("This comment has been removed by a moderator")).toBeVisible();
  });

  // --- Search ---

  test("navigate to search page", async ({ page }) => {
    await page.goto("/search");
    await expect(page.getByPlaceholder("Search issues")).toBeVisible();
  });

  test("search for open issues", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder("Search issues").fill("status:open");
    await page.getByRole("button", { name: "Search" }).click();

    await page.waitForResponse((resp) => resp.url().includes("/api/search"));

    await expect(page.locator("table")).toBeVisible();
    const rows = page.locator("table tbody tr");
    await expect(rows).not.toHaveCount(0);
  });

  test("search by priority P0", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder("Search issues").fill("priority:P0");
    await page.getByRole("button", { name: "Search" }).click();

    await page.waitForResponse((resp) => resp.url().includes("/api/search"));

    await expect(page.locator("td a").filter({ hasText: ISSUE_BUG }).first()).toBeAttached();
  });

  test("search by type BUG", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder("Search issues").fill("type:BUG");
    await page.getByRole("button", { name: "Search" }).click();

    await page.waitForResponse((resp) => resp.url().includes("/api/search"));

    await expect(page.locator("td a").filter({ hasText: ISSUE_BUG }).first()).toBeAttached();
  });

  test("search by keyword", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder("Search issues").fill(RUN_ID);
    await page.getByRole("button", { name: "Search" }).click();

    await page.waitForResponse((resp) => resp.url().includes("/api/search"));

    // Should find issues created in this run
    const rows = page.locator("table tbody tr");
    await expect(rows).not.toHaveCount(0);
  });

  test("click example query tag", async ({ page }) => {
    await page.goto("/search");
    await page.getByText("status:open", { exact: true }).click();

    await page.waitForResponse((resp) => resp.url().includes("/api/search"));

    await expect(page.locator("table")).toBeVisible();
    const rows = page.locator("table tbody tr");
    await expect(rows).not.toHaveCount(0);
  });

  // --- Hotlists ---

  test("navigate to hotlists page", async ({ page }) => {
    await page.goto("/hotlists");
    await expect(page.getByRole("button", { name: "New Hotlist" })).toBeVisible();
  });

  test("create a hotlist", async ({ page }) => {
    await page.goto("/hotlists");
    await page.getByRole("button", { name: "New Hotlist" }).click();

    const modal = page.locator(".ant-modal");
    await modal.getByLabel("Name").fill(HOTLIST_NAME);
    await modal.getByLabel("Description").fill("Sprint 1 work items");
    await modal.getByLabel("Owner").fill("pm@example.com");
    await modal.getByRole("button", { name: "OK" }).click();

    await expect(modal).not.toBeVisible();
    await expect(page.getByText(HOTLIST_NAME).first()).toBeVisible();
  });

  // --- Events ---

  test("navigate to events page", async ({ page }) => {
    await page.goto("/events");
    await expect(page.getByText("Event Log")).toBeVisible();
    await waitForApi(page);

    const rows = page.locator("table tbody tr");
    await expect(rows).not.toHaveCount(0);
  });

  // --- Console API ---

  test("console: it.fetch.components() works", async ({ page }) => {
    await page.goto("/");
    await waitForApi(page);

    const result = await page.evaluate(async () => {
      const it = (window as Record<string, unknown>)["it"] as {
        fetch: { components: () => Promise<unknown> };
      };
      return it.fetch.components();
    });

    const typed = result as { components: Array<{ name: string }> };
    expect(typed.components).toBeDefined();
    expect(typed.components.length).toBeGreaterThan(0);
  });

  test("console: it.fetch.search() works", async ({ page }) => {
    await page.goto("/");
    await waitForApi(page);

    const result = await page.evaluate(async () => {
      const it = (window as Record<string, unknown>)["it"] as {
        fetch: { search: (q: string) => Promise<unknown> };
      };
      return it.fetch.search("status:open");
    });

    const typed = result as { issues: Array<{ title: string }> };
    expect(typed.issues).toBeDefined();
    expect(typed.issues.length).toBeGreaterThan(0);
  });

  test("console: it.mutate.createIssue() works", async ({ page }) => {
    await page.goto("/");
    await waitForApi(page);

    const title = `${RUN_ID} console-created`;
    const result = await page.evaluate(
      async ({ title: t }) => {
        const it = (window as Record<string, unknown>)["it"] as {
          fetch: { components: () => Promise<{ components: Array<{ componentId: number }> }> };
          mutate: {
            createIssue: (data: {
              componentId: number;
              title: string;
              type: string;
              priority: string;
            }) => Promise<unknown>;
          };
        };
        const comps = await it.fetch.components();
        return it.mutate.createIssue({
          componentId: comps.components[0]!.componentId,
          title: t,
          type: "BUG",
          priority: "P1",
        });
      },
      { title },
    );

    const typed = result as { title: string; issueId: number };
    expect(typed.title).toBe(title);
    expect(typed.issueId).toBeGreaterThan(0);
  });

  test("console: it.mutate.addComment() works", async ({ page }) => {
    await page.goto("/");
    await waitForApi(page);

    const result = await page.evaluate(
      async ({ issueId }) => {
        const it = (window as Record<string, unknown>)["it"] as {
          mutate: {
            addComment: (
              issueId: number,
              data: { body: string; author: string },
            ) => Promise<unknown>;
          };
        };
        return it.mutate.addComment(issueId, {
          body: "Console comment test",
          author: "console@example.com",
        });
      },
      { issueId: bugIssueId },
    );

    const typed = result as { body: string };
    expect(typed.body).toBe("Console comment test");
  });

  // --- Navigation ---

  test("sidebar navigation works", async ({ page }) => {
    await page.goto("/");

    await page.locator(".ant-menu-item").filter({ hasText: "Issues" }).click();
    await expect(page).toHaveURL(/\/issues/);

    await page.locator(".ant-menu-item").filter({ hasText: "Components" }).click();
    await expect(page).toHaveURL(/\/components/);

    await page.locator(".ant-menu-item").filter({ hasText: "Search" }).click();
    await expect(page).toHaveURL(/\/search/);

    await page.locator(".ant-menu-item").filter({ hasText: "Dashboard" }).click();
    await expect(page).toHaveURL(/\/$/);
  });

  // --- Issue detail from search ---

  test("click issue in search results navigates to detail", async ({ page }) => {
    await page.goto("/search");
    await page.getByPlaceholder("Search issues").fill("status:open");
    await page.getByRole("button", { name: "Search" }).click();

    await page.waitForResponse((resp) => resp.url().includes("/api/search"));
    await waitForApi(page);

    const firstIssueLink = page.locator("table tbody tr a").first();
    await expect(firstIssueLink).toBeVisible();
    await firstIssueLink.click();

    await expect(page).toHaveURL(/\/issues\/\d+/);
  });

  // --- Mark issue as FIXED ---

  test("update issue to FIXED status", async ({ page }) => {
    await page.goto("/issues");
    await waitForApi(page);
    await page.locator(".ant-table-cell a").filter({ hasText: ISSUE_TASK }).first().click();
    await waitForApi(page);

    const statusSelect = page.locator("tr").filter({ hasText: "Status" }).locator(".ant-select").first();
    await statusSelect.click();
    const fixPatch = page.waitForResponse((resp) => resp.url().includes("/api/issues/") && resp.request().method() === "PATCH");
    await page.locator(".ant-select-item-option").filter({ hasText: /^FIXED$/ }).click();
    await fixPatch;

    await page.reload();
    await waitForApi(page);
    await expect(page.getByText("FIXED").first()).toBeVisible();
  });

  // --- Dashboard reflects changes ---

  test("dashboard shows updated stats", async ({ page }) => {
    await page.goto("/");
    await waitForApi(page);

    await expect(page.locator("text=Open Issues")).toBeVisible();

    const closedStat = page.locator(".ant-card").filter({ hasText: "Closed" }).locator(".ant-statistic-content-value");
    const closedCount = await closedStat.textContent();
    expect(Number(closedCount)).toBeGreaterThanOrEqual(1);
  });
});
