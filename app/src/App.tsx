import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import "./App.css";

type HookStatus = "starting" | "listening" | "permission-denied" | "unsupported" | "error";

const STATUS_TEXT: Record<HookStatus, string> = {
  starting: "Starting…",
  listening: "Listening. Hold Shift and drag a file from Finder.",
  "permission-denied":
    "Input Monitoring permission needed: System Settings → Privacy & Security → Input Monitoring, enable this app (or your terminal in dev), then restart.",
  unsupported: "This OS is not implemented yet.",
  error: "Could not start the listener.",
};

function App() {
  const [status, setStatus] = useState<HookStatus>("starting");

  useEffect(() => {
    // The status may have been reported before this window existed.
    void invoke<HookStatus>("get_hook_status").then(setStatus);
    const s = listen<HookStatus>("hook-status", (e) => setStatus(e.payload));
    return () => {
      void s.then((f) => f());
    };
  }, []);

  return (
    <main className="app">
      <h1>Radial Convert</h1>
      <p className={`status ${status}`}>{STATUS_TEXT[status]}</p>
    </main>
  );
}

export default App;
