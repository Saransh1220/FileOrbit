import { useEffect, useState } from "react";
import { demoParam, isClean } from "../demoParams";
import { ManagerView } from "./ManagerView";
import type { SectionId } from "./sections";
import type { HookStatus, Overview, PackUpdate } from "./types";

const DESIGNS: [string, string][] = [
  ["current", "Current"],
  ["bold", "Bold"],
  ["brutal", "Brutal"],
];
/** A theme the app offers but this window has no design for: shown to check the fallback. */
const OTHER_DESIGN: [string, string] = ["petals", "Petals"];

const OVERVIEW: Overview = {
  folder: "C:\\Users\\you\\AppData\\Local\\Radial Convert",
  cache_bytes: 374_870_016,
  installed_bytes: 183_000_000,
  packs: [
    {
      id: "pdf",
      title: "PDF tools",
      description: "PDF pages to PNG, JPG and text; merge, reorder and combine",
      engines: [{ name: "PDFium", version: "158.0.8111", source: "downloaded" }],
      installed: true,
      managed: true,
      download_bytes: 3_541_773,
      disk_bytes: 9_800_000,
      shared_with: [],
      busy: false,
    },
    {
      id: "documents",
      title: "Documents",
      description: "Word, Excel, PowerPoint, RTF to PDF and to each other",
      engines: [{ name: "LibreOffice", version: null, source: null }],
      installed: false,
      managed: false,
      download_bytes: 374_870_016,
      disk_bytes: 0,
      shared_with: [],
      busy: false,
    },
    {
      id: "text",
      title: "Text & Markup",
      description: "Markdown, HTML, EPUB, TeX, plain text",
      engines: [{ name: "pandoc", version: null, source: "system" }],
      installed: true,
      managed: false,
      download_bytes: 42_154_914,
      disk_bytes: 0,
      shared_with: [],
      busy: false,
    },
    {
      id: "audio",
      title: "Audio",
      description: "MP3, WAV, FLAC, AAC, M4A, OGG",
      engines: [{ name: "ffmpeg", version: "9.0.2", source: "downloaded" }],
      installed: true,
      managed: true,
      download_bytes: 77_232_696,
      disk_bytes: 183_000_000,
      shared_with: ["Video", "Images"],
      busy: false,
    },
    {
      id: "video",
      title: "Video",
      description: "MP4, MOV, MKV, WEBM, GIF, extract audio",
      engines: [{ name: "ffmpeg", version: "9.0.2", source: "downloaded" }],
      installed: false,
      managed: false,
      download_bytes: 77_232_696,
      disk_bytes: 183_000_000,
      shared_with: ["Audio", "Images"],
      busy: false,
    },
    {
      id: "images",
      title: "Images",
      description: "PNG, JPG, WEBP, TIFF, BMP, GIF",
      engines: [{ name: "ffmpeg", version: "9.0.2", source: "downloaded" }],
      installed: true,
      managed: true,
      download_bytes: 77_232_696,
      disk_bytes: 183_000_000,
      shared_with: ["Audio", "Video"],
      busy: false,
    },
  ],
};

const UPDATES: Record<string, Record<string, PackUpdate>> = {
  none: {},
  installing: {
    documents: {
      pack: "documents",
      state: "installing",
      message: "Downloading LibreOffice: 157 of 375 MB · 1.4 MB/s · 3 min left",
      percent: 42,
      error: null,
    },
  },
  unpacking: {
    documents: { pack: "documents", state: "installing", message: "Unpacking LibreOffice, this can take a minute", percent: null, error: null },
  },
  failed: {
    documents: {
      pack: "documents",
      state: "failed",
      message: null,
      percent: null,
      error: "the Windows installer service could not unpack the package (exit code 1603). The download is kept, so trying again will not download it again",
    },
  },
};

const bar = { display: "flex", gap: 6, flexWrap: "wrap" as const, padding: "10px 12px", background: "#fff", borderBottom: "1px solid #ddd", color: "#111" };

const PAGES: SectionId[] = ["converters", "appearance", "storage", "shortcut"];

/** Dev-only playground (index.html#manager-demo) for judging the manager window without building the app. */
export function ManagerDemo() {
  const [design, setDesign] = useState("current");
  const [updates, setUpdates] = useState("none");
  const [status, setStatus] = useState<HookStatus>("listening");
  const [empty, setEmpty] = useState(false);
  const [page, setPage] = useState<SectionId>((demoParam("page") as SectionId | null) ?? "converters");
  const [problem, setProblem] = useState(false);

  // The real app sets this on <html> for every theme, "current" included.
  useEffect(() => {
    document.documentElement.dataset.design = design;
  }, [design]);

  const choose = (label: string, value: string, current: string, set: (v: string) => void) => (
    <button key={value} onClick={() => set(value)} style={{ fontWeight: value === current ? 700 : 400 }}>
      {label}
    </button>
  );

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
      {!isClean() && <div style={bar}>
        {[...DESIGNS, OTHER_DESIGN].map(([id, label]) => choose(label, id, design, setDesign))}
        {PAGES.map((p) => choose(`page: ${p}`, p, page, (v) => setPage(v as SectionId)))}
        {Object.keys(UPDATES).map((k) => choose(`rows: ${k}`, k, updates, setUpdates))}
        {(["listening", "permission-denied", "starting"] as HookStatus[]).map((s) => choose(s, s, status, (v) => setStatus(v as HookStatus)))}
        <button onClick={() => setEmpty(!empty)} style={{ fontWeight: empty ? 700 : 400 }}>loading</button>
        <button onClick={() => setProblem(!problem)} style={{ fontWeight: problem ? 700 : 400 }}>problem</button>
      </div>}
      <div style={{ flex: 1, minHeight: 0, display: "grid" }}>
        <ManagerView
          key={page}
          initialSection={page}
          status={status}
          overview={empty ? null : OVERVIEW}
          updates={UPDATES[updates]}
          designs={[...DESIGNS, OTHER_DESIGN]}
          design={design}
          problem={problem ? "Another install is running. Wait for it to finish first." : null}
          onInstall={() => {}}
          onRemove={() => {}}
          onClearDownloads={() => {}}
          onOpenFolder={() => {}}
          onDesign={setDesign}
        />
      </div>
    </div>
  );
}
