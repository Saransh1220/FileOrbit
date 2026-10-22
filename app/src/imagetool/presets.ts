import type { Settings } from "./settings";

/** One click that switches on and fills the parts of the window it needs. */
export interface Preset {
  id: string;
  label: string;
  hint: string;
  apply: (base: Settings) => Settings;
}

export const PRESETS: Preset[] = [
  {
    id: "email",
    label: "Email",
    hint: "Under 1 MB, up to 2000 px",
    apply: (s) => ({
      ...s,
      resize: { ...s.resize, on: true, mode: "longEdge", longEdge: 2000 },
      compress: { ...s.compress, on: true, mode: "size", sizeValue: 1, sizeUnit: "MB" },
      format: "jpg",
    }),
  },
  {
    id: "web",
    label: "Web",
    hint: "Up to 1600 px, good quality",
    apply: (s) => ({
      ...s,
      resize: { ...s.resize, on: true, mode: "longEdge", longEdge: 1600 },
      compress: { ...s.compress, on: true, mode: "quality", quality: 80 },
      format: "jpg",
    }),
  },
  {
    id: "square",
    label: "Square",
    hint: "1080 × 1080, cropped",
    apply: (s) => ({
      ...s,
      resize: { ...s.resize, on: true, mode: "exact", width: 1080, height: 1080, fit: "crop" },
      compress: { ...s.compress, on: false },
      format: "keep",
    }),
  },
  {
    id: "story",
    label: "Story",
    hint: "1080 × 1920, cropped",
    apply: (s) => ({
      ...s,
      resize: { ...s.resize, on: true, mode: "exact", width: 1080, height: 1920, fit: "crop" },
      compress: { ...s.compress, on: false },
      format: "keep",
    }),
  },
  {
    id: "thumbnail",
    label: "Thumbnail",
    hint: "Up to 400 px, small file",
    apply: (s) => ({
      ...s,
      resize: { ...s.resize, on: true, mode: "longEdge", longEdge: 400 },
      compress: { ...s.compress, on: true, mode: "quality", quality: 70 },
      format: "jpg",
    }),
  },
];
