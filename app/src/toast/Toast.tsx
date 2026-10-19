import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useDesign } from "../design/useDesign";
import { ToastView } from "./ToastView";
import "./ToastSkins.css";
import type { JobUpdate } from "./types";

export function Toast() {
  const [job, setJob] = useState<JobUpdate | null>(null);
  useDesign();

  useEffect(() => {
    const unlisten = listen<JobUpdate>("job-update", (e) => setJob(e.payload));
    return () => void unlisten.then((f) => f());
  }, []);

  if (!job) return null;

  const dismiss = () => void invoke("hide_toast");
  const reveal = () => {
    if (job.output) void invoke("reveal_in_file_manager", { path: job.output });
    dismiss();
  };

  const install = () => void invoke("install_pack_for_job", { jobId: job.id });

  return <ToastView job={job} onDismiss={dismiss} onReveal={reveal} onInstall={install} />;
}
