// Built-in demo console panel with toolbar buttons and event log.
// Renders as a bottom panel in the app layout (toggled via header button or Ctrl+`).

import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import type { CSSProperties } from "react";

// Scenario metadata (duplicated to avoid importing the full demo module at top level)
const SCENARIOS = [
  { key: "quickstart", label: "Quickstart", description: "Seed + browse, triage, comment, search" },
  { key: "triage", label: "Triage", description: "Seed + search bugs, escalate, fix" },
  { key: "lifecycle", label: "Lifecycle", description: "Seed + create issue, full status walk" },
  { key: "comments", label: "Comments", description: "Seed + edit, revisions, hide" },
  { key: "search", label: "Search", description: "Seed + 7 query patterns" },
  { key: "full", label: "Full Demo", description: "Quickstart + lifecycle + comments" },
] as const;

interface LogEntry {
  id: number;
  timestamp: number;
  type: "info" | "step" | "warn" | "error" | "done";
  text: string;
}

// Global log that the demo executor pushes to
const logEntries: LogEntry[] = [];
let logListeners: Array<() => void> = [];
let idCounter = 0;

export function pushLog(type: LogEntry["type"], text: string) {
  logEntries.push({ id: ++idCounter, timestamp: Date.now(), type, text });
  if (logEntries.length > 500) logEntries.splice(0, logEntries.length - 500);
  for (const fn of logListeners) fn();
}

function clearLog() {
  logEntries.length = 0;
  for (const fn of logListeners) fn();
}

function useLog(): LogEntry[] {
  const [snapshot, setSnapshot] = useState<LogEntry[]>([]);
  useEffect(() => {
    const cb = () => setSnapshot([...logEntries]);
    logListeners.push(cb);
    return () => { logListeners = logListeners.filter((l) => l !== cb); };
  }, []);
  return snapshot;
}

export function DemoConsole() {
  const log = useLog();
  const bottomRef = useRef<HTMLDivElement>(null);
  const [demoRunning, setDemoRunning] = useState(false);
  const [filterText, setFilterText] = useState("");

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [log.length]);

  const filteredLog = useMemo(() => {
    if (!filterText) return log;
    const q = filterText.toLowerCase();
    return log.filter((e) => e.text.toLowerCase().includes(q));
  }, [log, filterText]);

  const runScenario = useCallback(async (key: string) => {
    if (demoRunning) return;
    setDemoRunning(true);
    clearLog();
    pushLog("info", `Starting scenario: ${key}`);
    try {
      const mod = await import("../api/demoConsole");
      await mod.runDemo(key, 2000);
      pushLog("done", `Scenario "${key}" finished.`);
    } catch (err) {
      pushLog("error", `Scenario failed: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setDemoRunning(false);
    }
  }, [demoRunning]);

  const stopDemo = useCallback(async () => {
    try {
      const mod = await import("../api/demoConsole");
      mod.stopDemo();
      pushLog("warn", "Demo stop requested.");
    } catch { /* noop */ }
  }, []);

  const pauseDemo = useCallback(async () => {
    try {
      const mod = await import("../api/demoConsole");
      mod.pauseDemo();
      pushLog("info", "Demo paused.");
    } catch { /* noop */ }
  }, []);

  const resumeDemo = useCallback(async () => {
    try {
      const mod = await import("../api/demoConsole");
      mod.resumeDemo();
      pushLog("info", "Demo resumed.");
    } catch { /* noop */ }
  }, []);

  return (
    <div style={styles.wrapper}>
      {/* Toolbar */}
      <div style={styles.toolbar}>
        <span style={styles.toolbarLabel}>DEMO</span>

        {SCENARIOS.map(({ key, label }) => (
          <button
            key={key}
            style={{
              ...styles.btn,
              ...(key === "full" ? styles.btnPrimary : {}),
              ...(demoRunning ? styles.btnDisabled : {}),
            }}
            disabled={demoRunning}
            onClick={() => runScenario(key)}
            title={SCENARIOS.find((s) => s.key === key)?.description}
          >
            {label}
          </button>
        ))}

        <span style={styles.sep} />

        {demoRunning && (
          <>
            <button style={{ ...styles.btn, ...styles.btnWarn }} onClick={pauseDemo}>
              Pause
            </button>
            <button style={{ ...styles.btn, ...styles.btnSuccess }} onClick={resumeDemo}>
              Resume
            </button>
            <button style={{ ...styles.btn, ...styles.btnDanger }} onClick={stopDemo}>
              Stop
            </button>
            <span style={styles.sep} />
          </>
        )}

        {/* Filter + Clear */}
        <input
          style={styles.searchInput}
          placeholder="Filter..."
          value={filterText}
          onChange={(e) => setFilterText(e.target.value)}
        />
        <span style={{ color: "#595959", fontSize: 10, marginLeft: "auto", whiteSpace: "nowrap" }}>
          {filteredLog.length !== log.length
            ? `${filteredLog.length}/${log.length}`
            : `${log.length}`}
        </span>
        <button style={{ ...styles.btn, fontSize: 10, padding: "1px 8px" }} onClick={clearLog}>
          Clear
        </button>
      </div>

      {/* Log */}
      <div style={styles.logContainer}>
        {filteredLog.length === 0 ? (
          <div style={styles.empty}>
            {log.length === 0
              ? "Click a scenario button above to run a demo."
              : "No matching log entries."}
          </div>
        ) : (
          filteredLog.map((e) => (
            <div key={e.id} style={styles.row}>
              <span style={styles.time}>
                {new Date(e.timestamp).toLocaleTimeString()}
              </span>
              <span style={{
                color: e.type === "error" ? "#ff4d4f"
                  : e.type === "warn" ? "#faad14"
                  : e.type === "done" ? "#52c41a"
                  : e.type === "step" ? "#d9d9d9"
                  : "#1677ff",
              }}>
                {e.text}
              </span>
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}

const styles: Record<string, CSSProperties> = {
  wrapper: {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    background: "#141414",
  },
  toolbar: {
    display: "flex",
    alignItems: "center",
    gap: 6,
    padding: "4px 12px",
    borderBottom: "1px solid #303030",
    flexShrink: 0,
    flexWrap: "wrap",
  },
  toolbarLabel: {
    color: "#8c8c8c",
    fontSize: 11,
    fontWeight: 600,
    letterSpacing: 0.5,
  },
  btn: {
    background: "#303030",
    color: "#d9d9d9",
    border: "1px solid #424242",
    borderRadius: 4,
    padding: "2px 10px",
    fontSize: 11,
    cursor: "pointer",
    lineHeight: "20px",
    whiteSpace: "nowrap",
  },
  btnPrimary: {
    background: "#1668dc",
    borderColor: "#1668dc",
    color: "#fff",
  },
  btnSuccess: {
    background: "#389e0d",
    borderColor: "#389e0d",
    color: "#fff",
  },
  btnWarn: {
    background: "#d48806",
    borderColor: "#d48806",
    color: "#fff",
  },
  btnDanger: {
    background: "#a61d24",
    borderColor: "#a61d24",
    color: "#fff",
  },
  btnDisabled: {
    opacity: 0.4,
    cursor: "not-allowed",
  },
  sep: {
    width: 1,
    height: 16,
    background: "#424242",
    margin: "0 4px",
    flexShrink: 0,
  },
  searchInput: {
    background: "#1f1f1f",
    color: "#d9d9d9",
    border: "1px solid #424242",
    borderRadius: 4,
    padding: "2px 6px",
    fontSize: 11,
    lineHeight: "20px",
    outline: "none",
    width: 120,
  },
  logContainer: {
    flex: 1,
    overflowY: "auto",
    padding: "6px 12px",
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    fontSize: 12,
    lineHeight: "22px",
    color: "#d9d9d9",
  },
  empty: {
    color: "#595959",
    fontSize: 12,
    padding: "8px 0",
  },
  row: {
    display: "flex",
    gap: 8,
    alignItems: "baseline",
  },
  time: {
    color: "#595959",
    flexShrink: 0,
    width: 72,
  },
};
