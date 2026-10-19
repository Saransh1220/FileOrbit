import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useDesign } from "./design/useDesign";
import type { HookStatus } from "./status/hookStatus";
import { StatusView } from "./status/StatusView";

function App() {
  const [status, setStatus] = useState<HookStatus>("starting");
  useDesign();

  useEffect(() => {
    // The status may have been reported before this window existed.
    void invoke<HookStatus>("get_hook_status").then(setStatus);
    const s = listen<HookStatus>("hook-status", (e) => setStatus(e.payload));
    return () => {
      void s.then((f) => f());
    };
  }, []);

  return <StatusView status={status} />;
}

export default App;
