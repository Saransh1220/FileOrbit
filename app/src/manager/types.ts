/** Mirrors the structs in src-tauri/src/manager.rs. */

export interface EngineInfo {
  name: string;
  version: string | null;
  /** "downloaded" (managed by the app) or "system" (already on this computer). */
  source: "downloaded" | "system" | null;
}

export interface PackInfo {
  id: string;
  title: string;
  description: string;
  engines: EngineInfo[];
  installed: boolean;
  /** Download size on this platform; null when nothing is published for it. */
  download_bytes: number | null;
  disk_bytes: number;
  /** Titles of other packs that use the same program. */
  shared_with: string[];
  busy: boolean;
}

export interface Overview {
  packs: PackInfo[];
  cache_bytes: number;
  installed_bytes: number;
  folder: string;
}

/** Live progress of an install, from the "manager-update" event. */
export interface PackUpdate {
  pack: string;
  state: "installing" | "done" | "failed";
  message: string | null;
  percent: number | null;
  error: string | null;
}

export type HookStatus = "starting" | "listening" | "permission-denied" | "unsupported" | "error";

/** A design choice for the settings picker: [id, label]. */
export type DesignChoice = [string, string];
