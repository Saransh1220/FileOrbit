import type { HookStatus } from "./types";

/** One line for the header: what the app is doing with Shift+drag. */
export const STATUS_LABEL: Record<HookStatus, string> = {
  starting: "Starting",
  listening: "Listening for Shift+drag",
  "permission-denied": "Needs permission",
  unsupported: "Not supported here",
  error: "Not running",
};

/** Longer help, shown only when the status needs the user to do something. */
export const STATUS_HELP: Partial<Record<HookStatus, string>> = {
  "permission-denied":
    "Allow this app under System Settings, Privacy & Security, Input Monitoring, then restart it.",
  unsupported: "Shift+drag detection is not available on this system yet.",
  error: "The Shift+drag listener could not start.",
};
