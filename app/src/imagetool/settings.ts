import type { Compress, FitMode, Job, OutFormat, Resize } from "./types";

export type ResizeMode = "percent" | "fit" | "longEdge" | "exact" | "ratio";
export type SizeUnit = "KB" | "MB";

/** The window's state: every control's value, even for a part that is switched off. */
export interface Settings {
  resize: {
    on: boolean;
    mode: ResizeMode;
    percent: number;
    width: number;
    height: number;
    longEdge: number;
    fit: FitMode;
    /** One of RATIOS' ids; "custom" uses ratioWidth and ratioHeight. */
    ratio: string;
    ratioWidth: number;
    ratioHeight: number;
  };
  compress: { on: boolean; mode: "quality" | "size"; quality: number; sizeValue: number; sizeUnit: SizeUnit };
  format: OutFormat;
}

export const RATIOS = [
  { id: "1:1", width: 1, height: 1 },
  { id: "4:5", width: 4, height: 5 },
  { id: "3:2", width: 3, height: 2 },
  { id: "16:9", width: 16, height: 9 },
  { id: "9:16", width: 9, height: 16 },
];

export const DEFAULT_SETTINGS: Settings = {
  resize: { on: false, mode: "fit", percent: 50, width: 1600, height: 1600, longEdge: 1600, fit: "crop", ratio: "1:1", ratioWidth: 1, ratioHeight: 1 },
  compress: { on: false, mode: "size", quality: 80, sizeValue: 500, sizeUnit: "KB" },
  format: "keep",
};

/** Kilobytes here are 1000 bytes, as in Finder and Explorer, so "under 500 KB" holds in both. */
export function sizeToBytes(value: number, unit: SizeUnit): number {
  return Math.round(value * (unit === "MB" ? 1_000_000 : 1_000));
}

const whole = (n: number) => Number.isInteger(n) && n > 0;

function resizeOf(s: Settings["resize"]): { resize: Resize | null; problem: string | null } {
  const need = (ok: boolean, text: string) => (ok ? null : text);
  switch (s.mode) {
    case "percent":
      return { resize: { kind: "percent", percent: s.percent }, problem: need(whole(s.percent) && s.percent <= 1000, "Percentage must be between 1 and 1000.") };
    case "fit":
      return { resize: { kind: "fit", width: s.width, height: s.height }, problem: need(whole(s.width) && whole(s.height), "Width and height must be whole numbers above zero.") };
    case "longEdge":
      return { resize: { kind: "longEdge", pixels: s.longEdge }, problem: need(whole(s.longEdge), "The longer side must be a whole number above zero.") };
    case "exact":
      return { resize: { kind: "exact", width: s.width, height: s.height, fit: s.fit }, problem: need(whole(s.width) && whole(s.height), "Width and height must be whole numbers above zero.") };
    case "ratio": {
      const known = RATIOS.find((r) => r.id === s.ratio);
      const [width, height] = known ? [known.width, known.height] : [s.ratioWidth, s.ratioHeight];
      return { resize: { kind: "aspect", width, height, fit: s.fit === "pad" ? "pad" : "crop" }, problem: need(whole(width) && whole(height), "The ratio needs two whole numbers above zero.") };
    }
  }
}

function compressOf(s: Settings["compress"]): { compress: Compress; problem: string | null } {
  if (s.mode === "quality") {
    return { compress: { kind: "quality", quality: s.quality }, problem: s.quality >= 1 && s.quality <= 100 ? null : "Quality must be between 1 and 100." };
  }
  return { compress: { kind: "targetSize", max_bytes: sizeToBytes(s.sizeValue, s.sizeUnit) }, problem: s.sizeValue > 0 ? null : "The size limit must be above zero." };
}

/** The job to run, or why it cannot run (`job` null with no problem means nothing is switched on). */
export function toJob(settings: Settings): { job: Job | null; problems: string[] } {
  const problems: string[] = [];
  let resize: Resize | null = null;
  let compress: Compress | null = null;
  if (settings.resize.on) {
    const r = resizeOf(settings.resize);
    resize = r.resize;
    if (r.problem) problems.push(r.problem);
  }
  if (settings.compress.on) {
    const c = compressOf(settings.compress);
    compress = c.compress;
    if (c.problem) problems.push(c.problem);
  }
  if (problems.length > 0) return { job: null, problems };
  if (!resize && !compress && settings.format === "keep") return { job: null, problems };
  return { job: { resize, compress, format: settings.format }, problems };
}
