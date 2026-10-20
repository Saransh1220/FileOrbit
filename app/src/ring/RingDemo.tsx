import { useEffect, useState } from "react";
import { ToastView } from "../toast/ToastView";
import type { JobUpdate } from "../toast/types";
import "../toast/ToastSkins.css";
import { RING_VARIANTS, type RingVariant } from "./variants/ringVariants";
import type { Wedge } from "./types";

const toWedges = (ids: string[]): Wedge[] => ids.map((id) => ({ id, label: id.toUpperCase() }));

const SETS: Record<string, Wedge[]> = {
  docx: toWedges(["pdf", "docx", "odt", "txt", "html", "md"]),
  audio: toWedges(["mp3", "wav", "flac", "aac", "m4a"]),
  image: toWedges(["png", "jpg", "webp", "tiff", "bmp", "gif"]),
  video: toWedges(["mp4", "mkv", "webm", "gif", "mp3", "wav"]),
  // Several selected files: a tool wedge first (Merge or Combine), then what they share.
  pdfs: [{ id: "merge", label: "MERGE", tool: true }, ...toWedges(["png", "jpg", "txt", "docx", "odt"])],
  pictures: [{ id: "merge", label: "COMBINE", tool: true }, ...toWedges(["jpg", "webp", "tiff", "bmp", "gif", "pdf"])],
  mixed: toWedges(["pdf", "docx", "html", "md", "png", "mp3", "mp4", "xlsx"]),
  none: [],
};

const BACKGROUNDS: Record<string, string> = {
  "light wallpaper": "linear-gradient(135deg,#e9eef7,#cfd9ea 55%,#f2dcd3)",
  "dark wallpaper": "linear-gradient(135deg,#10131c,#1b2438 55%,#2a1f33)",
  "busy photo": "repeating-linear-gradient(45deg,#2b6a8f 0 18px,#e8b04a 18px 36px,#a23b52 36px 54px)",
};

const blank = { pack: null, pack_title: null, engine: null, download_mb: null, progress: null, message: null, count: 1, failed: 0 };

const JOBS: Record<string, JobUpdate> = {
  running: { id: 1, name: "Patient Acknowledgement Screen.docx", target: "PDF", state: "running", output: null, error: null, ...blank },
  done: { id: 2, name: "Patient Acknowledgement Screen.docx", target: "PDF", state: "done", output: "/Users/me/Downloads/Patient Acknowledgement Screen.pdf", error: null, ...blank },
  "needs-pack": { id: 4, name: "report.docx", target: "PDF", state: "needs-pack", output: null, error: null, pack: "documents", pack_title: "Documents", engine: "LibreOffice", download_mb: 375, progress: null, message: null, count: 1, failed: 0 },
  installing: { id: 5, name: "report.docx", target: "PDF", state: "installing", output: null, error: null, pack: "documents", pack_title: "Documents", engine: "LibreOffice", download_mb: null, progress: 42, message: "Downloading LibreOffice: 157 of 375 MB · 1.4 MB/s · 3 min left", count: 1, failed: 0 },
  unpacking: { id: 6, name: "report.docx", target: "PDF", state: "installing", output: null, error: null, pack: "documents", pack_title: "Documents", engine: "LibreOffice", download_mb: null, progress: null, message: "Unpacking LibreOffice, this can take a minute", count: 1, failed: 0 },
  "batch running": { id: 7, name: "5 files", target: "PDF", state: "running", output: null, error: null, ...blank, count: 5, message: "2 of 5: Quarterly report.docx" },
  "batch done": { id: 8, name: "5 files", target: "PDF", state: "done", output: "/Users/me/Downloads/Quarterly report.pdf", error: null, ...blank, count: 5 },
  "batch partial": { id: 9, name: "5 files", target: "PDF", state: "done", output: "/Users/me/Downloads/Quarterly report.pdf", error: null, ...blank, count: 3, failed: 2 },
  failed: { id: 3, name: "report.docx", target: "PDF", state: "failed", output: null, error: "no download is published for libreoffice on macos-aarch64 and none is installed on this machine", ...blank },
};

const FILE = "Patient Acknowledgement Screen.docx";

const bar = { display: "flex", gap: 6, flexWrap: "wrap" as const, marginBottom: 10 };

/** Dev-only playground (index.html#ring-demo) for judging the design without building the app. */
export function RingDemo() {
  const [set, setSet] = useState("mixed");
  const [hovered, setHovered] = useState<number | null>(0);
  const [confirmed, setConfirmed] = useState<number | null>(null);
  const [bg, setBg] = useState("light wallpaper");
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const [design, setDesign] = useState("orbit");
  const [compare, setCompare] = useState(true);
  const [job, setJob] = useState("done");
  const [replay, setReplay] = useState(0);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const pick = (k: string) => {
    setSet(k);
    setConfirmed(null);
    setHovered(SETS[k].length ? 0 : null);
    setReplay((n) => n + 1);
  };
  const count = SETS[set].length;


  const show = ({ View }: RingVariant) => (
    <>
      <View
        key={replay}
        wedges={SETS[set]}
        fileName={FILE}
        hovered={hovered}
        confirmed={confirmed}
        onConfirmed={() => {}}
      />
      <div style={{ width: 420 }}>
        <ToastView job={JOBS[job]} onDismiss={() => {}} onReveal={() => {}} onInstall={() => {}} />
      </div>
    </>
  );

  return (
    <div style={{ height: "100vh", overflow: "auto", padding: 20, boxSizing: "border-box", background: BACKGROUNDS[bg], fontFamily: "-apple-system, sans-serif" }}>
      <div style={bar}>
        {Object.keys(SETS).map((k) => (
          <button key={k} onClick={() => pick(k)}>{k}</button>
        ))}
      </div>
      <div style={bar}>
        <button onClick={() => setCompare(!compare)} style={{ fontWeight: compare ? 700 : 400 }}>compare all</button>
        {RING_VARIANTS.map((v) => (
          <button key={v.id} onClick={() => { setDesign(v.id); setCompare(false); }} style={{ fontWeight: !compare && v.id === design ? 700 : 400 }}>design: {v.label}</button>
        ))}
      </div>
      <div style={bar}>
        <button onClick={() => setTheme(theme === "light" ? "dark" : "light")}>theme: {theme}</button>
        {Object.keys(BACKGROUNDS).map((k) => (
          <button key={k} onClick={() => setBg(k)}>{k}</button>
        ))}
      </div>
      <div style={bar}>
        {Object.keys(JOBS).map((k) => (
          <button key={k} onClick={() => setJob(k)} style={{ fontWeight: k === job ? 700 : 400 }}>toast: {k}</button>
        ))}
      </div>
      <div style={bar}>
        <button onClick={() => { setConfirmed(null); setHovered(null); }}>no hover</button>
        {Array.from({ length: count }, (_, i) => (
          <button key={i} onClick={() => { setConfirmed(null); setHovered(i); }}>hover {i + 1}</button>
        ))}
        <button onClick={() => setConfirmed(hovered ?? 0)}>drop</button>
        <button onClick={() => { setConfirmed(null); setReplay((n) => n + 1); }}>replay open</button>
      </div>

      {compare ? (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(2, minmax(0, 1fr))", gap: 8 }}>
          {RING_VARIANTS.map((v) => (
            <section key={v.id} data-design={v.id} style={{ zoom: 0.8 }}>
              <div style={{ marginBottom: 6, fontSize: 13, fontWeight: 600, color: theme === "dark" ? "#fff" : "#222" }}>{v.label}</div>
              {show(v)}
            </section>
          ))}
        </div>
      ) : (
        show(RING_VARIANTS.find((v) => v.id === design) ?? RING_VARIANTS[0])
      )}
    </div>
  );
}
