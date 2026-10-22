import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import type { SaveStatus } from "./ActionBar";
import { ImageToolView } from "./ImageToolView";
import { useDesign } from "../design/useDesign";
import { DEFAULT_SETTINGS, toJob, type Settings } from "./settings";
import type { Job } from "./types";
import { usePictures } from "./usePictures";
import { usePreview } from "./usePreview";

interface SavedReply {
  saved: { output: string; before_bytes: number; after_bytes: number }[];
  errors: string[];
}

/** The picture tool window: loads the pictures, previews the chosen one, and runs the save. */
export function ImageTool() {
  useDesign();
  const { files, selected, setSelected, problem, choose, remove } = usePictures();
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [status, setStatus] = useState<SaveStatus>({ kind: "idle" });
  const { job, problems } = useMemo(() => toJob(settings), [settings]);
  const current = files[Math.min(selected, files.length - 1)] ?? null;
  const preview = usePreview(current?.path ?? null, problems.length === 0 ? job : null);

  useEffect(() => {
    const progress = listen<[number, number]>("imagetool-progress", (e) => setStatus({ kind: "saving", done: e.payload[0], total: e.payload[1] }));
    return () => void progress.then((off) => off());
  }, []);

  const save = async (run: Job) => {
    setStatus({ kind: "saving", done: 0, total: files.length });
    try {
      const reply = await invoke<SavedReply>("imagetool_save", { paths: files.map((f) => f.path), job: run });
      if (reply.saved.length === 0) return setStatus({ kind: "error", message: reply.errors[0] ?? "Nothing was saved." });
      setStatus({
        kind: "saved",
        count: reply.saved.length,
        before: reply.saved.reduce((n, s) => n + s.before_bytes, 0),
        after: reply.saved.reduce((n, s) => n + s.after_bytes, 0),
        path: reply.saved[0].output,
      });
    } catch (e) {
      setStatus({ kind: "error", message: typeof e === "string" ? e : "Could not save." });
    }
  };

  const shownProblems = problems.length > 0 ? problems : preview.error ? [preview.error] : problem ? [problem] : [];
  return (
    <ImageToolView
      files={files}
      selected={Math.min(selected, Math.max(0, files.length - 1))}
      settings={settings}
      estimate={preview.estimate}
      resultUrl={preview.resultUrl}
      problems={shownProblems}
      canSave={job !== null && files.length > 0}
      status={status}
      onSelect={setSelected}
      onRemove={remove}
      onAdd={() => void choose()}
      onSettings={(next) => {
        setSettings(next);
        setStatus({ kind: "idle" });
      }}
      onSave={() => job && void save(job)}
      onShowSaved={(path) => void invoke("reveal_in_file_manager", { path })}
    />
  );
}
