import type { Timestamp } from "../api/types";

export function priorityColor(p: string): string {
  switch (p) {
    case "P0": return "red";
    case "P1": return "orange";
    case "P2": return "blue";
    case "P3": return "green";
    case "P4": return "default";
    default: return "default";
  }
}

export function statusColor(s: string): string {
  switch (s) {
    case "NEW": return "blue";
    case "ASSIGNED": return "cyan";
    case "IN_PROGRESS": return "processing";
    case "FIXED": return "success";
    case "FIXED_VERIFIED": return "green";
    case "WONT_FIX": return "default";
    case "DUPLICATE": return "warning";
    case "INACTIVE": return "default";
    default: return "default";
  }
}

export function typeColor(t: string): string {
  switch (t) {
    case "BUG": return "red";
    case "FEATURE_REQUEST": return "blue";
    case "VULNERABILITY": return "magenta";
    case "TASK": return "cyan";
    case "INTERNAL_CLEANUP": return "default";
    case "CUSTOMER_ISSUE": return "orange";
    default: return "default";
  }
}

export function formatTimestamp(ts: Timestamp | null): string {
  if (!ts) return "-";
  const date = new Date(ts.seconds * 1000);
  return date.toLocaleString();
}
