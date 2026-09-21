import type { SyncStatus } from "./types";

export type StartupPresentation = "loading" | "load-error" | "first-run" | "ready";
export type PanePresentation = "unbound" | "missing-list" | "empty" | "tasks";

export function classifyStartupPresentation(input: {
  loading: boolean;
  loadError: string | null;
  loggedIn: boolean;
  taskCount: number;
}): StartupPresentation {
  if (input.loading) return "loading";
  if (input.loadError) return "load-error";
  if (!input.loggedIn && input.taskCount === 0) return "first-run";
  return "ready";
}

export function classifyPanePresentation(input: {
  paneListId?: string | null;
  listExists: boolean;
  taskCount: number;
}): PanePresentation {
  if (!input.paneListId) return "unbound";
  if (!input.listExists) return "missing-list";
  return input.taskCount === 0 ? "empty" : "tasks";
}

export type SyncPresentation = "idle" | "syncing" | "offline" | "error" | "auth-required";

export function classifySyncPresentation(status: SyncStatus): SyncPresentation {
  if (status.state === "syncing") return "syncing";
  if (status.state === "offline") return "offline";
  if (status.state === "error") return "error";
  if (status.state === "auth_required") return "auth-required";
  return "idle";
}
