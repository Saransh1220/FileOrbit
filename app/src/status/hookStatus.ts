export type HookStatus = "starting" | "listening" | "permission-denied" | "unsupported" | "error";

export const STATUS_TEXT: Record<HookStatus, string> = {
  starting: "Starting…",
  listening: "Listening. Hold Shift and drag a file from Finder or Explorer.",
  "permission-denied":
    "Input Monitoring permission needed: System Settings → Privacy & Security → Input Monitoring, enable this app (or your terminal in dev), then restart.",
  unsupported: "Shift+drag detection is not available on this OS yet.",
  error: "Could not start the listener.",
};
