import { useMemo, useState } from "react";
import { describeGroup, everyN, parseRanges, type Groups } from "./ranges";
import "./SplitPanel.css";

interface SplitPanelProps {
  /** Pages in the plan; the split cuts these. */
  total: number;
  onSplit: (groups: Groups) => void;
  onClose: () => void;
}

type Mode = "every" | "ranges";

/** Cuts the plan into several PDFs: every N pages, or the ranges the user types. */
export function SplitPanel({ total, onSplit, onClose }: SplitPanelProps) {
  const [mode, setMode] = useState<Mode>("every");
  const [size, setSize] = useState("2");
  const [ranges, setRanges] = useState("");

  const result = useMemo(() => (mode === "every" ? everyN(total, Number(size)) : parseRanges(ranges, total)), [mode, size, ranges, total]);

  return (
    <section className="split" aria-label="Split into several PDFs">
      <div className="head">
        <h3>Split into several PDFs</h3>
        <button type="button" className="close" onClick={onClose}>
          Close
        </button>
      </div>

      <div className="modes" role="radiogroup">
        <label>
          <input type="radio" name="split-mode" checked={mode === "every"} onChange={() => setMode("every")} /> Every
          <input className="number" type="number" min={1} max={total} value={size} onChange={(e) => setSize(e.target.value)} onFocus={() => setMode("every")} aria-label="Pages per file" />
          pages
        </label>
        <label>
          <input type="radio" name="split-mode" checked={mode === "ranges"} onChange={() => setMode("ranges")} /> These ranges
          <input className="text" type="text" placeholder="1-3, 5, 8-" value={ranges} onChange={(e) => setRanges(e.target.value)} onFocus={() => setMode("ranges")} aria-label="Page ranges" />
        </label>
      </div>

      {result.ok ? (
        <p className="preview">
          {result.groups.length} {result.groups.length === 1 ? "file" : "files"}: {result.groups.slice(0, 6).map(describeGroup).join(" · ")}
          {result.groups.length > 6 && " · …"}
        </p>
      ) : (
        <p className="preview error">{result.error}</p>
      )}

      <button type="button" className="go" disabled={!result.ok} onClick={() => result.ok && onSplit(result.groups)}>
        Save {result.ok ? result.groups.length : ""} PDFs
      </button>
    </section>
  );
}
