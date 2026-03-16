import { store } from "../store";
import { api } from "../store/api";
import type {
  CreateComponentRequest,
  CreateIssueRequest,
  UpdateIssueRequest,
  CreateCommentRequest,
  UpdateCommentRequest,
  CreateHotlistRequest,
} from "./types";

/**
 * Console-bound API for controlling the issue tracker from DevTools.
 *
 * Usage in browser console:
 *   it.fetch.components()
 *   it.fetch.issue(1)
 *   it.fetch.search("status:open priority:P0")
 *   it.mutate.createIssue({ componentId: 1, title: "Bug", priority: "P0", type: "BUG" })
 *   it.mutate.updateIssue(1, { status: "IN_PROGRESS" })
 *   it.mutate.addComment(1, { body: "Fixed it", author: "alice@example.com" })
 */

function logResult(data: unknown): void {
  if (console.table && typeof data === "object" && data !== null) {
    console.table(data);
  } else {
    console.log(data);
  }
}

const consoleApi = {
  fetch: {
    components: async () => {
      const r = await store.dispatch(api.endpoints.listComponents.initiate());
      logResult(r.data);
      return r.data;
    },
    component: async (id: number) => {
      const r = await store.dispatch(api.endpoints.getComponent.initiate(id));
      logResult(r.data);
      return r.data;
    },
    issues: async (componentId?: number) => {
      const r = await store.dispatch(
        api.endpoints.listIssues.initiate(componentId ? { componentId } : undefined),
      );
      logResult(r.data);
      return r.data;
    },
    issue: async (id: number) => {
      const r = await store.dispatch(api.endpoints.getIssue.initiate(id));
      logResult(r.data);
      return r.data;
    },
    comments: async (issueId: number) => {
      const r = await store.dispatch(api.endpoints.listComments.initiate(issueId));
      logResult(r.data);
      return r.data;
    },
    commentRevisions: async (commentId: number) => {
      const r = await store.dispatch(api.endpoints.listCommentRevisions.initiate(commentId));
      logResult(r.data);
      return r.data;
    },
    hotlists: async () => {
      const r = await store.dispatch(api.endpoints.listHotlists.initiate());
      logResult(r.data);
      return r.data;
    },
    hotlist: async (id: number) => {
      const r = await store.dispatch(api.endpoints.getHotlist.initiate(id));
      logResult(r.data);
      return r.data;
    },
    hotlistIssues: async (hotlistId: number) => {
      const r = await store.dispatch(api.endpoints.listHotlistIssues.initiate(hotlistId));
      logResult(r.data);
      return r.data;
    },
    search: async (query: string) => {
      const r = await store.dispatch(api.endpoints.searchIssues.initiate(query));
      logResult(r.data);
      return r.data;
    },
    events: async (entityType?: string, entityId?: number) => {
      const r = await store.dispatch(
        api.endpoints.listEvents.initiate(entityType ? { entityType, entityId } : undefined),
      );
      logResult(r.data);
      return r.data;
    },
  },
  mutate: {
    createComponent: async (data: CreateComponentRequest) => {
      const r = await store.dispatch(api.endpoints.createComponent.initiate(data));
      console.log("Result:", r.data);
      return r.data;
    },
    updateComponent: async (id: number, data: { name?: string; description?: string }) => {
      const r = await store.dispatch(api.endpoints.updateComponent.initiate({ id, ...data }));
      console.log("Result:", r.data);
      return r.data;
    },
    deleteComponent: async (id: number) => {
      const r = await store.dispatch(api.endpoints.deleteComponent.initiate(id));
      console.log("Result:", r.data);
      return r.data;
    },
    createIssue: async (data: CreateIssueRequest) => {
      const r = await store.dispatch(api.endpoints.createIssue.initiate(data));
      console.log("Result:", r.data);
      return r.data;
    },
    updateIssue: async (id: number, data: UpdateIssueRequest) => {
      const r = await store.dispatch(api.endpoints.updateIssue.initiate({ id, ...data }));
      console.log("Result:", r.data);
      return r.data;
    },
    addComment: async (issueId: number, data: CreateCommentRequest) => {
      const r = await store.dispatch(api.endpoints.createComment.initiate({ issueId, ...data }));
      console.log("Result:", r.data);
      return r.data;
    },
    updateComment: async (data: UpdateCommentRequest) => {
      const r = await store.dispatch(api.endpoints.updateComment.initiate(data));
      console.log("Result:", r.data);
      return r.data;
    },
    hideComment: async (commentId: number) => {
      const r = await store.dispatch(api.endpoints.hideComment.initiate({ commentId }));
      console.log("Result:", r.data);
      return r.data;
    },
    createHotlist: async (data: CreateHotlistRequest) => {
      const r = await store.dispatch(api.endpoints.createHotlist.initiate(data));
      console.log("Result:", r.data);
      return r.data;
    },
  },
  store,
  // demo is populated lazily below
  demo: (() => {}) as unknown,
};

/** Create the demo function with attached methods (stop/pause/resume/run) */
async function initDemo(): Promise<void> {
  const { runDemo, runSteps, stopDemo, pauseDemo, resumeDemo } = await import("./demoConsole");

  // Build a callable function with attached control methods
  const demoFn = (target?: string, intervalMs?: number) => runDemo(target, intervalMs);
  demoFn.run = (steps: unknown[], intervalMs?: number) =>
    runSteps(steps as import("./demoConsole").ScenarioStep[], intervalMs);
  demoFn.stop = stopDemo;
  demoFn.pause = pauseDemo;
  demoFn.resume = resumeDemo;

  consoleApi.demo = demoFn;
}

export function bindConsole(): void {
  (window as unknown as Record<string, unknown>)["it"] = consoleApi;

  // Lazy-load demo console
  initDemo().catch((err) => console.warn("[Demo] Failed to load demo console:", err));

  console.log(
    "%c Issue Tracker Console Ready ",
    "background: #1890ff; color: white; font-size: 14px; padding: 4px 8px; border-radius: 4px;",
  );
  console.log("Available commands:");
  console.log("  it.fetch.components()           - List all components");
  console.log("  it.fetch.issues()               - List all issues");
  console.log("  it.fetch.issue(id)              - Get issue by ID");
  console.log("  it.fetch.search('status:open')  - Search issues");
  console.log("  it.fetch.comments(issueId)      - List comments");
  console.log("  it.fetch.events()               - List events");
  console.log("  it.mutate.createComponent({name, description})");
  console.log("  it.mutate.createIssue({componentId, title, type, priority})");
  console.log("  it.mutate.updateIssue(id, {status, assignee, ...})");
  console.log("  it.mutate.addComment(issueId, {body, author})");
  console.log("");
  console.log(
    "%c Demo Console ",
    "background: #52c41a; color: white; font-size: 14px; padding: 4px 8px; border-radius: 4px;",
  );
  console.log("  it.demo()                       - List demo scenarios");
  console.log("  it.demo('quickstart')           - Run a scenario");
  console.log("  it.demo.stop()                  - Stop running demo");
  console.log("  it.demo.pause() / .resume()     - Pause/resume demo");
}
