import { formatBytes } from "../manager/format";
import "./ActionBar.css";

export type SaveStatus =
  | { kind: "idle" }
  | { kind: "saving"; done: number; total: number }
  | { kind: "saved"; count: number; before: number; after: number; path: string }
  | { kind: "error"; message: string };

export interface ActionBarProps {
  count: number;
  /** Why the job cannot run yet (bad number), or nothing. */
  problems: string[];
  canSave: boolean;
  status: SaveStatus;
  onSave: () => void;
  onShowSaved: (path: string) => void;
}

/** What will be saved, how the last save went, and the one main button. */
export function ActionBar({ count, problems, canSave, status, onSave, onShowSaved }: ActionBarProps) {
  const saving = status.kind === "saving";
  const label = saving ? `Saving ${status.done + 1} of ${status.total}…` : count === 1 ? "Save as new picture" : `Save ${count} new pictures`;
  return (
    <footer className="actions">
      <div className="state" role="status">
        {problems[0] && <span className="problem">{problems[0]}</span>}
        {!problems[0] && status.kind === "saved" && (
          <>
            <span>
              Saved {status.count === 1 ? "1 picture" : `${status.count} pictures`}: {formatBytes(status.before)} → {formatBytes(status.after)}
            </span>
            <button type="button" className="link" onClick={() => onShowSaved(status.path)}>
              Show in folder
            </button>
          </>
        )}
        {!problems[0] && status.kind === "error" && <span className="problem">{status.message}</span>}
        {!problems[0] && status.kind === "idle" && <span className="faint">The original is never changed.</span>}
      </div>
      <button type="button" className="save" disabled={!canSave || saving} onClick={onSave}>
        {label}
      </button>
    </footer>
  );
}
