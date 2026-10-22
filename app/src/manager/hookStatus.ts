import type { HookStatus } from "./types";

/** One line for the header: what the app is doing with Shift+drag. */
export const STATUS_LABEL: Record<HookStatus, string> = {
  starting: "Starting",
  listening: "Listening for Shift+drag",
  "permission-denied": "Needs permission",
  unsupported: "Not supported here",
  error: "Not running",
};

/** Windows blocks dragging from a window that runs as administrator onto one that does not. */
export const WINDOWS_ADMIN_NOTE =
  "Windows does not allow dragging files out of a window running as administrator. Open Explorer normally to use Shift+drag.";

/** Longer help, shown only when the status needs the user to do something. */
export const STATUS_HELP: Partial<Record<HookStatus, string>> = {
  "permission-denied":
    "Allow this app under System Settings, Privacy & Security, Input Monitoring, then restart it.",
  unsupported: "Shift+drag detection is not available on this system yet.",
  error: "The Shift+drag listener could not start.",
};

export type Tone = "ok" | "neutral" | "warn" | "danger";

/** The color family of each status: calm when it works, loud only when the user must act. */
export const STATUS_TONE: Record<HookStatus, Tone> = {
  starting: "neutral",
  listening: "ok",
  "permission-denied": "danger",
  unsupported: "warn",
  error: "danger",
};

/** Whether the sidebar should flag the Shift+drag section. */
export const needsAttention = (status: HookStatus): boolean => STATUS_TONE[status] === "danger";

/** A label short enough for the narrow sidebar footer. */
export const STATUS_SHORT: Record<HookStatus, string> = {
  starting: "Starting",
  listening: "Shift+drag is on",
  "permission-denied": "Needs permission",
  unsupported: "Not supported",
  error: "Not running",
};
