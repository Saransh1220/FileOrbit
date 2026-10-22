import { useEffect, useMemo, useState } from "react";
import type { SaveStatus } from "./ActionBar";
import { ImageToolView } from "./ImageToolView";
import { demoParam, isClean } from "../demoParams";
import { PRESETS } from "./presets";
import { DEFAULT_SETTINGS, toJob, type Settings } from "./settings";
import type { Estimate, ImageFile, Job } from "./types";

const DESIGNS = ["current", "bold", "brutal"];

/** A picture-like SVG, so the preview has something to crop and scale. */
function scene(hue: number): string {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 300"><defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="hsl(${hue} 70% 72%)"/><stop offset="1" stop-color="hsl(${hue + 40} 80% 88%)"/></linearGradient></defs><rect width="400" height="300" fill="url(#g)"/><circle cx="300" cy="80" r="34" fill="hsl(${hue + 150} 90% 80%)"/><path d="M0 230 L90 140 L170 215 L250 120 L400 250 L400 300 L0 300Z" fill="hsl(${hue + 10} 35% 38%)"/><path d="M0 262 L120 200 L220 255 L330 190 L400 240 L400 300 L0 300Z" fill="hsl(${hue + 20} 40% 26%)"/></svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

const FILES: ImageFile[] = [
  { path: "/p/IMG_4821.jpg", name: "IMG_4821.jpg", width: 4000, height: 3000, bytes: 4_210_000, previewUrl: scene(200) },
  { path: "/p/holiday-beach.png", name: "holiday-beach.png", width: 3024, height: 4032, bytes: 9_800_000, previewUrl: scene(20) },
  { path: "/p/team photo.jpg", name: "team photo.jpg", width: 1920, height: 1080, bytes: 640_000, previewUrl: scene(110) },
];

/** A rough stand-in for the engine, only so the playground reacts. The real numbers come from Rust. */
function fakeEstimate(file: ImageFile, job: Job | null): Estimate | null {
  if (!job) return null;
  let { width, height } = file;
  const notes: string[] = [];
  const r = job.resize;
  if (r) {
    const shrinkTo = (s: number) => {
      s = Math.min(1, s);
      width = Math.max(1, Math.round(width * s));
      height = Math.max(1, Math.round(height * s));
    };
    if (r.kind === "percent") [width, height] = [Math.max(1, Math.round(file.width * (r.percent / 100))), Math.max(1, Math.round(file.height * (r.percent / 100)))];
    if (r.kind === "fit") shrinkTo(Math.min(r.width / width, r.height / height));
    if (r.kind === "longEdge") shrinkTo(r.pixels / Math.max(width, height));
    if (r.kind === "exact") [width, height] = [r.width, r.height];
    if (r.kind === "aspect") {
      const ratio = r.width / r.height;
      const wide = width / height > ratio;
      if (r.fit === "pad") [width, height] = wide ? [width, Math.round(width / ratio)] : [Math.round(height * ratio), height];
      else [width, height] = wide ? [Math.round(height * ratio), height] : [width, Math.round(width / ratio)];
    }
  }
  const png = job.format === "png" || (job.format === "keep" && file.name.endsWith(".png"));
  const quality = job.compress?.kind === "quality" ? job.compress.quality : 90;
  let bytes = Math.round(width * height * (png ? 1.4 : 0.05 + (quality / 100) ** 2 * 0.55));
  if (job.compress?.kind === "targetSize" && bytes > job.compress.max_bytes) {
    bytes = Math.round(job.compress.max_bytes * 0.97);
    notes.push("Shrunk to 70% of the size to fit.");
  }
  if (png && job.compress?.kind === "quality") notes.push("PNG has no quality setting; choose JPG or a size limit to make it smaller.");
  return { width, height, bytes, notes };
}

const bar = { display: "flex", gap: 6, flexWrap: "wrap" as const, padding: "10px 12px", background: "#fff", borderBottom: "1px solid #ddd", color: "#111" };

/** Dev-only playground (index.html#imagetool-demo) for judging the picture tool without building the app. */
export function ImageToolDemo() {
  const [design, setDesign] = useState("current");
  const [count, setCount] = useState(Number(demoParam("count")) || 1);
  const [selected, setSelected] = useState(0);
  const [settings, setSettings] = useState<Settings>(() => PRESETS.find((p) => p.id === demoParam("preset"))?.apply(DEFAULT_SETTINGS) ?? DEFAULT_SETTINGS);
  const [status, setStatus] = useState<SaveStatus>({ kind: "idle" });

  useEffect(() => {
    document.documentElement.dataset.design = design;
  }, [design]);

  const files = FILES.slice(0, count);
  const current = files[Math.min(selected, files.length - 1)];
  const { job, problems } = useMemo(() => toJob(settings), [settings]);
  const estimate = problems.length > 0 ? null : fakeEstimate(current, job);

  const pick = (label: string, active: boolean, on: () => void) => (
    <button key={label} onClick={on} style={{ fontWeight: active ? 700 : 400 }}>
      {label}
    </button>
  );

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
      {!isClean() && <div style={bar}>
        {DESIGNS.map((d) => pick(d, d === design, () => setDesign(d)))}
        {[1, 3].map((n) => pick(`${n} picture${n > 1 ? "s" : ""}`, n === count, () => (setCount(n), setSelected(0))))}
        {pick("saving", status.kind === "saving", () => setStatus({ kind: "saving", done: 1, total: 3 }))}
        {pick("saved", status.kind === "saved", () => setStatus({ kind: "saved", count, before: 4_210_000, after: 380_000, path: "/p/x.jpg" }))}
        {pick("error", status.kind === "error", () => setStatus({ kind: "error", message: "Could not read holiday-beach.png: the file is damaged." }))}
        {pick("idle", status.kind === "idle", () => setStatus({ kind: "idle" }))}
      </div>}
      <div style={{ flex: 1, minHeight: 0, display: "grid" }}>
        <ImageToolView
          files={files}
          selected={Math.min(selected, files.length - 1)}
          settings={settings}
          estimate={estimate}
          resultUrl={estimate ? current.previewUrl : null}
          problems={problems}
          canSave={job !== null}
          status={status}
          onSelect={setSelected}
          onRemove={() => {}}
          onAdd={() => {}}
          onSettings={setSettings}
          onSave={() => setStatus({ kind: "saved", count, before: 4_210_000, after: 380_000, path: "/p/x.jpg" })}
          onShowSaved={() => {}}
        />
      </div>
    </div>
  );
}
