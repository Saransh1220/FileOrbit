export type JobState = "running" | "done" | "failed" | "needs-pack" | "installing";

/** Mirrors `JobUpdate` in src-tauri/src/engine/jobs.rs. */
export interface JobUpdate {
  id: number;
  name: string;
  target: string;
  state: JobState;
  output: string | null;
  error: string | null;
  /** For needs-pack and installing: the pack id and its display title. */
  pack: string | null;
  pack_title: string | null;
  /** 0-100 while a download size is known. */
  progress: number | null;
  /** Short phase text such as "Downloading" or "Unpacking". */
  message: string | null;
}
