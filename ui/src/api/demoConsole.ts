// In-browser demo console for Issue Tracker.
// Dev-only -- lazy-loaded via import() and guarded by import.meta.env.DEV.
//
// Usage (browser console):
//   it.demo()                       -- list available scenarios
//   it.demo("quickstart")           -- run a scenario
//   it.demo("quickstart", 2000)     -- custom delay between steps
//   it.demo.run([...steps])         -- run arbitrary steps
//   it.demo.stop()                  -- stop running demo
//   it.demo.pause()                 -- pause demo
//   it.demo.resume()                -- resume paused demo

import { tid } from "../testIds";
import { store } from "../store";
import { login } from "../store/authSlice";
import { api } from "../store/api";
import { pushLog } from "../components/DemoConsole";

// ---------------------------------------------------------------------------
// Step types
// ---------------------------------------------------------------------------

interface GotoStep { goto: string; label?: string }
interface NavStep { nav: string; label?: string }
interface ClickStep { ui_click: string; label?: string }
interface ClickNthStep { ui_click_nth: string; nth: number; label?: string }
interface FillStep { ui_fill: string; value: string; label?: string }
interface ClearFillStep { ui_clear_fill: string; value: string; label?: string }
interface SelectStep { ui_select: string; value: string; search?: boolean; label?: string }
interface ModalOkStep { ui_click_modal_ok: true; label?: string }
interface PopconfirmOkStep { ui_popconfirm_ok: true; label?: string }
interface TableClickStep { ui_table_click: string; label?: string }
interface WaitStep { wait: number; label?: string }
interface PressEnterStep { ui_press_enter: true; label?: string }
interface PressEscapeStep { ui_press_escape: true; label?: string }
interface LoginPersistStep { login_persist: string; label?: string }
/** Seed data via API -- skips UI, directly calls backend */
interface SeedStep { seed: true; label?: string }
/** Wait until a DOM condition is met (verification gate) */
interface WaitForStep { wait_for: string; text?: string; timeout?: number; label?: string }

export type ScenarioStep =
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
  | WaitForStep
  | LoginPersistStep
  | SeedStep;

interface ScenarioDef {
  name: string;
  description: string;
  steps: ScenarioStep[];
}

// ---------------------------------------------------------------------------
// Seed data -- creates demo data via direct API calls
// ---------------------------------------------------------------------------

interface SeedState {
  seeded: boolean;
  components: Map<string, number>; // name -> componentId
  issues: Map<string, number>; // title prefix -> issueId
}

const seedState: SeedState = { seeded: false, components: new Map(), issues: new Map() };

async function apiPost<T>(url: string, body: Record<string, unknown>, userId: string): Promise<T> {
  const resp = await fetch(`/api${url}`, {
    method: "POST",
    headers: { "Content-Type": "application/json", "x-user-id": userId },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`POST ${url} failed: ${resp.status}`);
  return resp.json() as Promise<T>;
}

async function apiPatch<T>(url: string, body: Record<string, unknown>, userId: string): Promise<T> {
  const resp = await fetch(`/api${url}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json", "x-user-id": userId },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`PATCH ${url} failed: ${resp.status}`);
  return resp.json() as Promise<T>;
}

async function apiGet<T>(url: string): Promise<T> {
  const resp = await fetch(`/api${url}`);
  if (!resp.ok) throw new Error(`GET ${url} failed: ${resp.status}`);
  return resp.json() as Promise<T>;
}

interface ApiComponent { componentId: number; name: string }
interface ApiIssue { issueId: number; title: string }

async function seedDemoData(): Promise<void> {
  // Check if already seeded
  const existing = await apiGet<{ components: ApiComponent[] }>("/components");
  if (existing.components.length > 0) {
    pushLog("info", `Seed: found ${existing.components.length} existing components, skipping seed.`);
    // Populate seedState from existing data
    for (const c of existing.components) {
      seedState.components.set(c.name, c.componentId);
    }
    const existingIssues = await apiGet<{ issues: ApiIssue[] }>("/search?q=");
    for (const iss of existingIssues.issues) {
      seedState.issues.set(iss.title.slice(0, 20), iss.issueId);
    }
    seedState.seeded = true;
    return;
  }

  const user = "alice@payments.dev";

  // Components
  pushLog("info", "Seed: creating components...");
  const comps = [
    { name: "Payments", description: "Payment processing and billing services" },
    { name: "Auth", description: "Authentication and authorization services" },
    { name: "Infrastructure", description: "Cloud infrastructure and deployment pipelines" },
    { name: "Frontend", description: "Web frontend application" },
    { name: "Backend", description: "Server-side API services" },
    { name: "Mobile", description: "iOS and Android apps" },
  ];
  for (const c of comps) {
    const res = await apiPost<ApiComponent>("/components", c, user);
    seedState.components.set(c.name, res.componentId);
  }

  // Issues -- diverse types, priorities, statuses, assignees
  pushLog("info", "Seed: creating issues...");
  const issues: Array<{
    componentName: string;
    title: string;
    description: string;
    type?: string;
    priority: string;
    severity?: string;
    assignee?: string;
    reporter?: string;
  }> = [
    {
      componentName: "Payments",
      title: "Payment fails for amounts > $10,000",
      description: "Transactions above $10,000 return HTTP 422. Integer overflow in amount field.",
      priority: "P0",
      assignee: "alice@payments.dev",
      reporter: "frank@support.dev",
    },
    {
      componentName: "Auth",
      title: "Add OAuth2 support for Google and GitHub",
      description: "Enterprise customers need SSO via OAuth2 providers.",
      type: "FEATURE_REQUEST",
      priority: "P1",
      assignee: "bob@auth.dev",
      reporter: "alice@payments.dev",
    },
    {
      componentName: "Infrastructure",
      title: "Database connection pool exhaustion under load",
      description: "Production database connections spike to max (200) during peak traffic. Queries queue and timeout after 30s.",
      priority: "P0",
      severity: "S0",
      assignee: "dave@infra.dev",
      reporter: "dave@infra.dev",
    },
    {
      componentName: "Frontend",
      title: "Login button unresponsive on mobile Safari",
      description: "Users on iOS 17+ cannot tap the login button. Touch events seem to be intercepted by CSS :hover.",
      priority: "P1",
      assignee: "carol@frontend.dev",
      reporter: "frank@support.dev",
    },
    {
      componentName: "Backend",
      title: "API returns 500 on large payloads",
      description: "POST /api/issues with body > 1MB returns HTTP 500. Missing request size limit config.",
      priority: "P0",
      assignee: "alice@payments.dev",
      reporter: "carol@frontend.dev",
    },
    {
      componentName: "Mobile",
      title: "Add dark mode support for iOS",
      description: "iOS app should follow system appearance settings. Currently hardcoded to light theme.",
      type: "FEATURE_REQUEST",
      priority: "P1",
      assignee: "eve@mobile.dev",
      reporter: "eve@mobile.dev",
    },
    {
      componentName: "Backend",
      title: "Upgrade database driver to v5",
      description: "Current driver v3 is EOL. v5 adds connection pooling improvements and query plan caching.",
      type: "TASK",
      priority: "P2",
      assignee: "dave@infra.dev",
    },
    {
      componentName: "Backend",
      title: "SQL injection in search endpoint",
      description: "The /search endpoint passes user input directly to SQL query without parameterization.",
      type: "VULNERABILITY",
      priority: "P0",
      assignee: "bob@auth.dev",
      reporter: "bob@auth.dev",
    },
    {
      componentName: "Mobile",
      title: "Customer reports slow app launch on Android 13",
      description: "Cold start takes 8+ seconds on Pixel 7. Traced to synchronous config loading on main thread.",
      type: "CUSTOMER_ISSUE",
      priority: "P3",
      assignee: "eve@mobile.dev",
      reporter: "frank@support.dev",
    },
    {
      componentName: "Payments",
      title: "Refund processing stuck in PENDING state",
      description: "Refunds initiated through the dashboard remain in PENDING for 24+ hours. Payment gateway webhook not reaching our endpoint.",
      priority: "P1",
      assignee: "alice@payments.dev",
      reporter: "frank@support.dev",
    },
    {
      componentName: "Auth",
      title: "Session tokens not invalidated on password change",
      description: "After password change, old session tokens remain valid. User must manually log out from all devices.",
      type: "VULNERABILITY",
      priority: "P1",
      assignee: "bob@auth.dev",
      reporter: "bob@auth.dev",
    },
    {
      componentName: "Frontend",
      title: "Dashboard chart does not update on filter change",
      description: "Selecting a different date range in the dashboard does not re-render the chart. Requires page refresh.",
      priority: "P2",
      assignee: "carol@frontend.dev",
      reporter: "carol@frontend.dev",
    },
  ];

  for (const iss of issues) {
    const componentId = seedState.components.get(iss.componentName);
    if (!componentId) continue;
    const body: Record<string, unknown> = {
      componentId,
      title: iss.title,
      description: iss.description,
      priority: iss.priority,
    };
    if (iss.type) body.type = iss.type;
    if (iss.severity) body.severity = iss.severity;
    if (iss.assignee) body.assignee = iss.assignee;
    if (iss.reporter) body.reporter = iss.reporter;
    const res = await apiPost<ApiIssue>("/issues", body, iss.reporter ?? user);
    seedState.issues.set(iss.title.slice(0, 20), res.issueId);
  }

  // Add some status transitions and comments to make it realistic
  pushLog("info", "Seed: adding comments and status updates...");

  // Payment bug: assign, add comments
  const paymentId = seedState.issues.get("Payment fails for a");
  if (paymentId) {
    await apiPatch(`/issues/${paymentId}`, { status: "ASSIGNED" }, "alice@payments.dev");
    await apiPost(`/issues/${paymentId}/comments`, {
      body: "Reproduced locally. Integer overflow when amount exceeds INT32_MAX cents.",
      author: "alice@payments.dev",
    }, "alice@payments.dev");
    await apiPatch(`/issues/${paymentId}`, { status: "IN_PROGRESS" }, "alice@payments.dev");
    await apiPost(`/issues/${paymentId}/comments`, {
      body: "Fix in PR #142: switched amount field to int64. All payment tests passing.",
      author: "alice@payments.dev",
    }, "alice@payments.dev");
  }

  // DB connection pool: escalate
  const dbPoolId = seedState.issues.get("Database connection p");
  if (dbPoolId) {
    await apiPatch(`/issues/${dbPoolId}`, { status: "ASSIGNED" }, "dave@infra.dev");
    await apiPost(`/issues/${dbPoolId}/comments`, {
      body: "Investigating. Connection leak suspected in batch processor cron job.",
      author: "dave@infra.dev",
    }, "dave@infra.dev");
  }

  // SQL injection: assign
  const sqliId = seedState.issues.get("SQL injection in sea");
  if (sqliId) {
    await apiPatch(`/issues/${sqliId}`, { status: "ASSIGNED" }, "bob@auth.dev");
    await apiPost(`/issues/${sqliId}/comments`, {
      body: "CRITICAL: confirmed SQLi via search endpoint. Deploying WAF rule as immediate mitigation.",
      author: "bob@auth.dev",
    }, "bob@auth.dev");
    await apiPatch(`/issues/${sqliId}`, { status: "IN_PROGRESS" }, "bob@auth.dev");
  }

  // Create hotlist
  pushLog("info", "Seed: creating hotlists...");
  await apiPost("/hotlists", {
    name: "Q1 2026 Release",
    description: "Critical issues targeted for Q1 release",
    owner: "alice@payments.dev",
  }, "alice@payments.dev");

  await apiPost("/hotlists", {
    name: "Security Audit Findings",
    description: "Issues discovered during Q1 security audit",
    owner: "bob@auth.dev",
  }, "bob@auth.dev");

  // Invalidate RTK Query caches so UI shows fresh data
  store.dispatch(api.util.invalidateTags(["Component", "Issue", "Comment", "Hotlist", "Event"]));

  seedState.seeded = true;
  pushLog("done", `Seed complete: ${comps.length} components, ${issues.length} issues, 2 hotlists.`);
}

// ---------------------------------------------------------------------------
// UI interaction helpers
// ---------------------------------------------------------------------------

const delay = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

function findByTestId(testId: string): HTMLElement | null {
  return document.querySelector(`[data-testid="${testId}"]`);
}

/** Simulate typing into an input: uses React's internal _valueTracker for change detection */
async function simulateTyping(el: HTMLInputElement | HTMLTextAreaElement, text: string) {
  el.focus();
  const proto = el instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  const nativeSetter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
  if (!nativeSetter) {
    el.value = text;
    el.dispatchEvent(new Event("input", { bubbles: true }));
    return;
  }

  for (let i = 1; i <= text.length; i++) {
    const val = text.slice(0, i);
    nativeSetter.call(el, val);
    const tracker = (el as unknown as { _valueTracker?: { setValue(v: string): void } })._valueTracker;
    if (tracker) tracker.setValue(text.slice(0, i - 1));
    el.dispatchEvent(new Event("input", { bubbles: true }));
    await delay(25 + Math.random() * 35);
  }
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

/** Clear an input then type new text */
async function simulateClearAndType(el: HTMLInputElement | HTMLTextAreaElement, text: string) {
  el.focus();
  const proto = el instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  const nativeSetter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
  if (!nativeSetter) {
    el.value = text;
    el.dispatchEvent(new Event("input", { bubbles: true }));
    return;
  }

  // Clear
  nativeSetter.call(el, "");
  const tracker = (el as unknown as { _valueTracker?: { setValue(v: string): void } })._valueTracker;
  if (tracker) tracker.setValue(el.value);
  el.dispatchEvent(new Event("input", { bubbles: true }));
  await delay(50);

  // Type
  await simulateTyping(el, text);
}

function findInputInElement(testId: string): HTMLInputElement | HTMLTextAreaElement | null {
  const wrapper = findByTestId(testId);
  if (!wrapper) return null;
  if (wrapper.tagName === "INPUT" || wrapper.tagName === "TEXTAREA") {
    return wrapper as HTMLInputElement | HTMLTextAreaElement;
  }
  return wrapper.querySelector("input, textarea");
}

/** Click modal OK button */
function clickModalOk(): boolean {
  const modals = document.querySelectorAll<HTMLElement>(".ant-modal-wrap:not([style*='display: none'])");
  for (const modal of Array.from(modals).reverse()) {
    const okBtn = modal.querySelector<HTMLButtonElement>(".ant-modal-footer .ant-btn-primary");
    if (okBtn && !okBtn.disabled) {
      okBtn.click();
      return true;
    }
  }
  return false;
}

/** Click popconfirm OK button */
function clickPopconfirmOk(): boolean {
  const popovers = document.querySelectorAll<HTMLElement>(".ant-popover:not(.ant-popover-hidden)");
  for (const popover of Array.from(popovers).reverse()) {
    const okBtn = popover.querySelector<HTMLButtonElement>(".ant-btn-primary");
    if (okBtn) {
      okBtn.click();
      return true;
    }
  }
  return false;
}

/** Open an Ant Design Select and pick an option */
async function selectOption(testId: string, value: string): Promise<boolean> {
  const wrapper = findByTestId(testId);
  if (!wrapper) return false;

  const selectEl = wrapper.closest(".ant-select") ?? wrapper;
  const clickTarget = selectEl.querySelector<HTMLElement>(".ant-select-selector")
    ?? selectEl.querySelector<HTMLElement>(".ant-select-selection-search input")
    ?? selectEl as HTMLElement;

  clickTarget.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  await delay(400);

  const dropdowns = document.querySelectorAll<HTMLElement>(".ant-select-dropdown:not(.ant-select-dropdown-hidden)");
  const dropdown = dropdowns[dropdowns.length - 1];
  if (!dropdown) {
    console.warn(`[Demo] No dropdown for testid="${testId}"`);
    return false;
  }

  const options = dropdown.querySelectorAll<HTMLElement>(".ant-select-item-option");
  const lower = value.toLowerCase();

  // Exact match
  for (const opt of options) {
    const title = (opt.getAttribute("title") ?? "").toLowerCase();
    const text = (opt.textContent ?? "").trim().toLowerCase();
    if (title === lower || text === lower || text === value) {
      opt.click();
      await delay(200);
      return true;
    }
  }

  // Partial match
  for (const opt of options) {
    const text = (opt.textContent ?? "").trim().toLowerCase();
    if (text.includes(lower)) {
      opt.click();
      await delay(200);
      return true;
    }
  }

  console.warn(`[Demo] Option "${value}" not found in testid="${testId}"`);
  document.body.click();
  return false;
}

/** Click a sidebar nav menu item by text */
function clickNavItem(text: string): boolean {
  const menuItems = document.querySelectorAll<HTMLElement>(".ant-menu-item");
  for (const item of menuItems) {
    const itemText = item.textContent?.trim() ?? "";
    if (itemText.toLowerCase() === text.toLowerCase()) {
      item.click();
      return true;
    }
  }
  return false;
}

/** Click a table cell link by text */
function clickTableLink(text: string): boolean {
  const links = document.querySelectorAll<HTMLElement>(".ant-table-cell a");
  const lower = text.toLowerCase();
  for (const link of links) {
    if ((link.textContent ?? "").toLowerCase().includes(lower)) {
      link.click();
      return true;
    }
  }
  return false;
}

// ---------------------------------------------------------------------------
// Verification: wait for DOM conditions before proceeding
// ---------------------------------------------------------------------------

/** Wait until an element with data-testid appears, or until text appears on screen */
async function waitForCondition(
  testId: string,
  text: string | undefined,
  timeoutMs: number,
): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const el = findByTestId(testId);
    if (el) {
      if (!text) return true;
      if ((el.textContent ?? "").toLowerCase().includes(text.toLowerCase())) return true;
    }
    // Also check if text exists anywhere on page
    if (text && document.body.textContent?.toLowerCase().includes(text.toLowerCase())) {
      return true;
    }
    await delay(200);
  }
  return false;
}

/** After a modal OK click, wait for the modal to close */
async function waitForModalClose(timeoutMs = 5000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const visibleModals = document.querySelectorAll(".ant-modal-wrap:not([style*='display: none'])");
    // Check if any modal has visible body content (not closing animation)
    let hasOpenModal = false;
    for (const m of visibleModals) {
      const body = m.querySelector(".ant-modal-body");
      if (body && body.children.length > 0) {
        hasOpenModal = true;
        break;
      }
    }
    if (!hasOpenModal) return true;
    await delay(200);
  }
  return false;
}

/** After navigation, wait until the page has rendered content */
async function waitForPageReady(timeoutMs = 5000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    // Check that we're not on a loading spinner
    const spinner = document.querySelector(".ant-spin-spinning");
    if (!spinner) return true;
    await delay(200);
  }
  return false;
}

/** After a table click, wait until the URL changes (we navigated to detail page) */
async function waitForUrlChange(prevPath: string, timeoutMs = 5000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (window.location.pathname !== prevPath) return true;
    await delay(200);
  }
  return false;
}

// ---------------------------------------------------------------------------
// Narration overlay
// ---------------------------------------------------------------------------

function ensureNarrationOverlay(): HTMLElement {
  let el = document.getElementById("demo-narration");
  if (el) return el;
  el = document.createElement("div");
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
  return el;
}

function showNarration(text: string) {
  const el = ensureNarrationOverlay();
  el.textContent = text;
  el.style.opacity = "1";
}

function hideNarration() {
  const el = document.getElementById("demo-narration");
  if (el) el.style.opacity = "0";
}

// ---------------------------------------------------------------------------
// Scenarios -- now use { seed: true } to pre-populate data via API
// ---------------------------------------------------------------------------

const scenarios: Map<string, ScenarioDef> = new Map();

function defineScenario(name: string, description: string, steps: ScenarioStep[]): void {
  scenarios.set(name, { name, description, steps });
}

// -- Quickstart: seeded data, then UI walkthrough ----------------------------

defineScenario("quickstart", "Seed data, then walkthrough: browse, triage, comment, search", [
  { seed: true, label: "Seeding demo data (6 components, 12 issues, 2 hotlists)..." },

  // Login as Alice (Tech Lead)
  { goto: "/", label: "Welcome to Issue Tracker. Let's sign in as Alice (Tech Lead)." },
  { wait: 300 },
  { login_persist: "alice@payments.dev" },
  { wait: 500 },

  // Dashboard overview
  { wait_for: tid.dashboard.statComponents, timeout: 5000, label: "Dashboard loaded" },
  { wait: 1500, label: "Dashboard -- overview of components, open issues, and P0 counts." },

  // Browse components
  { nav: "Components", label: "Browse team components." },
  { wait_for: tid.components.table, timeout: 5000 },
  { wait: 1500 },

  // Browse issues
  { nav: "Issues", label: "View all issues across the system." },
  { wait_for: tid.issues.table, timeout: 5000 },
  { wait: 1500 },

  // Open a P0 bug
  { ui_table_click: "Payment fails", label: "Open the P0 payment bug to triage." },
  { wait_for: tid.issueDetail.selectStatus, timeout: 5000 },
  { wait: 800 },

  // Change status
  { ui_select: tid.issueDetail.selectStatus, value: "FIXED", label: "Mark as FIXED -- patch deployed." },
  { wait: 800 },

  // Add comment
  { ui_fill: tid.issueDetail.commentInput, value: "Fix verified in staging. Deploying to production.", label: "Add a verification comment." },
  { wait: 300 },
  { ui_click: tid.issueDetail.commentSend, label: "Send comment" },
  { wait: 1000 },

  // Search
  { nav: "Search", label: "Search for all open P0 bugs." },
  { wait_for: tid.search.input, timeout: 5000 },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "priority:P0", label: "Type search query: priority:P0" },
  { wait: 300 },
  { ui_press_enter: true, label: "Submit search" },
  { wait: 1500 },

  // Events
  { nav: "Events", label: "Event log -- full audit trail of every action." },
  { wait_for: tid.events.table, timeout: 5000 },
  { wait: 2000 },

  // Hotlists
  { nav: "Hotlists", label: "Hotlists -- track critical issue collections." },
  { wait_for: tid.hotlists.table, timeout: 5000 },
  { wait: 1500 },

  // Dashboard
  { nav: "Dashboard", label: "Back to dashboard -- stats updated." },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },
  { wait: 2000 },

  { wait: 2000, label: "Quickstart complete. Issue Tracker is ready for your team." },
]);

// -- Triage: work through bugs with seeded data -----------------------------

defineScenario("triage", "Triage workflow: search bugs, escalate, comment, resolve", [
  { seed: true, label: "Seeding demo data..." },

  // Login as Dave (SRE / On-Call)
  { goto: "/", label: "Triage workflow -- signing in as Dave (SRE / On-Call)." },
  { wait: 300 },
  { login_persist: "dave@infra.dev" },
  { wait: 500 },

  // Search for bugs
  { nav: "Search", label: "Search for all open P0 bugs." },
  { wait_for: tid.search.input, timeout: 5000 },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "priority:P0 status:open", label: "Filter: P0 + open" },
  { ui_press_enter: true },
  { wait: 1500 },

  // Open the DB connection pool bug
  { ui_table_click: "Database connection", label: "Open the database connection pool issue." },
  { wait_for: tid.issueDetail.selectStatus, timeout: 5000 },
  { wait: 1000 },

  // Move to IN_PROGRESS
  { ui_select: tid.issueDetail.selectStatus, value: "IN_PROGRESS", label: "Status -> IN_PROGRESS (actively investigating)." },
  { wait: 800 },

  // Add triage note
  { ui_fill: tid.issueDetail.commentInput, value: "Root cause identified: batch processor not releasing connections. Deploying hotfix.", label: "Add investigation findings." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 1000 },

  // Fix it
  { ui_select: tid.issueDetail.selectStatus, value: "FIXED", label: "Status -> FIXED (hotfix deployed)." },
  { wait: 800 },

  { ui_fill: tid.issueDetail.commentInput, value: "Fix deployed. Pool stable at ~40 connections during peak.", label: "Add resolution note." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 1000 },

  // Check events
  { nav: "Events", label: "Event log shows the full triage trail." },
  { wait_for: tid.events.table, timeout: 5000 },
  { wait: 2000 },

  // Dashboard
  { nav: "Dashboard", label: "Dashboard -- P0 count updated." },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },
  { wait: 2000 },

  { wait: 2000, label: "Triage complete: investigated, fixed, documented." },
]);

// -- Lifecycle: full status machine ------------------------------------------

defineScenario("lifecycle", "Full lifecycle: create issue, walk through all statuses", [
  { seed: true, label: "Seeding demo data..." },

  // Login as Carol (Frontend Engineer)
  { goto: "/", label: "Issue lifecycle demo -- sign in as Carol (Frontend Engineer)." },
  { wait: 300 },
  { login_persist: "carol@frontend.dev" },
  { wait: 500 },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },

  // Create a new issue via UI
  { nav: "Issues", label: "Create a new bug report." },
  { wait_for: tid.issues.table, timeout: 5000 },
  { wait: 500 },
  { ui_click: tid.issues.createBtn },
  { wait: 800 },
  { ui_select: tid.issues.selectComponent, value: "Frontend", label: "Component: Frontend" },
  { ui_fill: tid.issues.inputTitle, value: "Modal flickers on resize in Chrome 120" },
  { ui_fill: tid.issues.inputDescription, value: "When resizing the browser window, the create-issue modal flickers rapidly. Only in Chrome 120+." },
  { ui_select: tid.issues.selectPriority, value: "P2" },
  { ui_fill: tid.issues.inputAssignee, value: "carol@frontend.dev" },
  { wait: 300 },
  { ui_click_modal_ok: true, label: "Submit issue" },
  { wait: 1500 },

  // Open the new issue
  { ui_table_click: "Modal flickers", label: "Open the newly created issue." },
  { wait_for: tid.issueDetail.selectStatus, timeout: 5000 },
  { wait: 800 },

  // Walk through status transitions
  { ui_select: tid.issueDetail.selectStatus, value: "ASSIGNED", label: "Status: NEW -> ASSIGNED" },
  { wait: 800 },

  { ui_fill: tid.issueDetail.commentInput, value: "Investigating. Suspect requestAnimationFrame loop conflict with Ant Design modal." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 1000 },

  { ui_select: tid.issueDetail.selectStatus, value: "IN_PROGRESS", label: "Status: ASSIGNED -> IN_PROGRESS" },
  { wait: 800 },

  { ui_fill: tid.issueDetail.commentInput, value: "Found it. ResizeObserver callback triggers re-render on every pixel change. Fix: debounce to 16ms." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 1000 },

  { ui_select: tid.issueDetail.selectStatus, value: "FIXED", label: "Status: IN_PROGRESS -> FIXED" },
  { wait: 800 },

  { ui_fill: tid.issueDetail.commentInput, value: "Fix in PR #501. Debounced ResizeObserver. Smooth on Chrome 120/121/122." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 1000 },

  { ui_select: tid.issueDetail.selectStatus, value: "FIXED_VERIFIED", label: "Status: FIXED -> FIXED_VERIFIED" },
  { wait: 800 },

  // Events
  { nav: "Events", label: "Event log shows every status transition." },
  { wait_for: tid.events.table, timeout: 5000 },
  { wait: 2000 },

  { nav: "Dashboard", label: "Dashboard -- issue resolved." },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },
  { wait: 2000 },

  { wait: 2000, label: "Lifecycle: NEW -> ASSIGNED -> IN_PROGRESS -> FIXED -> FIXED_VERIFIED" },
]);

// -- Comments: edit, revisions, hide -----------------------------------------

defineScenario("comments", "Comment workflow: add, edit, view revisions, hide", [
  { seed: true, label: "Seeding demo data..." },

  // Login as Bob (Security Engineer)
  { goto: "/", label: "Comment workflow -- sign in as Bob (Security Engineer)." },
  { wait: 300 },
  { login_persist: "bob@auth.dev" },
  { wait: 500 },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },

  // Navigate to an issue with existing comments
  { nav: "Issues", label: "Find the SQL injection issue." },
  { wait_for: tid.issues.table, timeout: 5000 },
  { wait: 500 },
  { ui_table_click: "SQL injection", label: "Open SQL injection issue." },
  { wait_for: tid.issueDetail.selectStatus, timeout: 5000 },
  { wait: 1000 },

  // Add a comment
  { ui_fill: tid.issueDetail.commentInput, value: "Deployed parameterized queries. All injection vectors patched.", label: "Add a fix comment." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 2000 },

  // Edit the comment
  { ui_click: tid.issueDetail.commentEditBtn, label: "Edit comment -- adding more detail." },
  { wait: 500 },
  { ui_clear_fill: tid.issueDetail.commentEditTextarea, value: "Deployed parameterized queries across all endpoints. Also added input validation layer. Pen test confirms no remaining vectors.", label: "Update with pen test results." },
  { wait: 500 },
  { ui_click: tid.issueDetail.commentEditSave, label: "Save edit" },
  { wait: 2000 },

  // View revision history
  { ui_click: tid.issueDetail.commentHistoryBtn, label: "View revision history -- see original text." },
  { wait: 2500 },
  { ui_press_escape: true, label: "Close revision modal" },
  { wait: 500 },

  // Add a temporary comment
  { ui_fill: tid.issueDetail.commentInput, value: "TODO: remove this note after security review sign-off.", label: "Add a temporary note." },
  { ui_click: tid.issueDetail.commentSend },
  { wait: 2000 },

  // Hide the temporary comment
  { ui_click_nth: tid.issueDetail.commentHideBtn, nth: 2, label: "Remove the temporary comment (moderator action)." },
  { wait: 500 },
  { ui_popconfirm_ok: true, label: "Confirm removal" },
  { wait: 1000 },

  // Mark as fixed
  { ui_select: tid.issueDetail.selectStatus, value: "FIXED", label: "Mark as FIXED." },
  { wait: 800 },

  // Events
  { nav: "Events", label: "Event log: create, edit, hide -- all recorded." },
  { wait_for: tid.events.table, timeout: 5000 },
  { wait: 2000 },

  { wait: 2000, label: "Comment workflow complete: add, edit, revisions, hide." },
]);

// -- Search: diverse queries on seeded data ---------------------------------

defineScenario("search", "Search demo: 7 query patterns on seeded data", [
  { seed: true, label: "Seeding demo data..." },

  // Login as Frank (Customer Support)
  { goto: "/", label: "Search demo -- sign in as Frank (Customer Support)." },
  { wait: 300 },
  { login_persist: "frank@support.dev" },
  { wait: 500 },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },

  { nav: "Search", label: "Search 1: Find all P0 issues." },
  { wait_for: tid.search.input, timeout: 5000 },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "priority:P0" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 2: Open bugs only." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "status:open type:BUG" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 3: Feature requests." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "type:FEATURE_REQUEST" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 4: Security vulnerabilities." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "type:VULNERABILITY" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 5: Keyword search -- 'database'." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "database" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 6: Issues assigned to Alice." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "assignee:alice@payments.dev" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Search", label: "Search 7: Customer-reported issues." },
  { wait: 500 },
  { ui_fill: tid.search.input, value: "type:CUSTOMER_ISSUE" },
  { ui_press_enter: true },
  { wait: 2000 },

  { nav: "Dashboard", label: "Dashboard -- all issue data at a glance." },
  { wait_for: tid.dashboard.statComponents, timeout: 5000 },
  { wait: 2000 },

  { wait: 2000, label: "Search demo complete: 7 query patterns demonstrated." },
]);

// Full pipeline
defineScenario("full", "Full demo: quickstart + lifecycle + comments", [
  ...scenarios.get("quickstart")!.steps,
  { wait: 1000, label: "--- Starting Issue Lifecycle demo ---" },
  ...scenarios.get("lifecycle")!.steps.filter((s) => !("seed" in s)),  // skip duplicate seed
  { wait: 1000, label: "--- Starting Comment Workflow demo ---" },
  ...scenarios.get("comments")!.steps.filter((s) => !("seed" in s)),  // skip duplicate seed
]);

// ---------------------------------------------------------------------------
// Step executor with verification
// ---------------------------------------------------------------------------

let running = false;
let stopRequested = false;
let paused = false;
let pauseResolve: (() => void) | null = null;

function logStep(msg: string) {
  console.log(`%c${msg}`, "color: #52c41a; font-weight: bold");
  pushLog("step", msg);
}

function logWarn(msg: string) {
  console.warn(`[Demo] ${msg}`);
  pushLog("warn", msg);
}

async function retryFind<T>(finder: () => T | null, maxAttempts = 15, intervalMs = 300): Promise<T | null> {
  for (let i = 0; i < maxAttempts; i++) {
    const result = finder();
    if (result) return result;
    await delay(intervalMs);
  }
  return null;
}

/** Dismiss any open overlays (dropdowns, modals) to unblock next step */
async function dismissOverlays(): Promise<void> {
  // Close open select dropdowns
  const openDropdowns = document.querySelectorAll(".ant-select-dropdown:not(.ant-select-dropdown-hidden)");
  if (openDropdowns.length > 0) {
    document.body.click();
    await delay(200);
  }
  // Close popconfirm
  const popconfirms = document.querySelectorAll(".ant-popover:not(.ant-popover-hidden)");
  if (popconfirms.length > 0) {
    document.body.click();
    await delay(200);
  }
}

async function executeSteps(steps: ScenarioStep[], title: string, intervalMs: number) {
  const total = steps.length;

  console.log(
    `%c[Demo] ${title} (${total} steps, ${intervalMs}ms interval)`,
    "color: #1890ff; font-weight: bold; font-size: 14px",
  );
  pushLog("info", `Starting: ${title} (${total} steps)`);

  ensureNarrationOverlay();

  for (let i = 0; i < total; i++) {
    if (stopRequested) {
      logStep(`[${i + 1}/${total}] Demo stopped by user`);
      break;
    }
    if (paused) {
      logStep(`[${i + 1}/${total}] Paused...`);
      await new Promise<void>((resolve) => { pauseResolve = resolve; });
    }
    if (stopRequested) break;

    const step = steps[i]!;
    const prefix = `[${i + 1}/${total}]`;

    // Show narration for steps with meaningful labels
    if (step.label && step.label.length > 15) {
      showNarration(step.label);
    }

    try {
      if ("seed" in step) {
        logStep(`${prefix} ${step.label ?? "Seeding demo data..."}`);
        await seedDemoData();

      } else if ("wait_for" in step) {
        logStep(`${prefix} ${step.label ?? `Waiting for [${step.wait_for}]`}`);
        const ok = await waitForCondition(step.wait_for, step.text, step.timeout ?? 5000);
        if (!ok) logWarn(`Timeout waiting for [${step.wait_for}]${step.text ? ` containing "${step.text}"` : ""}`);

      } else if ("goto" in step) {
        logStep(`${prefix} ${step.label ?? `Navigate to ${step.goto}`}`);
        window.history.pushState({}, "", step.goto);
        window.dispatchEvent(new PopStateEvent("popstate"));
        await delay(300);
        await waitForPageReady();

      } else if ("nav" in step) {
        logStep(`${prefix} ${step.label ?? `Sidebar: ${step.nav}`}`);
        await dismissOverlays();
        let ok = false;
        for (let attempt = 0; attempt < 5; attempt++) {
          if (clickNavItem(step.nav)) { ok = true; break; }
          await delay(300);
        }
        if (!ok) logWarn(`Nav item "${step.nav}" not found`);
        await delay(300);
        await waitForPageReady();

      } else if ("ui_click_nth" in step) {
        logStep(`${prefix} ${step.label ?? `Click [${step.ui_click_nth}] #${step.nth}`}`);
        const els = await retryFind(() => {
          const all = document.querySelectorAll<HTMLElement>(`[data-testid="${step.ui_click_nth}"]`);
          return all.length > step.nth ? all : null;
        });
        if (els) {
          els[step.nth]!.scrollIntoView({ behavior: "smooth", block: "center" });
          await delay(200);
          els[step.nth]!.click();
        } else {
          logWarn(`Element #${step.nth} not found: ${step.ui_click_nth}`);
        }
        await delay(200);

      } else if ("ui_click" in step) {
        logStep(`${prefix} ${step.label ?? `Click [${step.ui_click}]`}`);
        const el = await retryFind(() => findByTestId(step.ui_click));
        if (el) {
          el.scrollIntoView({ behavior: "smooth", block: "center" });
          await delay(200);
          el.click();
        } else {
          logWarn(`Element not found: ${step.ui_click}`);
        }
        await delay(200);

      } else if ("ui_clear_fill" in step) {
        logStep(`${prefix} ${step.label ?? `Clear+Type "${step.value}" into [${step.ui_clear_fill}]`}`);
        const el = await retryFind(() => findInputInElement(step.ui_clear_fill));
        if (el) {
          await simulateClearAndType(el, step.value);
        } else {
          logWarn(`Input not found: ${step.ui_clear_fill}`);
        }

      } else if ("ui_fill" in step) {
        logStep(`${prefix} ${step.label ?? `Type "${step.value}" into [${step.ui_fill}]`}`);
        const el = await retryFind(() => findInputInElement(step.ui_fill));
        if (el) {
          await simulateTyping(el, step.value);
        } else {
          logWarn(`Input not found: ${step.ui_fill}`);
        }

      } else if ("ui_select" in step) {
        logStep(`${prefix} ${step.label ?? `Select "${step.value}" in [${step.ui_select}]`}`);
        let ok = false;
        for (let attempt = 0; attempt < 8; attempt++) {
          if (await selectOption(step.ui_select, step.value)) { ok = true; break; }
          await delay(400);
        }
        if (!ok) logWarn(`Select failed: "${step.value}" in [${step.ui_select}]`);

      } else if ("ui_click_modal_ok" in step) {
        logStep(`${prefix} ${step.label ?? "Click modal OK"}`);
        let clicked = false;
        for (let attempt = 0; attempt < 10; attempt++) {
          if (clickModalOk()) { clicked = true; break; }
          await delay(300);
        }
        if (!clicked) {
          logWarn("No modal OK button found");
        } else {
          await waitForModalClose();
        }
        await delay(300);

      } else if ("ui_popconfirm_ok" in step) {
        logStep(`${prefix} ${step.label ?? "Confirm popover"}`);
        let clicked = false;
        for (let attempt = 0; attempt < 10; attempt++) {
          if (clickPopconfirmOk()) { clicked = true; break; }
          await delay(300);
        }
        if (!clicked) logWarn("No popconfirm OK button found");
        await delay(500);

      } else if ("ui_table_click" in step) {
        logStep(`${prefix} ${step.label ?? `Click table row: ${step.ui_table_click}`}`);
        const prevPath = window.location.pathname;
        let found = false;
        for (let attempt = 0; attempt < 15; attempt++) {
          if (clickTableLink(step.ui_table_click)) { found = true; break; }
          await delay(400);
        }
        if (!found) {
          logWarn(`Table link "${step.ui_table_click}" not found`);
        } else {
          await waitForUrlChange(prevPath);
          await waitForPageReady();
        }
        await delay(300);

      } else if ("ui_press_enter" in step) {
        logStep(`${prefix} ${step.label ?? "Press Enter"}`);
        const focused = document.activeElement as HTMLElement | null;
        if (focused) {
          focused.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", code: "Enter", bubbles: true }));
          focused.dispatchEvent(new KeyboardEvent("keypress", { key: "Enter", code: "Enter", bubbles: true }));
          focused.dispatchEvent(new KeyboardEvent("keyup", { key: "Enter", code: "Enter", bubbles: true }));
        }
        await delay(500);

      } else if ("ui_press_escape" in step) {
        logStep(`${prefix} ${step.label ?? "Press Escape"}`);
        document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true }));
        await delay(300);

      } else if ("login_persist" in step) {
        logStep(`${prefix} ${step.label ?? `Login as: ${step.login_persist}`}`);
        localStorage.setItem("it_user_id", step.login_persist);
        store.dispatch(login(step.login_persist));
        await delay(300);
        await waitForPageReady();

      } else if ("wait" in step) {
        if (step.label) logStep(`${prefix} ${step.label}`);
        await delay(step.wait);
        continue;
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      logWarn(`Step ${i + 1} error: ${msg}`);
      // Try to recover by dismissing overlays
      await dismissOverlays();
    }

    if (i < total - 1) await delay(intervalMs);
  }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export function stopDemo() {
  if (running) {
    stopRequested = true;
    if (paused) resumeDemo();
    console.log("%c[Demo] Stop requested", "color: #faad14; font-weight: bold");
    pushLog("warn", "Demo stop requested.");
  }
}

export function pauseDemo() {
  if (running && !paused) {
    paused = true;
    console.log("%c[Demo] Paused -- call it.demo.resume() to continue", "color: #faad14; font-weight: bold");
    pushLog("info", "Demo paused.");
  }
}

export function resumeDemo() {
  if (paused && pauseResolve) {
    paused = false;
    pauseResolve();
    pauseResolve = null;
    console.log("%c[Demo] Resumed", "color: #52c41a; font-weight: bold");
    pushLog("info", "Demo resumed.");
  }
}

export async function runDemo(target?: string, intervalMs = 2000): Promise<void> {
  if (!target) {
    const hdr = (t: string) => console.log(`%c${t}`, "color: #1890ff; font-weight: bold; font-size: 13px");
    const row = (n: string, d: string, color = "#52c41a") =>
      console.log(`  %c${n.padEnd(14)}%c ${d}`, `color: ${color}; font-weight: bold`, "color: inherit");

    hdr("[Demo] Available scenarios:");
    console.log("");
    for (const [name, def] of scenarios) {
      row(name, def.description);
    }
    console.log("");
    hdr("[Demo] Usage:");
    console.log("  it.demo('quickstart')          -- run a scenario");
    console.log("  it.demo('quickstart', 3000)     -- custom delay (ms)");
    console.log("  it.demo.stop()                  -- stop running demo");
    console.log("  it.demo.pause()                 -- pause demo");
    console.log("  it.demo.resume()                -- resume paused demo");
    console.log("  it.demo.run([...steps])          -- run custom steps");
    return;
  }

  if (running) {
    console.warn("[Demo] Already running. Call it.demo.stop() first.");
    return;
  }

  const scenario = scenarios.get(target);
  if (!scenario) {
    console.error(`[Demo] Unknown scenario: "${target}". Run it.demo() to see options.`);
    return;
  }

  running = true;
  stopRequested = false;
  paused = false;

  try {
    await executeSteps(scenario.steps, scenario.name + " -- " + scenario.description, intervalMs);
  } finally {
    running = false;
    hideNarration();
    console.log("%c[Demo] Complete", "color: #52c41a; font-weight: bold; font-size: 14px");
    pushLog("done", "Demo complete.");
  }
}

export async function runSteps(steps: ScenarioStep[], intervalMs = 2000): Promise<void> {
  if (running) {
    console.warn("[Demo] Already running.");
    return;
  }
  running = true;
  stopRequested = false;
  paused = false;
  try {
    await executeSteps(steps, "Custom Steps", intervalMs);
  } finally {
    running = false;
    hideNarration();
  }
}
