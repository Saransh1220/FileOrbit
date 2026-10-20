import { useEffect, useMemo, useState } from "react";
import { limitedLoader } from "./thumbnails";
import type { OpenedFile } from "./types";
import { useWorkspace } from "./useWorkspace";
import { WorkspaceView } from "./WorkspaceView";
import "../manager/ManagerSkins.css";
import "./WorkspaceSkins.css";

const DESIGNS: [string, string][] = [
  ["current", "Current"],
  ["bold", "Bold"],
  ["brutal", "Brutal"],
  ["petals", "Petals"],
  ["compass", "Compass"],
  ["fan", "Fan"],
  ["hex", "Hex"],
];

/** A mix of portrait, landscape and one very tall page, so the layout is tried on real shapes. */
const SHAPES = [
  { width: 595, height: 842 },
  { width: 595, height: 842 },
  { width: 842, height: 595 },
  { width: 595, height: 842 },
  { width: 420, height: 1200 },
  { width: 595, height: 842 },
  { width: 842, height: 595 },
  { width: 595, height: 842 },
  { width: 595, height: 842 },
  { width: 612, height: 792 },
  { width: 612, height: 792 },
  { width: 595, height: 842 },
];

const REPORT: OpenedFile = { path: "C:\Users\you\Documents\Quarterly report.pdf", name: "Quarterly report.pdf", pages: SHAPES };
const INVOICE: OpenedFile = { path: "C:\Users\you\Documents\Invoice March.pdf", name: "Invoice March.pdf", pages: SHAPES.slice(0, 3) };
const CONTRACT: OpenedFile = { path: "C:\Users\you\Documents\Contract.pdf", name: "Contract.pdf", pages: SHAPES.slice(3, 8) };
const MORE = [INVOICE, CONTRACT];

/** A page drawn as a picture: its number, a heading bar and lines of "text". */
function pageSvg(number: number, width: number, height: number): string {
  const lines = Array.from({ length: 14 }, (_, i) => `<rect x="${width * 0.12}" y="${height * 0.3 + i * height * 0.04}" width="${width * (i % 4 === 3 ? 0.4 : 0.76)}" height="${height * 0.012}" fill="#cfd3dc"/>`).join("");
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}"><rect width="100%" height="100%" fill="#fff"/><rect x="${width * 0.12}" y="${height * 0.08}" width="${width * 0.5}" height="${height * 0.03}" fill="#3a4258"/><text x="${width * 0.12}" y="${height * 0.22}" font-family="Arial" font-size="${height * 0.09}" font-weight="700" fill="#4f7bb8">${number}</text>${lines}</svg>`;
}


/** Stands in for the PDF engine: a picture arrives after the next frame, as it would from the app. */
const mockLoad = limitedLoader(
  (_path, page) =>
    new Promise<string>((resolve) => {
      const { width, height } = SHAPES[page % SHAPES.length];
      requestAnimationFrame(() => resolve(`data:image/svg+xml;utf8,${encodeURIComponent(pageSvg(page + 1, width, height))}`));
    }),
  3,
);

type Mode = "ready" | "two files" | "loading" | "problem";

const bar = { display: "flex", gap: 6, flexWrap: "wrap" as const, padding: "10px 12px", background: "#fff", borderBottom: "1px solid #ddd" };

const leaf = (path: string) => path.split("\\").pop() ?? path;

/** Dev-only playground (index.html#workspace-demo) for judging the page editor without building the app. */
export function WorkspaceDemo() {
  const [design, setDesign] = useState("current");
  const [mode, setMode] = useState<Mode>("ready");
  const [fail, setFail] = useState(false);
  const [dropping, setDropping] = useState(false);
  const [dropped, setDropped] = useState<string[]>([]);

  useEffect(() => {
    if (design === "current") delete document.documentElement.dataset.design;
    else document.documentElement.dataset.design = design;
  }, [design]);

  const initial = useMemo(() => (mode === "ready" ? [REPORT] : mode === "two files" ? [REPORT, INVOICE] : []), [mode]);
  const view = useWorkspace({
    initial,
    load: mockLoad,
    openMore: (paths) => Promise.resolve(MORE.filter((file) => paths.includes(file.path))),
    pickFiles: () => Promise.resolve([CONTRACT.path]),
    dropping,
    dropped,
    save: (files, pages, kind, part) =>
      new Promise((resolve, reject) =>
        requestAnimationFrame(() =>
          fail
            ? reject("cannot write the PDF: the disk is full")
            : resolve({ path: `${files[0].replace(/\.pdf$/, "")} (${part ? `part ${part}` : kind === "extract" ? "extract" : "edited"}).pdf`, pages: pages.length }),
        ),
      ),
    showInFolder: () => {},
    problem: mode === "problem" ? `${leaf(REPORT.path)} is protected by a password` : null,
    loading: mode === "loading",
  });

  const pick = (label: string, value: string, current: string, set: (v: string) => void) => (
    <button key={value} onClick={() => set(value)} style={{ fontWeight: value === current ? 700 : 400 }}>
      {label}
    </button>
  );

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
      <div style={bar}>
        {DESIGNS.map(([id, label]) => pick(label, id, design, setDesign))}
        {(["ready", "two files", "loading", "problem"] as Mode[]).map((m) => pick(m, m, mode, (v) => setMode(v as Mode)))}
        <button onClick={() => setDropping(!dropping)} style={{ fontWeight: dropping ? 700 : 400 }}>
          drag over
        </button>
        <button onClick={() => setDropped([INVOICE.path])}>drop invoice</button>
        <button onClick={() => setFail(!fail)} style={{ fontWeight: fail ? 700 : 400 }}>
          saving fails
        </button>
      </div>
      <div style={{ flex: 1, minHeight: 0, ["--workspace-height" as string]: "100%" }}>
        <WorkspaceView {...view} />
      </div>
    </div>
  );
}
