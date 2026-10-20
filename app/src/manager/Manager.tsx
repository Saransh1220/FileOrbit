import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import { useDesign } from "../design/useDesign";
import { ManagerView } from "./ManagerView";
import type { DesignChoice, HookStatus, Overview, PackUpdate } from "./types";
import "./ManagerSkins.css";

const message = (error: unknown) => (typeof error === "string" ? error : error instanceof Error ? error.message : "Something went wrong.");

/** The manager window: loads what is installed, follows installs live, and runs the actions. */
export function Manager() {
  const design = useDesign();
  const [status, setStatus] = useState<HookStatus>("starting");
  const [overview, setOverview] = useState<Overview | null>(null);
  const [updates, setUpdates] = useState<Record<string, PackUpdate>>({});
  const [designs, setDesigns] = useState<DesignChoice[]>([]);
  const [problem, setProblem] = useState<string | null>(null);

  const refresh = useCallback(() => {
    invoke<Overview>("get_overview").then(setOverview, (e) => setProblem(message(e)));
  }, []);

  useEffect(() => {
    refresh();
    void invoke<DesignChoice[]>("list_designs").then(setDesigns);
    // The hook may have reported before this window existed.
    void invoke<HookStatus>("get_hook_status").then(setStatus);

    const listeners = [
      listen<HookStatus>("hook-status", (e) => setStatus(e.payload)),
      listen("manager-changed", refresh),
      listen<PackUpdate>("manager-update", (e) => {
        const update = e.payload;
        // A finished install needs no row of its own: the refreshed list shows it as installed.
        setUpdates((all) => {
          const next = { ...all };
          if (update.state === "done") delete next[update.pack];
          else next[update.pack] = update;
          return next;
        });
      }),
    ];
    return () => listeners.forEach((l) => void l.then((off) => off()));
  }, [refresh]);

  const run = (command: string, args?: Record<string, unknown>) => {
    setProblem(null);
    invoke(command, args).catch((e) => setProblem(message(e)));
  };

  return (
    <ManagerView
      status={status}
      overview={overview}
      updates={updates}
      designs={designs}
      design={design}
      problem={problem}
      onInstall={(pack) => {
        // Clear an old failure right away so the row reads "Starting" instead of the stale error.
        setUpdates((all) => ({ ...all, [pack]: { pack, state: "installing", message: "Starting", percent: null, error: null } }));
        run("install_pack", { pack });
      }}
      onRemove={(pack) => run("remove_pack", { pack })}
      onClearDownloads={() => run("clear_downloads")}
      onOpenFolder={() => run("open_data_folder")}
      onDesign={(id) => run("set_design", { id })}
    />
  );
}
