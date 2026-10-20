import type { JobUpdate } from "./types";
import "./Toast.css";

const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;

export interface ToastViewProps {
  job: JobUpdate;
  onDismiss: () => void;
  onReveal: () => void;
  onInstall: () => void;
}

function icon(job: JobUpdate) {
  switch (job.state) {
    case "running":
    case "installing":
      return <span className="spinner" />;
    case "done":
      return "✓";
    case "needs-pack":
      return "↓";
    default:
      return "!";
  }
}

/** Merge and Combine make one PDF from several files, rather than converting each. */
const isMerge = (job: JobUpdate) => job.target === "MERGE";

function title(job: JobUpdate) {
  switch (job.state) {
    case "running":
      return isMerge(job) ? "Making one PDF…" : `Converting to ${job.target}…`;
    case "done":
      return job.count > 1 ? `Converted ${job.count} files` : `Saved ${fileName(job.output ?? "")}`;
    case "needs-pack":
      return `Install ${job.pack_title} support?`;
    case "installing":
      return `Installing ${job.engine ?? "converter"} for ${job.pack_title}…`;
    default:
      return isMerge(job) ? "Couldn’t make the PDF" : `Couldn’t convert to ${job.target}`;
  }
}

function needsPackDetail(job: JobUpdate) {
  const size = job.download_mb === null ? "" : ` · ${job.download_mb} MB download`;
  return `Needs ${job.engine ?? "a converter"}${size}`;
}

function doneDetail(job: JobUpdate) {
  if (job.message) return job.message;
  if (job.count <= 1 && job.failed === 0) return job.name;
  const failed = job.failed > 0 ? ` · ${job.failed} could not be converted` : "";
  return `Next to the originals${failed}`;
}

function detail(job: JobUpdate) {
  switch (job.state) {
    case "running":
      return job.message ?? job.name;
    case "done":
      return doneDetail(job);
    case "failed":
      return job.error;
    case "needs-pack":
      return needsPackDetail(job);
    case "installing": {
      const percent = job.progress === null ? "" : ` · ${job.progress}%`;
      return `${job.message ?? "Starting"}${percent}`;
    }
    default:
      return job.name;
  }
}

/** Presentational toast: state comes from props so it can be previewed without Tauri. */
export function ToastView({ job, onDismiss, onReveal, onInstall }: ToastViewProps) {
  return (
    // Key by job and state so the done-state fade restarts for every result.
    <div
      key={`${job.id}-${job.state}`}
      className={`toast ${job.state}`}
      onAnimationEnd={(e) => {
        if (job.state === "done" && e.animationName === "toast-out") onDismiss();
      }}
    >
      <div className="icon">{icon(job)}</div>
      <div className="body">
        <div className="title">{title(job)}</div>
        <div className="detail">{detail(job)}</div>
      </div>
      {job.state === "done" && <button onClick={onReveal}>Show</button>}
      {job.state === "failed" && <button onClick={onDismiss}>Dismiss</button>}
      {job.state === "needs-pack" && (
        <>
          <button className="ghost" onClick={onDismiss} aria-label="Not now" title="Not now">{"\u2715"}</button>
          <button className="primary" onClick={onInstall}>Install</button>
        </>
      )}
      {job.state === "installing" && (
        <div className="bar" aria-hidden>
          <i style={{ width: `${job.progress ?? 0}%` }} className={job.progress === null ? "indeterminate" : ""} />
        </div>
      )}
    </div>
  );
}
