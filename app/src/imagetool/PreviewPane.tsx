import { formatBytes } from "../manager/format";
import type { Estimate, ImageFile } from "./types";
import "./PreviewPane.css";

const saved = (before: number, after: number) => {
  if (before <= 0) return null;
  const change = Math.round((1 - after / before) * 100);
  return change >= 0 ? `${change}% smaller` : `${-change}% larger`;
};

/** The original beside what it becomes. `resultUrl` is the engine's own render, so what is shown is what is saved. */
export function PreviewPane({ file, estimate, resultUrl }: { file: ImageFile | null; estimate: Estimate | null; resultUrl: string | null }) {
  if (!file) return <div className="preview empty">Drop pictures here, or choose Add.</div>;
  const change = estimate ? saved(file.bytes, estimate.bytes) : null;
  return (
    <div className="preview">
      <figure>
        <div className="frame">
          <img src={file.previewUrl} alt="Original" style={{ aspectRatio: `${file.width} / ${file.height}` }} />
        </div>
        <figcaption>
          <strong>Original</strong>
          <span>
            {file.width} × {file.height} · {formatBytes(file.bytes)}
          </span>
        </figcaption>
      </figure>
      <span className="arrow" aria-hidden>
        →
      </span>
      <figure>
        <div className="frame">
          {resultUrl && estimate ? (
            <img src={resultUrl} alt="Result" style={{ aspectRatio: `${estimate.width} / ${estimate.height}` }} />
          ) : (
            <p className="idle">Switch on Resize, Compress or a format to see the result.</p>
          )}
        </div>
        <figcaption>
          <strong>Result</strong>
          {estimate ? (
            <span>
              {estimate.width} × {estimate.height} · {formatBytes(estimate.bytes)}
              {change && <b className="gain"> · {change}</b>}
            </span>
          ) : (
            <span>nothing to do yet</span>
          )}
        </figcaption>
      </figure>
      {estimate && estimate.notes.length > 0 && (
        <ul className="notes">
          {estimate.notes.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
