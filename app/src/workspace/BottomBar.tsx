import { Icon } from "./icons";
import type { SaveStatus } from "./workspaceProps";
import "./BottomBar.css";

interface BottomBarProps {
  total: number;
  selected: number;
  edited: boolean;
  status: SaveStatus;
  onSave: () => void;
  onShowSaved: (path: string) => void;
}

const pagesText = (count: number) => (count === 1 ? "1 page" : `${count} pages`);

/** What there is to save, how it went, and the one main button. */
export function BottomBar({ total, selected, edited, status, onSave, onShowSaved }: BottomBarProps) {
  const saving = status.kind === "saving";
  return (
    <footer className="bottombar">
      <p className="summary">
        {pagesText(total)}
        {selected > 0 && <span> · {selected} selected</span>}
        {!edited && status.kind !== "saved" && <span className="faint"> · no changes yet</span>}
      </p>

      <div className="result" role="status">
        {status.kind === "saved" && (
          <>
            <Icon name="check" />
            <span>
              Saved <strong>{status.name}</strong> ({pagesText(status.pages)})
            </span>
            <button type="button" className="link" onClick={() => onShowSaved(status.path)}>
              <Icon name="folder" /> Show in folder
            </button>
          </>
        )}
        {status.kind === "error" && <span className="error">{status.message}</span>}
      </div>

      <button type="button" className="primary" disabled={saving || !edited} onClick={onSave} title="Writes a new PDF next to the original (Ctrl+S)">
        <Icon name="save" />
        {saving ? "Saving…" : "Save as new PDF"}
      </button>
    </footer>
  );
}
