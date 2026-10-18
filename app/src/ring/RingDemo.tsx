import { useEffect, useState } from "react";
import { ToastView } from "../toast/ToastView";
import type { JobUpdate } from "../toast/types";
import { RingView } from "./RingView";
import type { Wedge } from "./types";

const toWedges = (ids: string[]): Wedge[] => ids.map((id) => ({ id, label: id.toUpperCase() }));

const SETS: Record<string, Wedge[]> = {
  docx: toWedges(["pdf", "docx", "odt", "txt", "html", "md"]),
  audio: toWedges(["mp3", "wav", "flac", "aac", "m4a"]),
  image: toWedges(["png", "jpg", "webp", "tiff", "bmp", "gif"]),
  video: toWedges(["mp4", "mkv", "webm", "gif", "mp3", "wav"]),
  mixed: toWedges(["pdf", "docx", "html", "md", "png", "mp3", "mp4", "xlsx"]),
  none: [],
};

const BACKGROUNDS: Record<string, string> = {
  "light wallpaper": "linear-gradient(135deg,#e9eef7,#cfd9ea 55%,#f2dcd3)",
  "dark wallpaper": "linear-gradient(135deg,#10131c,#1b2438 55%,#2a1f33)",
  "busy photo": "repeating-linear-gradient(45deg,#2b6a8f 0 18px,#e8b04a 18px 36px,#a23b52 36px 54px)",
};

const blank = { pack: null, pack_title: null, progress: null, message: null };

const JOBS: Record<string, JobUpdate> = {
  running: { id: 1, name: "Patient Acknowledgement Screen.docx", target: "PDF", state: "running", output: null, error: null, ...blank },
  done: { id: 2, name: "Patient Acknowledgement Screen.docx", target: "PDF", state: "done", output: "/Users/me/Downloads/Patient Acknowledgement Screen.pdf", error: null, ...blank },
  "needs-pack": { id: 4, name: "report.docx", target: "PDF", state: "needs-pack", output: null, error: null, pack: "documents", pack_title: "Documents", progress: null, message: null },
  installing: { id: 5, name: "report.docx", target: "PDF", state: "installing", output: null, error: null, pack: "documents", pack_title: "Documents", progress: 42, message: "Downloading" },
  unpacking: { id: 6, name: "report.docx", target: "PDF", state: "installing", output: null, error: null, pack: "documents", pack_title: "Documents", progress: null, message: "Unpacking" },
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

  return (
    <div style={{ minHeight: "100vh", padding: 20, boxSizing: "border-box", background: BACKGROUNDS[bg], fontFamily: "-apple-system, sans-serif" }}>
      <div style={bar}>
        {Object.keys(SETS).map((k) => (
          <button key={k} onClick={() => pick(k)}>{k}</button>
        ))}
      </div>
      <div style={bar}>
        <button onClick={() => setTheme(theme === "light" ? "dark" : "light")}>theme: {theme}</button>
        {Object.keys(BACKGROUNDS).map((k) => (
          <button key={k} onClick={() => setBg(k)}>{k}</button>
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

      <RingView
        key={replay}
        wedges={SETS[set]}
        fileName={FILE}
        hovered={hovered}
        confirmed={confirmed}
        onConfirmed={() => {}}
      />

      <div style={{ ...bar, marginTop: 16 }}>
        {Object.keys(JOBS).map((k) => (
          <button key={k} onClick={() => setJob(k)}>toast: {k}</button>
        ))}
      </div>
      <div style={{ width: 340 }}>
        <ToastView job={JOBS[job]} onDismiss={() => {}} onReveal={() => {}} onInstall={() => {}} />
      </div>
    </div>
  );
}
