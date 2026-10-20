import { useEffect, useState } from "react";
import { ManagerView } from "./ManagerView";
import type { HookStatus, Overview, PackUpdate } from "./types";
import "./ManagerSkins.css";

const DESIGNS: [string, string][] = [
  ["current", "Current"],
  ["bold", "Bold"],
  ["brutal", "Brutal"],
  ["petals", "Petals"],
  ["compass", "Compass"],
  ["fan", "Fan"],
  ["hex", "Hex"],
];

const OVERVIEW: Overview = {
  folder: "C:\\Users\\you\\AppData\\Local\\Radial Convert",
  cache_bytes: 374_870_016,
  installed_bytes: 183_000_000,
  packs: [
    {
      id: "documents",
      title: "Documents",
      description: "Word, Excel, PowerPoint, RTF to PDF and to each other",
      engines: [{ name: "LibreOffice", version: null, source: null }],
      installed: false,
      download_bytes: 374_870_016,
      disk_bytes: 0,
      shared_with: [],
      busy: false,
    },
    {
      id: "text",
      title: "Text & Markup",
      description: "Markdown, HTML, EPUB, TeX, plain text",
      engines: [{ name: "pandoc", version: null, source: null }],
      installed: false,
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

const bar = { display: "flex", gap: 6, flexWrap: "wrap" as const, padding: "10px 12px", background: "#fff", borderBottom: "1px solid #ddd" };

/** Dev-only playground (index.html#manager-demo) for judging the manager window without building the app. */
export function ManagerDemo() {
  const [design, setDesign] = useState("current");
  const [updates, setUpdates] = useState("none");
  const [status, setStatus] = useState<HookStatus>("listening");
  const [empty, setEmpty] = useState(false);

  useEffect(() => {
    if (design === "current") delete document.documentElement.dataset.design;
    else document.documentElement.dataset.design = design;
  }, [design]);

  const choose = (label: string, value: string, current: string, set: (v: string) => void) => (
    <button key={value} onClick={() => set(value)} style={{ fontWeight: value === current ? 700 : 400 }}>
      {label}
    </button>
  );

  return (
    <div style={{ height: "100vh", overflow: "auto" }}>
      <div style={bar}>
        {DESIGNS.map(([id, label]) => choose(label, id, design, setDesign))}
        {Object.keys(UPDATES).map((k) => choose(`rows: ${k}`, k, updates, setUpdates))}
        {(["listening", "permission-denied", "starting"] as HookStatus[]).map((s) => choose(s, s, status, (v) => setStatus(v as HookStatus)))}
        <button onClick={() => setEmpty(!empty)} style={{ fontWeight: empty ? 700 : 400 }}>loading</button>
      </div>
      <ManagerView
        status={status}
        overview={empty ? null : OVERVIEW}
        updates={UPDATES[updates]}
        designs={DESIGNS}
        design={design}
        problem={null}
        onInstall={() => {}}
        onRemove={() => {}}
        onClearDownloads={() => {}}
        onOpenFolder={() => {}}
        onDesign={setDesign}
      />
    </div>
  );
}
