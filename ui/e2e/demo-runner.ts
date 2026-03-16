/**
 * Demo Runner Framework (Declarative)
 *
 * Playwright-based scripted demo system with declarative, serializable steps.
 * Steps are plain objects (no closures), making them inspectable and loggable.
 *
 * Usage:
 *   pnpm run demo                              # headless
 *   pnpm run demo:headed                       # local browser window
 *   pnpm run demo -- --scenario quickstart
 *   pnpm run demo:remote                       # CDP to remote Chrome
 *   pnpm run demo:record                       # with video recording
 */

import { chromium, type Page, type Browser, type BrowserContext } from "playwright";

// ---------------------------------------------------------------------------
// Test IDs -- mirrored from src/testIds.ts (keep in sync)
// ---------------------------------------------------------------------------

const tid = {
  login: {
    email: "login_email",
    submit: "login_submit",
  },
  nav: {
    dashboard: "nav_dashboard",
    issues: "nav_issues",
    components: "nav_components",
    hotlists: "nav_hotlists",
    search: "nav_search",
    events: "nav_events",
  },
  header: {
    userId: "header_user_id",
    signOut: "header_sign_out",
  },
  components: {
    createBtn: "components_create_btn",
    table: "components_table",
    inputName: "components_create_name",
    inputDescription: "components_create_description",
  },
  issues: {
    createBtn: "issues_create_btn",
    table: "issues_table",
    selectComponent: "issues_create_component",
    inputTitle: "issues_create_title",
    inputDescription: "issues_create_description",
    selectType: "issues_create_type",
    selectPriority: "issues_create_priority",
    selectSeverity: "issues_create_severity",
    inputAssignee: "issues_create_assignee",
  },
  issueDetail: {
    selectStatus: "issue_detail_status",
    selectPriority: "issue_detail_priority",
    commentInput: "issue_detail_comment_input",
    commentSend: "issue_detail_comment_send",
    commentEditBtn: "comment_edit_btn",
    commentEditTextarea: "comment_edit_textarea",
    commentEditSave: "comment_edit_save",
    commentEditCancel: "comment_edit_cancel",
    commentHistoryBtn: "comment_history_btn",
    commentHideBtn: "comment_hide_btn",
  },
  hotlists: {
    createBtn: "hotlists_create_btn",
    table: "hotlists_table",
    inputName: "hotlists_create_name",
    inputDescription: "hotlists_create_description",
    inputOwner: "hotlists_create_owner",
  },
  search: {
    input: "search_input",
    submitBtn: "search_submit",
    resultsTable: "search_results_table",
  },
  events: {
    table: "events_table",
  },
} as const;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

interface DemoConfig {
  baseUrl: string;
  stepPauseMs: number;
  typingDelayMs: number;
  user: string;
  scenario: string;
  headed: boolean;
  cdpEndpoint: string | undefined;
  recordVideo: boolean;
}

function parseConfig(): DemoConfig {
  const args = process.argv.slice(2);
  const getArg = (name: string): string | undefined => {
    const idx = args.indexOf(`--${name}`);
    return idx >= 0 && idx + 1 < args.length ? args[idx + 1] : undefined;
  };
  const hasFlag = (name: string): boolean => args.includes(`--${name}`);

  return {
    baseUrl: getArg("base-url") ?? process.env["BASE_URL"] ?? "http://localhost:5173",
    stepPauseMs: Number(getArg("pause") ?? "1500"),
    typingDelayMs: Number(getArg("typing-delay") ?? "40"),
    user: getArg("user") ?? "demo@example.com",
    scenario: getArg("scenario") ?? "full",
    headed: hasFlag("headed") || !!process.env["HEADED"],
    cdpEndpoint: getArg("cdp") ?? process.env["CDP_ENDPOINT"],
    recordVideo: hasFlag("record") || !!process.env["DEMO_RECORD"],
  };
}

// ---------------------------------------------------------------------------
// Step types -- declarative, serializable
// ---------------------------------------------------------------------------

/** Navigate to a route path */
interface GotoStep { goto: string; label?: string }
/** Click a sidebar menu item by text */
interface NavStep { nav: string; label?: string }
/** Click a button/element by data-testid */
interface ClickStep { ui_click: string; label?: string }
/** Click the Nth element matching a data-testid (0-indexed) */
interface ClickNthStep { ui_click_nth: string; nth: number; label?: string }
/** Type into an input by data-testid (character-by-character) */
interface FillStep { ui_fill: string; value: string; label?: string }
/** Clear an input then type new value by data-testid */
interface ClearFillStep { ui_clear_fill: string; value: string; label?: string }
/** Select an option in an Ant Design Select by data-testid + option text */
interface SelectStep { ui_select: string; value: string; search?: boolean; label?: string }
/** Click the OK button in the currently visible Ant Design modal */
interface ModalOkStep { ui_click_modal_ok: true; label?: string }
/** Confirm an Ant Design Popconfirm dialog (clicks the OK/confirm button) */
interface PopconfirmOkStep { ui_popconfirm_ok: true; label?: string }
/** Click a table row link by text match */
interface TableClickStep { ui_table_click: string; label?: string }
/** Explicit wait (ms) */
interface WaitStep { wait: number; label?: string }
/** Press Enter on the currently focused element */
interface PressEnterStep { ui_press_enter: true; label?: string }
/** Press Escape */
interface PressEscapeStep { ui_press_escape: true; label?: string }
/** Set login persistence (ensures login survives page reloads) */
interface LoginPersistStep { login_persist: string; label?: string }

type ScenarioStep =
  | GotoStep
  | NavStep
  | PressEnterStep
  | PressEscapeStep
  | ClickStep
  | ClickNthStep
  | FillStep
  | ClearFillStep
  | SelectStep
  | ModalOkStep
  | PopconfirmOkStep
  | TableClickStep
  | WaitStep
  | LoginPersistStep;

interface ScenarioDef {
  name: string;
  description: string;
  steps: ScenarioStep[];
}

// ---------------------------------------------------------------------------
// Narration overlay
// ---------------------------------------------------------------------------

async function injectNarrationOverlay(page: Page): Promise<void> {
  await page.evaluate(() => {
    if (document.getElementById("demo-narration")) return;
    const el = document.createElement("div");
    el.id = "demo-narration";
    el.style.cssText = [
      "position: fixed",
      "bottom: 0",
      "left: 0",
      "right: 0",
      "z-index: 99999",
      "background: rgba(0, 0, 0, 0.82)",
      "color: #fff",
      "font-family: 'SF Mono', 'Fira Code', monospace",
      "font-size: 16px",
      "padding: 14px 24px",
      "text-align: center",
      "transition: opacity 0.3s",
      "opacity: 0",
      "pointer-events: none",
    ].join(";");
    document.body.appendChild(el);
  });
}

async function showNarration(page: Page, text: string): Promise<void> {
  await page.evaluate((t) => {
    const el = document.getElementById("demo-narration");
    if (!el) return;
    el.textContent = t;
    el.style.opacity = "1";
  }, text);
}

async function hideNarration(page: Page): Promise<void> {
  await page.evaluate(() => {
    const el = document.getElementById("demo-narration");
    if (el) el.style.opacity = "0";
  });
}

// ---------------------------------------------------------------------------
// Step executor -- interprets declarative steps via Playwright
// ---------------------------------------------------------------------------

function testIdSelector(testId: string): string {
  return `[data-testid="${testId}"]`;
}

async function executeStep(page: Page, step: ScenarioStep, config: DemoConfig): Promise<void> {
  if ("goto" in step) {
    await page.goto(`${config.baseUrl}${step.goto}`);
    await page.waitForLoadState("networkidle");
    await injectNarrationOverlay(page);

  } else if ("nav" in step) {
    // Dismiss any open overlays first
    const openDropdowns = page.locator(".ant-select-dropdown:visible");
    if (await openDropdowns.count() > 0) {
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);
    }
    await page.locator(".ant-menu-item").filter({ hasText: step.nav }).click();
    await page.waitForLoadState("networkidle");
    await injectNarrationOverlay(page);

  } else if ("ui_click_nth" in step) {
    const el = page.locator(testIdSelector(step.ui_click_nth)).nth(step.nth);
    await el.scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await el.click();

  } else if ("ui_click" in step) {
    const el = page.locator(testIdSelector(step.ui_click)).first();
    await el.scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await el.click();

  } else if ("ui_clear_fill" in step) {
    const wrapper = page.locator(testIdSelector(step.ui_clear_fill)).first();
    const input = (await wrapper.evaluate((el) => el.tagName === "INPUT" || el.tagName === "TEXTAREA"))
      ? wrapper
      : wrapper.locator("input, textarea").first();
    await input.click();
    await input.fill("");
    await input.pressSequentially(step.value, { delay: config.typingDelayMs });

  } else if ("ui_fill" in step) {
    const wrapper = page.locator(testIdSelector(step.ui_fill)).first();
    // Ant Design wraps inputs -- find the actual <input> or <textarea> inside
    const input = (await wrapper.evaluate((el) => el.tagName === "INPUT" || el.tagName === "TEXTAREA"))
      ? wrapper
      : wrapper.locator("input, textarea").first();
    await input.click();
    await input.pressSequentially(step.value, { delay: config.typingDelayMs });

  } else if ("ui_select" in step) {
    const wrapper = page.locator(testIdSelector(step.ui_select)).first();
    const selectEl = wrapper.locator(".ant-select-selector").first();
    await selectEl.click({ timeout: 5000 });
    await page.waitForTimeout(500);

    if (step.search) {
      // Type to filter (for searchable selects like component picker)
      const searchInput = wrapper.locator("input").first();
      await searchInput.pressSequentially(step.value, { delay: config.typingDelayMs });
      await page.waitForTimeout(400);
    }

    // Wait for the specific option and click it
    const option = page.locator(".ant-select-item-option").filter({
      hasText: new RegExp(`^${step.value}$`),
    });
    await option.waitFor({ state: "visible", timeout: 5000 });
    await page.waitForTimeout(200);
    await option.click();
    await page.waitForTimeout(200);

  } else if ("ui_click_modal_ok" in step) {
    const modal = page.locator(".ant-modal:visible");
    const okBtn = modal.locator(".ant-modal-footer .ant-btn-primary").first();
    await okBtn.click();
    await page.waitForLoadState("networkidle");

  } else if ("ui_table_click" in step) {
    const link = page.locator(".ant-table-cell a").filter({ hasText: step.ui_table_click }).first();
    await link.click();
    await page.waitForLoadState("networkidle");

  } else if ("wait" in step) {
    await page.waitForTimeout(step.wait);

  } else if ("ui_popconfirm_ok" in step) {
    // Ant Design Popconfirm renders a popover with OK/Cancel buttons
    const popover = page.locator(".ant-popconfirm:visible").first();
    await popover.locator(".ant-btn-primary").click();
    await page.waitForLoadState("networkidle");

  } else if ("ui_press_enter" in step) {
    await page.keyboard.press("Enter");
    await page.waitForLoadState("networkidle");

  } else if ("ui_press_escape" in step) {
    await page.keyboard.press("Escape");
    await page.waitForTimeout(300);

  } else if ("login_persist" in step) {
    await page.context().addInitScript((user) => {
      localStorage.setItem("it_user_id", user);
    }, step.login_persist);
  }
}

function getStepLabel(step: ScenarioStep): string {
  if (step.label) return step.label;
  if ("goto" in step) return `Navigate to ${step.goto}`;
  if ("nav" in step) return `Sidebar: ${step.nav}`;
  if ("ui_click_nth" in step) return `Click [${step.ui_click_nth}] #${step.nth}`;
  if ("ui_click" in step) return `Click [${step.ui_click}]`;
  if ("ui_clear_fill" in step) return `Clear+Type "${step.value}" into [${step.ui_clear_fill}]`;
  if ("ui_fill" in step) return `Type "${step.value}" into [${step.ui_fill}]`;
  if ("ui_select" in step) return `Select "${step.value}" in [${step.ui_select}]`;
  if ("ui_click_modal_ok" in step) return "Click modal OK";
  if ("ui_popconfirm_ok" in step) return "Confirm popover";
  if ("ui_table_click" in step) return `Click table row: ${step.ui_table_click}`;
  if ("ui_press_enter" in step) return "Press Enter";
  if ("ui_press_escape" in step) return "Press Escape";
  if ("wait" in step) return `Wait ${step.wait}ms`;
  if ("login_persist" in step) return `Persist login: ${step.login_persist}`;
  return "Unknown step";
}

// ---------------------------------------------------------------------------
// Scenario definitions
// ---------------------------------------------------------------------------

const scenarios: Map<string, ScenarioDef> = new Map();

function defineScenario(name: string, description: string, steps: ScenarioStep[]): void {
  scenarios.set(name, { name, description, steps });
}

// -- Quickstart: full product demo ------------------------------------------

defineScenario("quickstart", "Create components, file bugs, triage, comment, search", [
  // Step 1: Login
  { goto: "/", label: "Welcome to Issue Tracker. Let's sign in." },
  { wait: 500 },
  { ui_fill: tid.login.email, value: "demo@example.com", label: "Type email address" },
  { wait: 400 },
  { ui_click: tid.login.submit, label: "Click Sign In" },
  { wait: 500 },
  { login_persist: "demo@example.com", label: "Persist login for page reloads" },

  // Step 2: Dashboard overview
  { wait: 1500, label: "Dashboard -- overview of components, open issues, and P0 counts." },

  // Step 3: Create Payments component
  { nav: "Components", label: "Let's create a new component for the Payments team." },
  { wait: 500 },
  { ui_click: tid.components.createBtn, label: "Click New Component" },
  { wait: 500 },
  { ui_fill: tid.components.inputName, value: "Payments", label: "Type component name" },
  { ui_fill: tid.components.inputDescription, value: "Payment processing and billing services", label: "Type description" },
  { wait: 300 },
  { ui_click_modal_ok: true, label: "Submit" },
  { wait: 800 },

  // Step 4: Create Auth component
  { ui_click: tid.components.createBtn, label: "And another component for Auth." },
  { wait: 500 },
  { ui_fill: tid.components.inputName, value: "Auth", label: "Type component name" },
  { ui_fill: tid.components.inputDescription, value: "Authentication and authorization services", label: "Type description" },
  { wait: 300 },
  { ui_click_modal_ok: true, label: "Submit" },
  { wait: 800 },

  // Step 5: File P0 bug against Payments
  { goto: "/issues", label: "Now let's file a P0 bug against Payments." },
  { wait: 800 },
  { ui_click: tid.issues.createBtn, label: "Click New Issue" },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Payments", search: true, label: "Select component: Payments" },
  { ui_fill: tid.issues.inputTitle, value: "Payment fails for amounts > $10,000", label: "Type issue title" },
  { ui_fill: tid.issues.inputDescription, value: "Transactions above $10,000 return HTTP 422.", label: "Type description" },
  { ui_select: tid.issues.selectPriority, value: "P0", label: "Set priority to P0" },
  { ui_fill: tid.issues.inputAssignee, value: "alice@payments.dev", label: "Assign to Alice" },
  { wait: 300 },
  { ui_click_modal_ok: true, label: "Submit issue" },
  { wait: 800 },

  // Step 6: File feature request for Auth
  { ui_click: tid.issues.createBtn, label: "File a feature request for OAuth2 support." },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Auth", search: true, label: "Select component: Auth" },
  { ui_fill: tid.issues.inputTitle, value: "Add OAuth2 support for Google and GitHub", label: "Type issue title" },
  { ui_fill: tid.issues.inputDescription, value: "Enterprise customers need SSO via OAuth2.", label: "Type description" },
  { ui_select: tid.issues.selectType, value: "FEATURE_REQUEST", label: "Set type to FEATURE_REQUEST" },
  { ui_select: tid.issues.selectPriority, value: "P1", label: "Set priority to P1" },
  { ui_fill: tid.issues.inputAssignee, value: "bob@auth.dev", label: "Assign to Bob" },
  { wait: 300 },
  { ui_click_modal_ok: true, label: "Submit issue" },
  { wait: 800 },

  // Step 7: Open the P0 bug
  { ui_table_click: "Payment fails", label: "Let's open the P0 bug and triage it." },
  { wait: 800 },

  // Step 8: Change status to IN_PROGRESS
  { ui_select: tid.issueDetail.selectStatus, value: "ASSIGNED", label: "Assign the issue and move it to IN_PROGRESS." },
  { wait: 500 },
  { ui_select: tid.issueDetail.selectStatus, value: "IN_PROGRESS", label: "Status -> IN_PROGRESS" },
  { wait: 800 },

  // Step 9: Add comment
  { ui_fill: tid.issueDetail.commentInput, value: "Root cause: integer overflow in amount field. Fix in PR #142.", label: "Add a comment with the root cause analysis." },
  { wait: 300 },
  { ui_click: tid.issueDetail.commentSend, label: "Send comment" },
  { wait: 800 },

  // Step 10: Search
  { nav: "Search", label: "Search for all open P0 bugs across the system." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "status:open priority:P0", label: "Type search query" },
  { wait: 300 },
  // Input.Search has a button inside -- click the search icon button
  { ui_press_enter: true, label: "Submit search" },
  { wait: 1500 },

  // Step 11: Events
  { nav: "Events", label: "Check the event log to see the full audit trail." },
  { wait: 2000 },

  // Step 12: Create hotlist
  { nav: "Hotlists", label: "Let's create a hotlist for the Q1 release." },
  { wait: 500 },
  { ui_click: tid.hotlists.createBtn, label: "Click New Hotlist" },
  { wait: 500 },
  { ui_fill: tid.hotlists.inputName, value: "Q1 2026 Release", label: "Type hotlist name" },
  { ui_fill: tid.hotlists.inputDescription, value: "Critical issues targeted for Q1 release", label: "Type description" },
  { ui_fill: tid.hotlists.inputOwner, value: "demo@example.com", label: "Set owner" },
  { wait: 300 },
  { ui_click_modal_ok: true, label: "Submit" },
  { wait: 800 },

  // Step 13: Dashboard
  { nav: "Dashboard", label: "Back to the dashboard -- stats reflect our new data." },
  { wait: 2000 },

  // Step 14: Done
  { wait: 3000, label: "Demo complete. Issue Tracker is ready for your team." },
]);

// -- Triage workflow --------------------------------------------------------

defineScenario("triage", "Triage workflow: search, update status, comment, resolve", [
  // Login
  { goto: "/", label: "Triage workflow -- signing in as a tech lead." },
  { wait: 500 },
  { ui_fill: tid.login.email, value: "demo@example.com", label: "Type email" },
  { wait: 300 },
  { ui_click: tid.login.submit, label: "Sign in" },
  { wait: 500 },
  { login_persist: "demo@example.com" },

  // Search for bugs
  { nav: "Search", label: "Search for all open bugs assigned to the team." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "status:open type:BUG", label: "Type search query" },
  { ui_press_enter: true, label: "Submit search" },
  { wait: 1500 },

  // Open first bug
  { ui_table_click: "Payment fails", label: "Open the first bug to investigate." },
  { wait: 1500 },

  // Escalate priority
  { ui_select: tid.issueDetail.selectPriority, value: "P0", label: "Escalate priority to P0 -- this is customer-impacting." },
  { wait: 800 },

  // Add triage note
  { ui_fill: tid.issueDetail.commentInput, value: "Triaged: escalating to P0. Assigning to on-call.", label: "Add a triage note." },
  { ui_click: tid.issueDetail.commentSend, label: "Send" },
  { wait: 800 },

  // Move to IN_PROGRESS
  { ui_select: tid.issueDetail.selectStatus, value: "IN_PROGRESS", label: "Move to IN_PROGRESS." },
  { wait: 800 },

  // Dashboard
  { nav: "Dashboard", label: "Check the dashboard for updated P0 count." },
  { wait: 2000 },

  // Done
  { wait: 2000, label: "Triage complete." },
]);

// -- Issue Lifecycle: full status machine ------------------------------------

defineScenario("lifecycle", "Full issue lifecycle: create, assign, progress, fix, verify", [
  // Login
  { goto: "/", label: "Issue Lifecycle demo -- sign in." },
  { wait: 500 },
  { ui_fill: tid.login.email, value: "lead@example.com", label: "Sign in as team lead" },
  { wait: 300 },
  { ui_click: tid.login.submit },
  { wait: 500 },
  { login_persist: "lead@example.com" },

  // Create a component first
  { nav: "Components", label: "Create an Infrastructure component." },
  { wait: 500 },
  { ui_click: tid.components.createBtn },
  { wait: 500 },
  { ui_fill: tid.components.inputName, value: "Infrastructure" },
  { ui_fill: tid.components.inputDescription, value: "Cloud infrastructure and deployment pipelines" },
  { wait: 300 },
  { ui_click_modal_ok: true },
  { wait: 800 },

  // Create a critical bug
  { goto: "/issues", label: "File a critical infrastructure bug." },
  { wait: 800 },
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Infrastructure", search: true, label: "Select Infrastructure component" },
  { ui_fill: tid.issues.inputTitle, value: "Database connection pool exhaustion under load" },
  { ui_fill: tid.issues.inputDescription, value: "Production database connections spike to max (200) during peak traffic. Queries queue and timeout after 30s." },
  { ui_select: tid.issues.selectPriority, value: "P0" },
  { ui_select: tid.issues.selectSeverity, value: "S0", label: "Severity S0 -- production impact" },
  { ui_fill: tid.issues.inputAssignee, value: "lead@example.com" },
  { wait: 300 },
  { ui_click_modal_ok: true },
  { wait: 800 },

  // Open the bug and walk through status transitions
  { ui_table_click: "Database connection pool", label: "Open the bug and begin the lifecycle." },
  { wait: 1000 },

  // NEW -> ASSIGNED
  { ui_select: tid.issueDetail.selectStatus, value: "ASSIGNED", label: "Status: NEW -> ASSIGNED (acknowledged by team)" },
  { wait: 800 },

  // Add investigation comment
  { ui_fill: tid.issueDetail.commentInput, value: "Investigating. Suspect connection leak in batch job." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 800 },

  // ASSIGNED -> IN_PROGRESS
  { ui_select: tid.issueDetail.selectStatus, value: "IN_PROGRESS", label: "Status: ASSIGNED -> IN_PROGRESS (actively working)" },
  { wait: 800 },

  // Add progress comment
  { ui_fill: tid.issueDetail.commentInput, value: "Found the leak in batch processor. Fix in PR #287." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 800 },

  // IN_PROGRESS -> FIXED
  { ui_select: tid.issueDetail.selectStatus, value: "FIXED", label: "Status: IN_PROGRESS -> FIXED (patch deployed)" },
  { wait: 800 },

  // Add fix comment
  { ui_fill: tid.issueDetail.commentInput, value: "Fix deployed. Pool stable at ~40 during peak." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 800 },

  // FIXED -> FIXED_VERIFIED
  { ui_select: tid.issueDetail.selectStatus, value: "FIXED_VERIFIED", label: "Status: FIXED -> FIXED_VERIFIED (confirmed stable after 24h)" },
  { wait: 800 },

  // Check event log
  { nav: "Events", label: "Event log shows the full audit trail of every status transition." },
  { wait: 2000 },

  // Dashboard
  { nav: "Dashboard", label: "Dashboard updated -- one closed issue." },
  { wait: 2000 },

  { wait: 2000, label: "Lifecycle complete: NEW -> ASSIGNED -> IN_PROGRESS -> FIXED -> FIXED_VERIFIED" },
]);

// -- Comment Editing: edit, revisions, hide ---------------------------------

defineScenario("comments", "Comment editing: add, edit, view revisions, hide", [
  // Login
  { goto: "/", label: "Comment editing demo -- sign in." },
  { wait: 500 },
  { ui_fill: tid.login.email, value: "demo@example.com", label: "Sign in" },
  { wait: 300 },
  { ui_click: tid.login.submit },
  { wait: 500 },
  { login_persist: "demo@example.com" },

  // Create a component and issue to work with
  { nav: "Components", label: "Set up a test component and issue." },
  { wait: 500 },
  { ui_click: tid.components.createBtn },
  { wait: 500 },
  { ui_fill: tid.components.inputName, value: "Frontend" },
  { ui_fill: tid.components.inputDescription, value: "Web frontend application" },
  { wait: 300 },
  { ui_click_modal_ok: true },
  { wait: 800 },

  { goto: "/issues" },
  { wait: 800 },
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Frontend", search: true },
  { ui_fill: tid.issues.inputTitle, value: "Login button unresponsive on mobile Safari" },
  { ui_fill: tid.issues.inputDescription, value: "Users on iOS 17+ cannot tap the login button. Touch events seem to be intercepted." },
  { ui_select: tid.issues.selectPriority, value: "P1" },
  { wait: 300 },
  { ui_click_modal_ok: true },
  { wait: 800 },

  // Open the issue
  { ui_table_click: "Login button unresponsive", label: "Open the issue to demonstrate comment workflow." },
  { wait: 1000 },

  // Add first comment
  { ui_fill: tid.issueDetail.commentInput, value: "Reproduced on iPhone 15 Pro, iOS 17.2, Safari.", label: "Add a comment with reproduction details." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 2000 },

  // Edit the comment
  { ui_click: tid.issueDetail.commentEditBtn, label: "Edit the comment -- found more details." },
  { wait: 500 },
  { ui_clear_fill: tid.issueDetail.commentEditTextarea, value: "Also affects iPad Air M2. Root cause: CSS :hover blocks touch events.", label: "Update with additional findings" },
  { wait: 500 },
  { ui_click: tid.issueDetail.commentEditSave, label: "Save the edited comment" },
  { wait: 2000 },

  // Now the comment has revisions -- click history button
  { ui_click: tid.issueDetail.commentHistoryBtn, label: "View revision history -- see the original text." },
  { wait: 2000 },
  { ui_press_escape: true, label: "Close revision history" },
  { wait: 500 },

  // Add another comment
  { ui_fill: tid.issueDetail.commentInput, value: "Temporary note -- will remove after fix is confirmed.", label: "Add a temporary comment." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 2000 },

  // Hide/remove the temporary comment
  { ui_click_nth: tid.issueDetail.commentHideBtn, nth: 1, label: "Remove the temporary comment (moderator action)." },
  { wait: 500 },
  { ui_popconfirm_ok: true, label: "Confirm removal" },
  { wait: 1000 },

  // Add final comment
  { ui_fill: tid.issueDetail.commentInput, value: "Fix in PR #412. Removed :hover for touch devices.", label: "Add the resolution comment." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 1000 },

  // Check events
  { nav: "Events", label: "Event log captured every comment action: create, edit, hide." },
  { wait: 2000 },

  { wait: 2000, label: "Comment workflow complete: add, edit, view revisions, hide, resolve." },
]);

// -- Multi-search: diverse queries ------------------------------------------

defineScenario("search", "Advanced search: multiple filters, query syntax demo", [
  // Login
  { goto: "/", label: "Search demo -- sign in and create diverse issues." },
  { wait: 500 },
  { ui_fill: tid.login.email, value: "demo@example.com" },
  { wait: 300 },
  { ui_click: tid.login.submit },
  { wait: 500 },
  { login_persist: "demo@example.com" },

  // Create components
  { nav: "Components" },
  { wait: 500 },
  { ui_click: tid.components.createBtn },
  { wait: 500 },
  { ui_fill: tid.components.inputName, value: "Backend" },
  { ui_fill: tid.components.inputDescription, value: "Server-side API services" },
  { wait: 300 },
  { ui_click_modal_ok: true },
  { wait: 500 },
  { ui_click: tid.components.createBtn },
  { wait: 500 },
  { ui_fill: tid.components.inputName, value: "Mobile" },
  { ui_fill: tid.components.inputDescription, value: "iOS and Android apps" },
  { wait: 300 },
  { ui_click_modal_ok: true },
  { wait: 800 },

  // Create diverse issues
  { goto: "/issues", label: "Filing issues of different types and priorities." },
  { wait: 800 },

  // Issue 1: P0 BUG
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Backend", search: true },
  { ui_fill: tid.issues.inputTitle, value: "API returns 500 on large payloads" },
  { ui_select: tid.issues.selectPriority, value: "P0" },
  { ui_fill: tid.issues.inputAssignee, value: "alice@dev.com" },
  { wait: 200 },
  { ui_click_modal_ok: true },
  { wait: 500 },

  // Issue 2: P1 FEATURE_REQUEST
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Mobile", search: true },
  { ui_fill: tid.issues.inputTitle, value: "Add dark mode support for iOS" },
  { ui_select: tid.issues.selectType, value: "FEATURE_REQUEST" },
  { ui_select: tid.issues.selectPriority, value: "P1" },
  { ui_fill: tid.issues.inputAssignee, value: "bob@dev.com" },
  { wait: 200 },
  { ui_click_modal_ok: true },
  { wait: 500 },

  // Issue 3: P2 TASK
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Backend", search: true },
  { ui_fill: tid.issues.inputTitle, value: "Upgrade database driver to v5" },
  { ui_select: tid.issues.selectType, value: "TASK" },
  { ui_select: tid.issues.selectPriority, value: "P2" },
  { wait: 200 },
  { ui_click_modal_ok: true },
  { wait: 500 },

  // Issue 4: P0 VULNERABILITY
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Backend", search: true },
  { ui_fill: tid.issues.inputTitle, value: "SQL injection in search endpoint" },
  { ui_select: tid.issues.selectType, value: "VULNERABILITY" },
  { ui_select: tid.issues.selectPriority, value: "P0" },
  { ui_fill: tid.issues.inputAssignee, value: "alice@dev.com" },
  { wait: 200 },
  { ui_click_modal_ok: true },
  { wait: 500 },

  // Issue 5: P3 CUSTOMER_ISSUE
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Mobile", search: true },
  { ui_fill: tid.issues.inputTitle, value: "Customer reports slow app launch on Android 13" },
  { ui_select: tid.issues.selectType, value: "CUSTOMER_ISSUE" },
  { ui_select: tid.issues.selectPriority, value: "P3" },
  { wait: 200 },
  { ui_click_modal_ok: true },
  { wait: 800 },

  // Now demonstrate different search queries
  { nav: "Search", label: "Search 1: Find all P0 issues." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "priority:P0" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 2: Find all open bugs." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "status:open type:BUG" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 3: Find all feature requests." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "type:FEATURE_REQUEST" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 4: Find vulnerabilities (critical security issues)." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "type:VULNERABILITY" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 5: Keyword search -- find issues mentioning 'database'." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "database" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 6: Compound query -- P0 bugs assigned to Alice." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "priority:P0 type:BUG assignee:alice@dev.com" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 7: Exclude tasks from results." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "status:open -type:TASK" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Dashboard", label: "Dashboard reflects all 5 new issues." },
  { wait: 2000 },

  { wait: 2000, label: "Search demo complete: 7 different query patterns demonstrated." },
]);

// -- Full pipeline: quickstart + lifecycle + comments -----------------------

defineScenario("full", "Full product demo (quickstart + lifecycle + comments + search)", [
  ...scenarios.get("quickstart")!.steps,
  { wait: 1000, label: "--- Starting Issue Lifecycle demo ---" },
  ...scenarios.get("lifecycle")!.steps,
  { wait: 1000, label: "--- Starting Comment Editing demo ---" },
  ...scenarios.get("comments")!.steps,
]);

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

async function run(): Promise<void> {
  const config = parseConfig();

  const scenario = scenarios.get(config.scenario);
  if (!scenario) {
    console.error(`Unknown scenario: "${config.scenario}"`);
    console.error(`Available: ${[...scenarios.keys()].join(", ")}`);
    process.exit(1);
  }

  console.log(`\n  Demo Runner`);
  console.log(`  Scenario:  ${scenario.name} -- ${scenario.description}`);
  console.log(`  Base URL:  ${config.baseUrl}`);
  console.log(`  User:      ${config.user}`);
  console.log(`  Pause:     ${config.stepPauseMs}ms between steps`);
  if (config.cdpEndpoint) console.log(`  CDP:       ${config.cdpEndpoint}`);
  console.log();

  let browser: Browser;
  let context: BrowserContext;

  if (config.cdpEndpoint) {
    browser = await chromium.connectOverCDP(config.cdpEndpoint);
    const contexts = browser.contexts();
    context = contexts.length > 0 ? contexts[0]! : await browser.newContext();
  } else {
    browser = await chromium.launch({
      headless: !config.headed,
      slowMo: 50,
    });
    context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
      ...(config.recordVideo
        ? { recordVideo: { dir: "demo-recordings/", size: { width: 1440, height: 900 } } }
        : {}),
    });
  }

  const page = context.pages().length > 0 ? context.pages()[0]! : await context.newPage();
  page.setDefaultTimeout(10000);

  // Group consecutive steps into logical narration blocks
  // A step with a label that reads like narration becomes the displayed text
  const steps = scenario.steps;
  let currentNarration = "";

  console.log(`  Running ${steps.length} steps...\n`);

  for (let i = 0; i < steps.length; i++) {
    const step = steps[i]!;
    const label = getStepLabel(step);

    // Update narration if this step has a meaningful label
    if (step.label && step.label.length > 20) {
      currentNarration = step.label;
      console.log(`  [${i + 1}/${steps.length}] ${currentNarration}`);
      await injectNarrationOverlay(page);
      await showNarration(page, currentNarration);
      await page.waitForTimeout(800);
    }

    try {
      await executeStep(page, step, config);
    } catch (err) {
      const msg = err instanceof Error ? err.message.split("\n")[0] : String(err);
      console.error(`  ERROR at step ${i + 1} (${label}): ${msg}`);
      try {
        const screenshotPath = `demo-errors/step-${i + 1}.png`;
        await page.screenshot({ path: screenshotPath });
        console.error(`  Screenshot: ${screenshotPath}`);
      } catch { /* ignore */ }
      // Recover: dismiss overlays
      try {
        const modalWrap = page.locator(".ant-modal-wrap:visible");
        if (await modalWrap.count() > 0) {
          const cancelBtn = page.locator(".ant-modal:visible .ant-btn").filter({ hasText: "Cancel" }).first();
          if (await cancelBtn.isVisible()) {
            await cancelBtn.click();
            await page.waitForTimeout(300);
          }
        }
      } catch { /* ignore */ }
    }
  }

  // Final pause
  await page.waitForTimeout(config.stepPauseMs);
  await hideNarration(page);

  console.log("\n  Demo finished.\n");

  if (config.recordVideo) {
    const videoPath = await page.video()?.path();
    if (videoPath) {
      console.log(`  Video saved: ${videoPath}\n`);
    }
  }

  if (!config.cdpEndpoint) {
    await context.close();
    await browser.close();
  }
}

run().catch((err) => {
  console.error("Demo runner failed:", err);
  process.exit(1);
});
