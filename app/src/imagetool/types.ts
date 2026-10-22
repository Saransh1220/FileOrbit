/** What the engine receives: the same shapes as `engine/imaging/spec.rs`. */
export type FitMode = "stretch" | "crop" | "pad";

export type Resize =
  | { kind: "percent"; percent: number }
  | { kind: "fit"; width: number; height: number }
  | { kind: "longEdge"; pixels: number }
  | { kind: "exact"; width: number; height: number; fit: FitMode }
  | { kind: "aspect"; width: number; height: number; fit: FitMode };

export type Compress = { kind: "quality"; quality: number } | { kind: "targetSize"; max_bytes: number };

export type OutFormat = "keep" | "jpg" | "png";

export interface Job {
  resize: Resize | null;
  compress: Compress | null;
  format: OutFormat;
}

/** A picture in the window's list. */
export interface ImageFile {
  path: string;
  name: string;
  width: number;
  height: number;
  bytes: number;
  /** What the preview shows for the original. */
  previewUrl: string;
}

/** What the engine says the chosen file becomes. */
export interface Estimate {
  width: number;
  height: number;
  bytes: number;
  notes: string[];
}
